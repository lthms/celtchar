use std::fs;
use std::path::PathBuf;
use std::env::current_dir;

use serde_derive::Deserialize;

use ogmarkup::typography::Typography;

use crate::render::Html;

const PROJECT_FILE: &'static str = "Book.toml";

#[derive(Debug)]
pub struct Error(pub String);

#[derive(Debug, Deserialize)]
pub struct Chapter<A> {
    pub title: String,
    pub content: A,
}

fn compile_file <'input, T> (
    path : PathBuf,
    typo : &T,
) -> Result<String, Error>
where
    T : Typography,
{
    let input = fs::read_to_string(path.as_path()).map_err(|_| Error(format!("cannot open {:?}", path)))?;

    ogmarkup::compile(input.as_str(), typo)
        .map(|x: Html| x.to_string())
        .map_err(|_| Error(format!("cannot render {:?}", path)))
}

impl Chapter<Vec<PathBuf>> {
    pub fn load_and_render <'input, T> (
        self,
        typo : &T,
    ) -> Result<Chapter<String>, Error>
    where
        T : Typography
    {

        self.content.iter()
            .map(|path| compile_file(path.to_path_buf(), typo))
            .collect::<Result<Vec<String>, Error>>()
            .map(|x| Chapter {
                title: self.title,
                content: x.join(""),
            })
    }
}

#[derive(Debug, Deserialize)]
pub struct Project<A> {
    pub author: String,
    pub title: String,
    pub chapters: Vec<Chapter<A>>
}

impl Project<Vec<PathBuf>> {
    pub fn cd_root() -> Result<(), Error> {
        let mut cwd: PathBuf = current_dir()
            .map_err(|_| Error(String::from("cannot get current directory")))?;

        loop {
            cwd.push(PROJECT_FILE); // (*)

            if cwd.exists() {
                cwd.pop();

                std::env::set_current_dir(cwd.as_path())
                    .map_err(|_| Error(format!("cannot set current directory to {:?}", cwd)))?;

                return Ok(());
            } else {
                // We `pop` a first time for `Book.toml`, since we have pushed
                // previously it (see (*))
                cwd.pop();

                // We `pop` a second time to get the parent directory of cwd.  If
                // `pop` returns false, we are at the root of the current FS, and
                // there is no project file to find.
                if !cwd.pop() {
                    return Err(Error(String::from("could not find Book.toml")))
                }
            }
        }
    }

    /// This function tries to open `./Book.toml`.  If it succeeds, it tries to
    /// read it as a TOML file.
    pub fn find_project() -> Result<Self, Error> {
        let input = fs::read_to_string(PROJECT_FILE)
            .map_err(|_| Error(String::from("found Book.toml, but cannot read it")))?;

        return toml::from_str(input.as_str())
            .map_err(|e| Error(String::from(format!("toml error: {:?}", e))));
    }

    pub fn load_and_render<'input, T> (
        self,
        typo : &T,
    ) -> Result<Project<String>, Error>
    where
        T : Typography
    {
        let author = self.author;
        let title = self.title;

        self.chapters.into_iter()
            .map(|chapter| chapter.load_and_render(typo))
            .collect::<Result<Vec<Chapter<String>>, Error>>()
            .map(|x| Project {
                author: author,
                title: title,
                chapters: x
            })
    }
}
