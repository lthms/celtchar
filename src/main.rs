extern crate clap;
extern crate ogmarkup;
extern crate serde_derive;
extern crate toml;

use clap::{App, SubCommand};

pub mod render;
pub mod project;
pub mod epub;

use project::{Project, Error};
use ogmarkup::typography::FRENCH;

use std::fs::{create_dir, remove_dir_all};

const BUILD_DIR : &'static str = "_build";

fn cd_clean_build_dir() -> Result<(), Error> {
    remove_dir_all(BUILD_DIR)
        .map_err(|_| Error(String::from("cannot clean up _build/")))?;

    create_dir(BUILD_DIR)
        .map_err(|_| Error(String::from("cannot create _build/")))?;

    std::env::set_current_dir(BUILD_DIR)
        .map_err(|_| Error(String::from("cannot set current directory to _build")))?;

    Ok(())
}

pub fn build() -> Result<(), Error> {
    Project::cd_root()?;

    let project = Project::find_project()?
        .load_and_render(&FRENCH)?;

    cd_clean_build_dir()?;

    epub::generate(&project)?;

    Ok(())
}

fn main() -> Result<(), Error> {
    let matches = App::new("celtchar")
        .version("0.1")
        .author("Thomas Letan")
        .about("A tool to generate novels")
        .subcommand(SubCommand::with_name("new")
                    .about("Create a new celtchar document"))
        .subcommand(SubCommand::with_name("build")
                    .about("Build a celtchar document"))
        .get_matches();

    let (subcommand, _args) = matches.subcommand();

    match subcommand {
        "build"  => build()?,
        _        => unimplemented!(),
    }

    Ok(())
}
