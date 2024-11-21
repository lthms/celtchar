use serde_json::json;
use std::collections::HashMap;
use std::fs::{create_dir, read_to_string};
use std::path::PathBuf;
use tera::{Context, Error as TError, Result as TResult, Tera, Value};

use crate::assets::template_dir;
use crate::error::{Error, Raise};
use crate::project::{Chapter, Cover, Language, Project};
use crate::render::Html;
use crate::{BookWriter, Content, Part};

fn roman_filter(value: &Value, _kargs: &HashMap<String, Value>) -> TResult<Value> {
    let result = roman::to(
        value
            .as_i64()
            .map(|x| x as i32)
            .ok_or(TError::msg("Expected integer"))?,
    )
    .ok_or(TError::msg("Could not convert to roman notation"))?;

    Ok(Value::String(result))
}

pub struct Static {
    base: PathBuf,
    body_only: bool,
}

impl BookWriter for Static {
    fn write_bytes(&mut self, dst: &PathBuf, input: &[u8]) -> Result<(), Error> {
        std::fs::write(&self.base.join(dst), input)
            .or_raise(&format!("Could not write content to file {:?}", dst))?;

        Ok(())
    }

    fn write_file(&mut self, dst: &PathBuf, src: &PathBuf) -> Result<(), Error> {
        let input =
            read_to_string(src).or_raise(&format!("Could not read content of file {:?}", src))?;

        self.write_bytes(dst, input.as_bytes())?;

        Ok(())
    }
}

impl Static {
    pub fn init(base: &PathBuf, body_only: bool) -> Result<Static, Error> {
        if !base.exists() {
            create_dir(base).or_raise("Could not create output directory.")?;
        }

        if base.is_dir() {
            Ok(Static {
                base: base.to_owned(),
                body_only: body_only,
            })
        } else {
            Err(Error::new(&format!(
                "{:?} already exists and is not a directory",
                base
            )))
        }
    }

    fn generate_index(&mut self, project: &Project<Cover, Html>, tera: &Tera) -> Result<(), Error> {
        fn make_chaps(chapters: &Vec<Chapter<Html>>, idx_ofs: usize) -> Vec<serde_json::Value> {
            chapters
                .iter()
                .enumerate()
                .map(|(idx, chapter)| {
                    json!({
                        "index": idx + idx_ofs,
                        "title": chapter.title,
                    })
                })
                .collect()
        }

        let mut ctx = Context::new();

        match project.content {
            Content::WithParts(ref parts) => {
                let mut acc = vec![];
                let (_, _) = parts.iter().fold((0, 0), |(part_idx, chap_idx), p| {
                    let part_json = json!({
                        "index": part_idx,
                        "title": p.title,
                        "chapters": make_chaps(&p.content, chap_idx)
                    });
                    acc.push(part_json);

                    (part_idx + 1, chap_idx + p.content.len())
                });
                ctx.insert("parts", &acc);
            }
            Content::WithChapters(ref chaps) => {
                let chaps: Vec<_> = make_chaps(chaps, 0);
                ctx.insert("chapters", &chaps);
            }
        };

        ctx.insert("numbering", &project.numbering);
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
        tera: &Tera,
        chapters: &Vec<Chapter<Html>>,
        offset: usize,
        numbering: bool,
        lang: &Language,
        previous_part: Option<usize>,
        next_part: Option<usize>,
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
                ctx.insert("offset", &offset);
                ctx.insert("chapters_number", &max);
                ctx.insert("previous_part", &previous_part);
                ctx.insert("next_part", &next_part);

                let path: PathBuf = PathBuf::from(format!("{}.html", idx + offset));

                self.write_template(&path, tera, "static/chapter.html", &ctx)?;

                Ok(())
            })
            .collect::<Result<Vec<()>, Error>>()?;

        Ok(())
    }

    pub fn generate_parts(
        &mut self,
        tera: &Tera,
        parts: &Vec<Part<Html>>,
        numbering: bool,
        lang: &Language,
    ) -> Result<(), Error> {
        let mut ofs = 0;

        for (idx, part) in parts.into_iter().enumerate() {
            let next_part = if idx + 1 < parts.len() {
                Some(idx + 1)
            } else {
                None
            };

            let mut ctx = Context::new();
            ctx.insert("title", &part.title);
            ctx.insert("number", &(idx + 1));
            ctx.insert("numbering", &numbering);
            ctx.insert("language", lang);
            ctx.insert("body_only", &self.body_only);
            ctx.insert("chapters_number", &part.content.len());
            ctx.insert("parts_number", &parts.len());
            ctx.insert("offset", &ofs);

            let path: PathBuf = PathBuf::from(format!("p{}.html", idx));

            self.write_template(&path, tera, "static/part.html", &ctx)?;

            self.generate_chapters(
                tera,
                &part.content,
                ofs,
                numbering,
                lang,
                Some(idx),
                next_part,
            )?;

            ofs += part.content.len();
        }

        Ok(())
    }

    pub fn generate_content(
        &mut self,
        tera: &Tera,
        content: &Content<Html>,
        numbering: bool,
        lang: &Language,
    ) -> Result<(), Error> {
        match content {
            Content::WithParts(ref parts) => self.generate_parts(tera, parts, numbering, lang)?,
            Content::WithChapters(ref chapters) => {
                self.generate_chapters(tera, chapters, 0, numbering, lang, None, None)?
            }
        }
        Ok(())
    }

    pub fn generate_static_website(
        &mut self,
        project: &Project<Cover, Html>,
        assets: &PathBuf,
    ) -> Result<(), Error> {
        let mut tera =
            Tera::new(template_dir(assets)?.as_str()).or_raise("Could not build templates")?;

        tera.register_filter("roman", roman_filter);

        self.generate_index(project, &tera)?;

        self.generate_content(
            &tera,
            &project.content,
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
