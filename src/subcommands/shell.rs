/*
 *  Avatar CLI: Magic wrapper to run containerized CLI tools
 *  Copyright (C) 2019-2020  Andres Correa Casablanca
 *  License: GPL 3.0 (See the LICENSE file in the repository root directory)
 */

use std::env;
use std::os::unix::process::CommandExt; // Brings trait that allows us to use exec
use std::{
    path::PathBuf,
    process::{exit, Command},
};

extern crate exitcode;
extern crate rand;
use rand::{distributions::Alphanumeric, thread_rng, Rng};

use crate::avatar_env::{CONFIG_LOCK_PATH, CONFIG_PATH, PROJECT_PATH, SESSION_TOKEN, STATE_PATH};
use crate::directories::get_project_path;
use crate::project_config::{get_config, get_config_lock, ProjectConfigLock};

pub(crate) fn shell_subcommand() -> () {
    if let Ok(session_token) = env::var(SESSION_TOKEN) {
        eprintln!(
            "You are already in an Avatar CLI session (with token '{}').\nIf the environment changed, consider typing 'exit' and trying again.",
            session_token
        );
        exit(exitcode::USAGE)
    }

    let project_path = match get_project_path() {
        Some(p) => p,
        None => {
            eprintln!("The command was not executed inside an Avatar CLI project directory");
            exit(exitcode::USAGE)
        }
    };

    // We do not check again if config_path exists, since it was implicitly checked by `get_project_path`.
    let config_path = project_path.join(".avatar-cli").join("avatar-cli.yml");
    let config_lock_path = project_path.join(".avatar-cli").join("avatar-cli.lock.yml");
    let project_state_path = project_path
        .join(".avatar-cli")
        .join("volatile")
        .join("state.yml");

    let project_state =
        check_project_settings(&config_path, &config_lock_path, &project_state_path);
    check_oci_images_availability(&project_state);

    let shell_path = match env::var("SHELL") {
        Ok(sp) => sp,
        Err(_) => "/bin/sh".to_string(),
    };

    let path_var = match env::var("PATH") {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Unable to load PATH environment variable");
            exit(exitcode::OSERR)
        }
    };
    let avatar_bin_path = project_path
        .join(".avatar-cli")
        .join("volatile")
        .join("bin");
    let path_var = format!("{}:{}", avatar_bin_path.display(), path_var);

    let session_token: String = thread_rng().sample_iter(&Alphanumeric).take(16).collect();

    Command::new(shell_path)
        .env("PATH", path_var)
        .env(CONFIG_PATH, config_path)
        .env(CONFIG_LOCK_PATH, config_lock_path)
        .env(PROJECT_PATH, project_path)
        .env(SESSION_TOKEN, session_token)
        .env(STATE_PATH, project_state_path)
        .exec();
}

fn check_project_settings(
    config_path: &PathBuf,
    config_lock_path: &PathBuf,
    project_state_path: &PathBuf,
) -> ProjectConfigLock {
    if !config_lock_path.exists() || !config_lock_path.is_file() {
        eprintln!("Avatar CLI does not yet implement the implicit 'install' step");
        exit(exitcode::SOFTWARE) // TODO: Trigger implicit "install" step (but here it will do more stuff than in the previous case)
    }

    if !project_state_path.exists() || !project_state_path.is_file() {
        eprintln!("Avatar CLI does not yet implement the implicit 'install' step");
        exit(exitcode::SOFTWARE) // TODO: Trigger implicit "install" step
    }

    let (_, config_hash) = get_config(&config_path);
    let (config_lock, config_lock_hash) = get_config_lock(&config_lock_path);

    if &config_hash.as_ref() != &&config_lock.getProjectConfigHash()[..] {
        eprintln!(
            "The hash for the file '{}' does not match with the one in '{}'",
            config_path.display(),
            config_lock_path.display()
        );
        exit(exitcode::DATAERR) // TODO: Update config_lock & state instead of stopping the process
    }

    let (project_state, _) = get_config_lock(&project_state_path);

    if &config_lock_hash.as_ref() != &&project_state.getProjectConfigHash()[..] {
        eprintln!(
            "The hash for the file '{}' does not match with the one in '{}'",
            config_lock_path.display(),
            project_state_path.display()
        );
        exit(exitcode::DATAERR) // TODO: Update state instead of stopping the process
    }

    return project_state;
}

fn check_oci_images_availability(project_state: &ProjectConfigLock) {
    let images = project_state.getImages();

    let docker_client_path = match which::which("docker") {
        Ok(p) => p,
        Err(_) => {
            eprintln!("docker client is not available");
            exit(exitcode::UNAVAILABLE)
        }
    };

    for (image_name, image_tags) in images.iter() {
        for (_, image_hash) in image_tags.iter() {
            let inspect_output = Command::new(&docker_client_path)
                .args(&["inspect", &format!("{}@sha256:{}", image_name, image_hash)])
                .output();

            match inspect_output {
                Ok(output) => {
                    if !output.status.success() {
                        // TODO: Pull image
                        eprintln!("Image {}@sha256:{} not available", image_name, image_hash);
                        exit(exitcode::UNAVAILABLE)
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Unable to use docker to inspect image {}@sha256:{}.\n\n{}\n",
                        image_name,
                        image_hash,
                        e.to_string()
                    );
                    exit(exitcode::OSERR)
                }
            }
        }
    }
}
