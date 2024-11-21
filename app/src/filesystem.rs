use std::env::{current_dir, set_current_dir};
use std::fs;
use std::fs::canonicalize;
use std::path::PathBuf;

use libceltchar::{Chapter, Content, Cover, Error, Loader, Part, Project, Raise};

const PROJECT_FILE: &str = "Book.toml";
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
                return Err(Error::new("could not find Book.toml"));
            }
        }
    }
}

fn canonicalize_chapter(chapter: &Chapter<PathBuf>) -> Result<Chapter<PathBuf>, Error> {
    let title = chapter.title.clone();
    Ok(Chapter {
        title: title,
        content: chapter
            .content
            .iter()
            .map(|x| canonicalize(x).or_raise(&format!("Could not canonicalize {:?}", x)))
            .collect::<Result<_, Error>>()?,
    })
}

fn canonicalize_part(part: &Part<PathBuf>) -> Result<Part<PathBuf>, Error> {
    let title = part.title.clone();
    Ok(Part {
        title,
        content: part
            .content
            .iter()
            .map(canonicalize_chapter)
            .collect::<Result<_, Error>>()?,
    })
}

fn canonicalize_content(content: &Content<PathBuf>) -> Result<Content<PathBuf>, Error> {
    match content {
        Content::WithParts(parts) => Ok(Content::WithParts(
            parts
                .iter()
                .map(canonicalize_part)
                .collect::<Result<_, Error>>()?,
        )),
        Content::WithChapters(chapters) => Ok(Content::WithChapters(
            chapters
                .iter()
                .map(canonicalize_chapter)
                .collect::<Result<_, Error>>()?,
        )),
    }
}

fn canonicalize_project(
    project: Project<PathBuf, PathBuf>,
) -> Result<Project<PathBuf, PathBuf>, Error> {
    Ok(Project {
        author: project.author,
        title: project.title,
        description: project.description,
        cover: project
            .cover
            .map(canonicalize)
            .map_or(Ok(None), |r| r.map(Some))
            .or_raise("…")?,
        numbering: project.numbering,
        content: canonicalize_content(&project.content)?,
        language: project.language,
    })
}

impl Loader for Fs {
    type ProjId = PathBuf;
    type CovId = PathBuf;
    type DocId = PathBuf;

    fn load_project(&self, id: &PathBuf) -> Result<Project<PathBuf, PathBuf>, Error> {
        let cwd = current_dir().or_raise("could not get current dir")?;

        let input =
            fs::read_to_string(PROJECT_FILE).or_raise("found Book.toml, but cannot read it")?;

        // We have to modify set the current directory to the PROJECT_FILE directory,
        // otherwise `canonicalize` will not work.
        set_current_dir(id).or_raise("could not change the current directory")?;
        let res = canonicalize_project(
            toml::from_str(input.as_str())
                .map_err(|e| Error(format!("Could not parse Book.toml: {}", e)))?,
        )?;
        set_current_dir(cwd).or_raise("could not change the current directory")?;

        Ok(res)
    }

    fn load_cover(&self, id: &PathBuf) -> Result<Cover, Error> {
        let extension = id
            .extension()
            .or_raise("cover lacks an extension")?
            .to_str()
            .or_raise("cover extension is not valid utf-8")?;

        let content = fs::read(id).or_raise(&format!("could not read cover from {:?}", id))?;

        Ok(Cover {
            extension: String::from(extension),
            content: content,
        })
    }

    fn load_document(&self, id: &PathBuf) -> Result<String, Error> {
        fs::read_to_string(id).or_raise(&format!("Could not read {:?}", id))
    }
}
