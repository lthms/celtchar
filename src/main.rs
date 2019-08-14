extern crate clap;
extern crate ogmarkup;
extern crate serde_derive;
extern crate serde_json;
extern crate toml;
#[macro_use]
extern crate tera;
extern crate zip;

use clap::{App, SubCommand};

pub mod error;
pub mod render;
pub mod project;
pub mod epub;

use std::path::PathBuf;

#[cfg(debug_assertions)]
use std::env::current_dir;
#[cfg(debug_assertions)]
use error::Raise;

use ogmarkup::typography::FRENCH;

use error::Error;
use project::Project;
use epub::{Zip, Fs};

use epub::EpubWriter;

pub fn build(assets : &PathBuf) -> Result<(), Error> {
    Project::cd_root()?;

    let project = Project::find_project()?
        .load_and_render(&FRENCH)?;

    let mut zip_writer = Zip::init()?;
    zip_writer.generate(&project, assets)?;

    let mut fs_writer = Fs::init()?;
    fs_writer.generate(&project, assets)?;

    Ok(())
}

#[cfg(debug_assertions)]
fn get_assets() -> Result<PathBuf, Error> {
    current_dir().or_raise("cannot get current directory")
}

#[cfg(not(debug_assertions))]
fn get_assets() -> Result<PathBuf, Error> {
    Ok(PathBuf::from("/usr/local/share/celtchar"))
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

    let assets: PathBuf = get_assets()?;

    match subcommand {
        "build"  => build(&assets)?,
        _        => unimplemented!(),
    }

    Ok(())
}
