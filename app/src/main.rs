extern crate clap;
extern crate libceltchar;
extern crate ogmarkup;
extern crate serde_json;
extern crate tera;
extern crate toml;

use std::path::PathBuf;

use clap::{App, Arg, SubCommand};

use libceltchar::{EpubWriter, Error, Loader, Project, Static, Zip};

#[cfg(debug_assertions)]
use libceltchar::Raise;
#[cfg(debug_assertions)]
use std::env::current_dir;

mod filesystem;
use crate::filesystem::{find_root, Fs};

fn deps() -> Result<(), Error> {
    let root = find_root()?;
    let loader = Fs;
    let project = loader.load_project(&root)?;

    let mut files = vec![];

    for mut chapter in project.chapters.into_iter() {
        files.append(&mut chapter.content)
    }

    for file in files {
        println!("{}", file.to_str().unwrap_or("<invalid utf8 filename>"));
    }

    Ok(())
}

fn build_epub(assets : &PathBuf) -> Result<(), Error> {
    let root = find_root()?;
    let loader = Fs;

    let project = Project::load_and_render(&root, &loader)?;

    let mut zip_writer = Zip::init()?;
    zip_writer.generate_epub(&project, assets)?;

    Ok(())
}

fn build_static(assets : &PathBuf, body_only : bool, out : &PathBuf) -> Result<(), Error> {
    let root = find_root()?;
    let loader = Fs;

    let project = Project::load_and_render(&root, &loader)?;

    let mut static_website = Static::init(out, body_only)?;
    static_website.generate_static_website(&project, assets)?;

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
        .subcommand(SubCommand::with_name("new").about("Create a new celtchar document"))
        .subcommand(SubCommand::with_name("epub").about("Build a epub"))
        .subcommand(
            SubCommand::with_name("static")
                .about("Build a static website")
                .arg(
                    Arg::with_name("body-only")
                        .help("Only output the bodies of the documents.")
                        .takes_value(false)
                        .short("b")
                        .long("body-only"),
                )
                .arg(
                    Arg::with_name("output")
                        .value_name("DIRECTORY")
                        .help("Output directory where the generated documents are saved")
                        .takes_value(true)
                        .short("o")
                        .long("output"),
                ),
        )
        .subcommand(SubCommand::with_name("build").about("Build a celtchar document"))
        .subcommand(SubCommand::with_name("deps").about("List dependencies of a celtchar document"))
        .get_matches();

    let assets : PathBuf = get_assets()?;

    match matches.subcommand() {
        ("epub", _) => build_epub(&assets)?,
        ("static",Some(args)) => {
            let body_only = args.is_present("body-only");
            let output_dir = PathBuf::from(
                args.value_of("output").unwrap_or("out")
            );
            build_static(&assets, body_only, &output_dir)?
        },
        ("deps", _) => deps()?,
        _ => unimplemented!(),
    }

    Ok(())
}
