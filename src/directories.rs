/*
 *  Avatar CLI: Magic wrapper to run containerized CLI tools
 *  Copyright (C) 2019-2020  Andres Correa Casablanca
 *  License: GPL 3.0 (See the LICENSE file in the repository root directory)
 */

use std::env;
use std::path::PathBuf;
use std::process::exit;

pub(crate) fn get_project_path() -> Option<PathBuf> {
    let current_dir = match env::current_dir() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Unable to get current working directory");
            exit(exitcode::NOINPUT)
        }
    };

    for ancestor in current_dir.ancestors() {
        let config_path = ancestor.join(".avatar-cli").join("avatar-cli.yml");
        if config_path.exists() && config_path.is_file() {
            return Some(ancestor.to_owned());
        }
    }

    None
}

pub(crate) fn check_if_inside_project_dir(project_path: &PathBuf, current_dir: &PathBuf) -> () {
    let mut in_project_dir = false;
    for ancestor in current_dir.ancestors() {
        if ancestor == project_path {
            in_project_dir = true;
            break;
        }
    }
    if !in_project_dir {
        eprintln!(
            "The configured project directory is '{}', but you are in '{}'",
            project_path.display(),
            current_dir.display()
        );
        exit(exitcode::USAGE)
    }
}