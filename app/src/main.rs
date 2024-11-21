extern crate clap;
extern crate libceltchar;
extern crate ogam;
extern crate serde_json;
extern crate tera;
extern crate toml;

use std::path::PathBuf;

use clap::{App, Arg, SubCommand};

use libceltchar::{Chapter, Content, EpubWriter, Error, Loader, Part, Project, Static, Zip};

#[cfg(debug_assertions)]
use libceltchar::Raise;
#[cfg(debug_assertions)]
use std::env::current_dir;

use ogam::stats::Digest;

mod filesystem;
use crate::filesystem::{find_root, Fs};

fn deps() -> Result<(), Error> {
    let root = find_root()?;
    let loader = Fs;
    let project = loader.load_project(&root)?;

    let mut files = vec![];

    for chapter in project.content.chapters() {
        files.append(&mut chapter.content.clone())
    }

    for file in files {
        println!("{}", file.to_str().unwrap_or("<invalid utf8 filename>"));
    }

    Ok(())
}

fn build_epub(assets: &PathBuf) -> Result<(), Error> {
    let root = find_root()?;
    let loader = Fs;

    let project = Project::load_and_render(&root, &loader)?;

    let mut zip_writer = Zip::init()?;
    zip_writer.generate_epub(&project, assets)?;

    Ok(())
}

fn build_static(assets: &PathBuf, body_only: bool, out: &PathBuf) -> Result<(), Error> {
    let root = find_root()?;
    let loader = Fs;

    let project = Project::load_and_render(&root, &loader)?;

    let mut static_website = Static::init(out, body_only)?;
    static_website.generate_static_website(&project, assets)?;

    Ok(())
}

fn wc_chapters(chapters: &Vec<Chapter<Digest>>, mut idx: usize) -> usize {
    let mut res = 0;

    for c in chapters {
        let mut curr = 0usize;

        for d in c.content.iter() {
            curr += d.words_count;
        }

        println!(
            "{} ({})",
            c.title
                .as_ref()
                .map(|x| format!("{}. {}", idx, x))
                .unwrap_or(format!("Chapter {}", idx)),
            curr
        );

        res += curr;
        idx += 1;
    }

    res
}

fn wc_parts(parts: &Vec<Part<Digest>>) -> usize {
    let mut res = 0;
    let mut idx = 1;
    let mut chap_idx = 1;

    for p in parts {
        let part_count = p.content.iter().fold(0, |acc, chap| {
            chap.content.iter().fold(acc, |acc, d| acc + d.words_count)
        });

        println!(
            "{} ({})",
            p.title
                .as_ref()
                .map(|x| format!("{}. {}", idx, x))
                .unwrap_or(format!("Part {}", idx)),
            part_count
        );

        for c in &p.content {
            let chap_count = c.content.iter().fold(0, |acc, d| acc + d.words_count);

            println!(
                "  {} ({})",
                c.title
                    .as_ref()
                    .map(|x| format!("{}. {}", chap_idx, x))
                    .unwrap_or(format!("Chapter {}", chap_idx)),
                chap_count
            );

            chap_idx += 1;
        }

        res += part_count;
        idx += 1;
    }

    res
}

fn wc() -> Result<(), Error> {
    let root = find_root()?;
    let loader = Fs;

    let project: Project<_, Digest> = Project::load_and_render(&root, &loader)?;

    let res = match project.content {
        Content::WithChapters(ref chaps) => wc_chapters(chaps, 0),
        Content::WithParts(ref parts) => wc_parts(parts),
    };

    println!("Total: {}", res);

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

fn main_with_error() -> Result<(), Error> {
    let matches = App::new("celtchar")
        .version("0.1")
        .author("Thomas Letan")
        .about("A tool to generate novels")
        .subcommand(SubCommand::with_name("new").about("Create a new celtchar document"))
        .subcommand(SubCommand::with_name("wc").about("World count"))
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
        .subcommand(SubCommand::with_name("deps").about("List dependencies of a celtchar document"))
        .get_matches();

    let assets: PathBuf = get_assets()?;

    match matches.subcommand() {
        ("wc", _) => wc()?,
        ("epub", _) => build_epub(&assets)?,
        ("static", Some(args)) => {
            let body_only = args.is_present("body-only");
            let output_dir = PathBuf::from(args.value_of("output").unwrap_or("out"));
            build_static(&assets, body_only, &output_dir)?
        }
        ("deps", _) => deps()?,
        _ => unimplemented!(),
    }

    Ok(())
}

fn main() -> () {
    match main_with_error() {
        Err(Error(msg)) => eprintln!("error: {}", msg),
        Ok(x) => x,
    }
}
