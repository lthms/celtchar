extern crate clap;
extern crate serde_json;
extern crate toml;
extern crate libceltchar;
extern crate ogmarkup;
extern crate tera;

use std::path::PathBuf;

use clap::{App, SubCommand};

use ogmarkup::typography::FRENCH;

use libceltchar::{Error, Zip, EpubWriter, Loader};

#[cfg(debug_assertions)]
use std::env::current_dir;
#[cfg(debug_assertions)]
use libceltchar::Raise;

mod filesystem;
use crate::filesystem::{find_root, Fs};

fn build(assets : &PathBuf) -> Result<(), Error> {
    let root = find_root()?;
    let loader = Fs;

    let project = loader.load_project(&root)?
        .load_and_render(&loader, &FRENCH)?;

    let mut zip_writer = Zip::init()?;
    zip_writer.generate(&project, assets)?;

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
