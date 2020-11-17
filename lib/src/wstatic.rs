use serde_json::json;
use std::fs::{create_dir, read_to_string};
use std::path::PathBuf;
use tera::{Context, Tera};

use crate::assets::template_dir;
use crate::error::{Error, Raise};
use crate::project::{Chapter, Cover, Language, Project};
use crate::BookWriter;

pub struct Static {
    base : PathBuf,
    body_only : bool,
}

impl BookWriter for Static {
    fn write_bytes(&mut self, dst : &PathBuf, input : &[u8]) -> Result<(), Error> {
        std::fs::write(&self.base.join(dst), input)
            .or_raise(&format!("Could not write content to file {:?}", dst))?;

        Ok(())
    }

    fn write_file(&mut self, dst : &PathBuf, src : &PathBuf) -> Result<(), Error> {
        let input =
            read_to_string(src).or_raise(&format!("Could not read content of file {:?}", src))?;

        self.write_bytes(dst, input.as_bytes())?;

        Ok(())
    }
}

impl Static {
    pub fn init(base : &PathBuf, body_only : bool) -> Result<Static, Error> {
        if !base.exists() {
            create_dir(base).or_raise("Could not create output directory.")?;
        }

        if base.is_dir() {
            Ok(Static {
                base : base.to_owned(),
                body_only : body_only,
            })
        } else {
            Err(Error::new(&format!(
                "{:?} already exists and is not a directory",
                base
            )))
        }
    }

    fn generate_index(
        &mut self,
        project : &Project<Cover, String>,
        tera : &Tera,
    ) -> Result<(), Error> {
        let chaps : Vec<_> = project
            .chapters
            .iter()
            .enumerate()
            .map(|(idx, chapter)| {
                json!({
                    "index": idx,
                    "title": chapter.title,
                })
            })
            .collect();

        let mut ctx = Context::new();
        ctx.insert("chapters", &chaps);
        ctx.insert("language", &project.language);
        ctx.insert("title", &project.title);
        ctx.insert("body_only", &self.body_only);
        ctx.insert("description", &project.description);

        self.write_template(
            &PathBuf::from("index.html"),
            tera,
            "static/index.html",
            &ctx,
        )?;

        Ok(())
    }

    fn generate_chapters(
        &mut self,
        tera : &Tera,
        chapters : &Vec<Chapter<String>>,
        numbering : bool,
        lang : &Language,
    ) -> Result<(), Error> {
        let max = chapters.len();

        chapters
            .iter()
            .enumerate()
            .map(|(idx, c)| {
                let mut ctx = Context::new();
                ctx.insert("number", &(idx + 1));
                ctx.insert("chapter", c);
                ctx.insert("numbering", &numbering);
                ctx.insert("language", lang);
                ctx.insert("body_only", &self.body_only);
                ctx.insert("chapters_number", &max);

                let path : PathBuf = PathBuf::from(format!("{}.html", idx));

                self.write_template(&path, tera, "static/chapter.html", &ctx)?;

                Ok(())
            })
            .collect::<Result<Vec<()>, Error>>()?;

        Ok(())
    }

    pub fn generate_static_website(
        &mut self,
        project : &Project<Cover, String>,
        assets : &PathBuf,
    ) -> Result<(), Error> {
        let tera =
            Tera::new(template_dir(assets)?.as_str()).or_raise("Could not build templates")?;

        self.generate_index(project, &tera)?;
        self.generate_chapters(
            &tera,
            &project.chapters,
            project.numbering.unwrap_or(false),
            &project.language,
        )?;

        if !self.body_only {
            self.write_file(
                &PathBuf::from("style.css"),
                &assets.join(PathBuf::from("templates/static/style.css")),
            )?;
        }

        Ok(())
    }
}
