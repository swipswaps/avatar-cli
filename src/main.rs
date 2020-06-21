/*
 *  Avatar CLI: Magic wrapper to run containerized CLI tools
 *  Copyright (C) 2019-2020  Andres Correa Casablanca
 *  License: GPL 3.0 (See the LICENSE file in the repository root directory)
 */

extern crate atty;
extern crate exitcode;
extern crate which;

use std::env;
use std::fs::read;
use std::io::ErrorKind;
use std::path::{PathBuf, MAIN_SEPARATOR};
use std::os::unix::process::CommandExt; // Brings trait that allows us to use exec
use std::process::{exit, Command};

mod project_config;
use project_config::ProjectConfigLock;

fn get_config_lock_vec(config_lock_filepath: &PathBuf) -> Vec<u8> {
    if !config_lock_filepath.exists() || !config_lock_filepath.is_file() {
        eprintln!(
            "The lock file {} is not available",
            &config_lock_filepath.display()
        );
        exit(exitcode::NOINPUT)
    }

    match read(config_lock_filepath) {
        Ok(s) => s,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {
                eprintln!(
                    "The lock file {} is not available",
                    config_lock_filepath.display()
                );
                exit(exitcode::NOINPUT)
            }
            ErrorKind::PermissionDenied => {
                eprintln!(
                    "The lock file {} is not readable due to filesystem permissions",
                    config_lock_filepath.display()
                );
                exit(exitcode::IOERR)
            }
            _ => {
                eprintln!(
                    "Unknown IO error while reading the lock file {}",
                    config_lock_filepath.display()
                );
                exit(exitcode::IOERR)
            }
        }
    }
}

fn get_config_lock(config_lock_slice: &[u8], config_lock_filepath: &PathBuf) -> ProjectConfigLock {
    match serde_yaml::from_slice::<ProjectConfigLock>(config_lock_slice) {
        Ok(_config_lock) => _config_lock,
        Err(e) => {
            let error_msg = match e.location() {
                Some(l) => format!(
                    "Malformed lock file '{}', line {}, column {}:\n\t{}",
                    config_lock_filepath.display(),
                    l.line(),
                    l.column(),
                    e.to_string(),
                ),
                None => format!(
                    "Malformed lock file '{}':\n\t{}",
                    config_lock_filepath.display(),
                    e.to_string(),
                ),
            };

            eprintln!("{}", error_msg);
            exit(exitcode::DATAERR)
        }
    }
}

fn main() {
    let cmd_args: Vec<String> = env::args().collect();
    if cmd_args.is_empty() {
        eprintln!(
            "Due to an unknown reason, it was impossible to retrieve the command arguments list"
        );
        exit(exitcode::OSERR);
    }
    let used_program_name = match (&cmd_args[0]).split(MAIN_SEPARATOR).last() {
        Some(pname) => pname,
        None => {
            eprintln!("Due to an unknown reason, an empty first command argument was passed to this process");
            exit(exitcode::OSERR)
        }
    };

    if used_program_name == "avatar" || used_program_name == "avatar-cli" {
        println!("This code path has not been defined yet");
        exit(exitcode::SOFTWARE)
    }

    let config_lock_filepath = PathBuf::from(match env::var("AVATAR_CLI_CONFIG_LOCK_PATH") {
        Ok(fp) => fp,
        Err(_) => {
            eprintln!("The AVATAR_CLI_CONFIG_LOCK_PATH environment variable is not defined");
            exit(exitcode::CONFIG)
        }
    });

    let config_lock_vec = get_config_lock_vec(&config_lock_filepath);
    let config_lock = get_config_lock(&config_lock_vec, &config_lock_filepath);

    let binary_configuration = match config_lock.getBinaryConfiguration(used_program_name) {
        Some(c) => c,
        None => {
            eprintln!("Binary '{}' not properly configure in lock file '{}'", used_program_name, &config_lock_filepath.display());
            exit(1)
        }
    };

    let mut interactive_options: Vec<&str> = Vec::new();
    // TODO: Check if stdin is open
    interactive_options.push("-i");

    if atty::is(atty::Stream::Stdin) && atty::is(atty::Stream::Stdout) {
        interactive_options.push("-t")
    }

    let docker_client_path = match which::which("docker") {
        Ok(p) => p,
        Err(_) => {
            eprintln!("docker client is not available");
            exit(exitcode::UNAVAILABLE)
        }
    };

    Command::new(docker_client_path)
        .arg("run")
        .arg("--rm")
        .args(interactive_options)
        .arg(format!(
            "{}@sha256:{}",
            binary_configuration.getOCIImageName(),
            binary_configuration.getOCIImageHash()
        ))
        .arg(binary_configuration.getPath())
        .args(env::args().skip(1))
        .exec(); // Only for UNIX
}
