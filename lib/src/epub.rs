use std::io::{Read, Write};
use std::path::PathBuf;

use serde_json::json;

use tera::{Context, Tera};

use crate::error::{Error, Raise};
use crate::project::{Chapter, Cover, Language, Project};

use std::collections::HashSet;
use std::fs::File;
use zip::write::FileOptions;
use zip::ZipWriter;

use crate::assets::{fonts_dir, template_dir};
use crate::render::Html;
use crate::writer::BookWriter;

const EPUB_MIMETYPE : &'static str = "application/epub+zip";

pub trait EpubWriter: BookWriter {
    fn create_mimetype(&mut self) -> Result<(), Error> {
        self.write_bytes(&PathBuf::from("mimetype"), EPUB_MIMETYPE.as_bytes())
    }

    fn create_container(&mut self, tera : &Tera) -> Result<(), Error> {
        self.write_template(
            &PathBuf::from("META-INF/container.xml"),
            tera,
            "epub/container.xml",
            &Context::default(),
        )
    }

    fn create_chapters(
        &mut self,
        tera : &Tera,
        chapters : &Vec<Chapter<Html>>,
        numbering : bool,
        lang : &Language,
    ) -> Result<(), Error> {
        chapters
            .iter()
            .enumerate()
            .map(|(idx, c)| {
                let mut ctx = Context::new();
                ctx.insert("number", &(idx + 1));
                ctx.insert("chapter", &c);
                ctx.insert("numbering", &numbering);
                ctx.insert("language", &lang);

                let path : String = format!("{}.xhtml", idx);

                self.write_template(
                    &PathBuf::from(format!("OEBPS/Text/{}", path)),
                    tera,
                    "epub/chapter.xhtml",
                    &ctx,
                )?;

                Ok(path)
            })
            .collect::<Result<Vec<String>, Error>>()?;

        Ok(())
    }

    fn install_fonts(&mut self, assets : &PathBuf, fonts : &Vec<&str>) -> Result<(), Error> {
        for f in fonts {
            let src = fonts_dir(assets)?.join(f);
            let dst = PathBuf::from("OEBPS/Fonts").join(f);

            self.write_file(&dst, &src)?;
        }

        Ok(())
    }

    fn install_cover(&mut self, cover : &Cover) -> Result<(), Error> {
        let dst = PathBuf::from("OEBPS").join(format!("cover.{}", cover.extension));
        self.write_bytes(&dst, cover.content.as_slice())
    }

    fn generate_epub(
        &mut self,
        project : &Project<Cover, Html>,
        assets : &PathBuf,
    ) -> Result<(), Error> {
        let tera =
            Tera::new(template_dir(assets)?.as_str()).or_raise("Could not build templates")?;

        self.create_mimetype()?;
        self.create_container(&tera)?;

        self.create_chapters(
            &tera,
            &project.chapters,
            project.numbering.unwrap_or(false),
            &project.language,
        )?;

        self.write_template(
            &PathBuf::from("OEBPS/Style/main.css"),
            &tera,
            "epub/main.css",
            &Context::new(),
        )?;

        if let Some(ref cov) = project.cover {
            self.install_cover(cov)?;
        }

        let fonts = vec![
            "et-book-roman-line-figures.ttf",
            "et-book-bold-line-figures.ttf",
            "et-book-display-italic-old-style-figures.ttf",
        ];

        self.install_fonts(assets, &fonts)?;

        let files = project
            .chapters
            .iter()
            .enumerate()
            .map(|(idx, _)| idx)
            .collect::<Vec<usize>>();

        let mut ctx = Context::new();
        ctx.insert("title", &project.title);
        ctx.insert("author", &project.author);
        ctx.insert(
            "cover_extension",
            &project.cover.as_ref().map(|x| x.extension.clone()),
        );
        ctx.insert("files", &files);
        ctx.insert("fonts", &fonts);
        ctx.insert("language", &project.language);

        self.write_template(
            &PathBuf::from("OEBPS/content.opf"),
            &tera,
            "epub/content.opf",
            &ctx,
        )?;

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
        self.write_template(&PathBuf::from("OEBPS/toc.ncx"), &tera, "epub/toc.ncx", &ctx)?;

        Ok(())
    }
}

impl<W> EpubWriter for W where W : BookWriter {}

pub struct Zip {
    output : ZipWriter<File>,
    dirs : HashSet<PathBuf>,
}

impl Zip {
    pub fn init() -> Result<Zip, Error> {
        let file = File::create("Book.epub").or_raise("Could not create Book.epub")?;

        Ok(Zip {
            output : ZipWriter::new(file),
            dirs : HashSet::new(),
        })
    }

    fn create_parent(&mut self, dst : &PathBuf) -> Result<(), Error> {
        if let Some(dir) = dst.parent() {
            if self.dirs.contains(dir) {
                if let Some(dir_str) = dir.to_str() {
                    self.output
                        .add_directory(dir_str, FileOptions::default())
                        .or_raise(&format!("Could not create directory {:?}", dir))?;
                    self.dirs.insert(dir.to_path_buf());
                }
            }
        }

        Ok(())
    }
}

impl BookWriter for Zip {
    fn write_bytes(&mut self, dst : &PathBuf, input : &[u8]) -> Result<(), Error> {
        self.create_parent(dst)?;

        if let Some(dst) = dst.to_str() {
            self.output
                .start_file(dst, FileOptions::default())
                .or_raise(&format!("Could not add file {:?} to archive", dst))?;

            self.output
                .write_all(input)
                .or_raise(&format!("Could not write {:?} content", dst))?;
        }

        Ok(())
    }

    fn write_file(&mut self, dst : &PathBuf, src : &PathBuf) -> Result<(), Error> {
        if let Some(dst_str) = dst.to_str() {
            let mut buffer = Vec::new();
            let mut f = File::open(src).or_raise(&format!("Could not open {:?}", src))?;
            f.read_to_end(&mut buffer)
                .or_raise(&format!("Could not read {:?} content", src))?;

            self.create_parent(dst)?;

            self.output
                .start_file(dst_str, FileOptions::default())
                .or_raise(&format!("Could not add file {:?} to archive", dst))?;

            self.output
                .write_all(buffer.as_ref())
                .or_raise(&format!("Could not write {:?} content", dst))?;
        }

        Ok(())
    }
}
