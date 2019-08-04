extern crate clap;
extern crate ogmarkup;
extern crate serde_derive;
extern crate serde_json;
extern crate toml;
#[macro_use]
extern crate tera;

use clap::{App, SubCommand};

pub mod error;
pub mod render;
pub mod project;
pub mod epub;

use std::env::{current_dir, set_current_dir};
use std::fs::{create_dir, remove_dir_all};
use std::path::PathBuf;

use ogmarkup::typography::FRENCH;

use error::{Raise, Error};
use project::Project;

const BUILD_DIR : &'static str = "_build";

fn cd_clean_build_dir() -> Result<(), Error> {
    remove_dir_all(BUILD_DIR).or_raise("cannot clean up _build/")?;

    create_dir(BUILD_DIR).or_raise("cannot create _build/")?;

    set_current_dir(BUILD_DIR).or_raise("cannot set current directory to _build/")?;

    Ok(())
}

pub fn build(assets : &PathBuf) -> Result<(), Error> {
    Project::cd_root()?;

    let project = Project::find_project()?
        .load_and_render(&FRENCH)?;

    cd_clean_build_dir()?;

    epub::generate(&project, assets)?;

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

    // TODO: in release mode, look for /usr/share/celtchar/assets
    let assets: PathBuf = current_dir().or_raise("cannot get current directory")?;

    match subcommand {
        "build"  => build(&assets)?,
        _        => unimplemented!(),
    }

    Ok(())
}
