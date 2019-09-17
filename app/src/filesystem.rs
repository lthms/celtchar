use std::path::{PathBuf};
use std::fs;
use std::fs::{canonicalize};
use std::env::{current_dir, set_current_dir};

use libceltchar::{Loader, Raise, Error, Project, Chapter, Cover};

const PROJECT_FILE : &str = "Book.toml";
pub struct Fs;

pub fn find_root() -> Result<PathBuf, Error> {
    let mut cwd: PathBuf = current_dir().or_raise("cannot get current directory")?;

    loop {
        cwd.push(PROJECT_FILE); // (*)

        if cwd.exists() {
            cwd.pop(); // we pop the `Book.toml` previously pushed (see (*))

            return Ok(cwd);
        } else {
            // We `pop` a first time for `Book.toml`, since we have pushed
            // previously it (see (*))
            cwd.pop();

            // We `pop` a second time to get the parent directory of cwd.  If
            // `pop` returns false, we are at the root of the current FS, and
            // there is no project file to find.
            if !cwd.pop() {
                return Err(Error::new("could not find Book.toml"))
            }
        }
    }
}

fn canonicalize_chapter(
    chapter : &Chapter<Vec<PathBuf>>
) -> Result<Chapter<Vec<PathBuf>>, Error> {
    let title = chapter.title.clone();
    Ok(Chapter {
        title: title,
        content: chapter.content
            .iter()
            .map(|x| canonicalize(x).or_raise(&format!("Could not canonicalize {:?}", x)))
            .collect::<Result<_, Error>>()?
    })
}

fn canonicalize_project(
    project : Project<PathBuf, Vec<PathBuf>>
) -> Result<Project<PathBuf, Vec<PathBuf>>, Error> {
    Ok(Project {
        author: project.author,
        title: project.title,
        cover: project.cover.map(canonicalize)
            .map_or(Ok(None), |r| r.map(Some))
            .or_raise("…")?,
        numbering: project.numbering,
        chapters: project.chapters.iter().map(canonicalize_chapter)
            .collect::<Result<_, Error>>()?,
        language: project.language,
    })
}

impl Loader for Fs {
    type ProjId = PathBuf;
    type CovId = PathBuf;
    type DocId = PathBuf;

    fn load_project(
        &self,
        id : &PathBuf
    ) -> Result<Project<PathBuf, Vec<PathBuf>>, Error> {
        let cwd = current_dir().or_raise("could not get current dir")?;

        let input = fs::read_to_string(PROJECT_FILE)
            .or_raise("found Book.toml, but cannot read it")?;

        // We have to modify set the current directory to the PROJECT_FILE directory,
        // otherwise `canonicalize` will not work.
        set_current_dir(id).or_raise("could not change the current directory")?;
        let res = canonicalize_project(
            toml::from_str(input.as_str())
                .or_raise(&format!("could not parse Book.toml"))?
        )?;
        set_current_dir(cwd).or_raise("could not change the current directory")?;

        Ok(res)
    }

    fn load_cover(
        &self,
        id : &PathBuf
    ) -> Result<Cover, Error> {
        let extension = id.extension()
            .or_raise("cover lacks an extension")?
            .to_str()
            .or_raise("cover extension is not valid utf-8")?;

        let content = fs::read(id)
            .or_raise(&format!("could not read cover from {:?}", id))?;

        Ok(Cover {
            extension: String::from(extension),
            content: content,
        })
    }

    fn load_document(
        &self,
        id : &PathBuf
    ) -> Result<String, Error> {
        fs::read_to_string(id).or_raise(&format!("Could not read {:?}", id))
    }
}