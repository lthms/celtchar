use std::fs;
use std::fs::{create_dir, remove_dir_all, create_dir_all};
use std::path::{Path, PathBuf};
use std::env::set_current_dir;

use tera::{Tera, Context};
use serde_json::json;

use crate::error::{Raise, Error};
use crate::project::{Project, Chapter};

const EPUB_MIMETYPE: &'static str = "application/epub+zip";

pub trait EpubWriter {
    fn write_template(
        &mut self,
        dst : &PathBuf,
        tera : & Tera,
        template : &str,
        ctx : &Context,
    ) -> Result<(), Error> ;

    fn write_file(
        &mut self,
        dst : &PathBuf,
        src : &PathBuf,
    ) -> Result<(), Error> ;

    fn write_str(
        &mut self,
        dst : &PathBuf,
        input : &str
    ) -> Result<(), Error> ;
}

impl EpubWriter {
    fn create_mimetype(&mut self) -> Result<(), Error> {
        self.write_str(&PathBuf::from("mimetype"), EPUB_MIMETYPE)
    }

    fn create_container(&mut self, tera : &Tera) -> Result<(), Error> {
        self.write_template(
            &PathBuf::from("META-INF/container.xml"),
            tera,
            "container.xml",
            &Context::default(),
        )
    }

    fn create_chapters(&mut self, tera : &Tera, chapters : &Vec<Chapter<String>>) -> Result<(), Error> {
        chapters.iter().enumerate()
            .map(|(idx, c)| {
                let mut ctx = Context::new();
                ctx.insert("number", &(idx + 1));
                ctx.insert("chapter", &c);

                let path : String = format!("{}.xhtml", idx);

                self.write_template(
                    &PathBuf::from(format!("OEBPS/Text/{}", path)),
                    tera,
                    "chapter.xhtml",
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

    fn install_cover(&mut self, cover : &PathBuf) -> Result<String, Error> {
        let extension = cover.extension()
            .or_raise("cover lacks an extension")?
            .to_str()
            .or_raise("cover extension is not valid utf-8")?;

        let dst = PathBuf::from("OEBPS").join(format!("cover.{}", extension));

        self.write_file(&dst, cover)?;

        Ok(extension.into())
    }

    pub fn generate(&mut self, project : &Project<String>, assets : &PathBuf) -> Result<(), Error> {

        let tera = compile_templates!(template_dir(assets)?.as_str());

        self.create_mimetype()?;
        self.create_container(&tera)?;

        self.create_chapters(&tera, &project.chapters)?;

        self.write_template(
            &PathBuf::from("OEBPS/Style/main.css"),
            &tera,
            "main.css",
            &Context::new(),
        )?;

        let cover_extension = project.cover.clone().map(|cov| self.install_cover(&cov))
        // from Option<Result<_, E>> to Result<Option<_>, E>
            .map_or(Ok(None), |r| r.map(Some))?;

        let fonts = vec![
            "et-book-roman-line-figures.ttf",
            "et-book-bold-line-figures.ttf",
            "et-book-display-italic-old-style-figures.ttf",
        ];

        self.install_fonts(assets, &fonts)?;

        let files = project.chapters.iter().enumerate()
            .map(|(idx, _)| idx)
            .collect::<Vec<usize>>();

        let mut ctx = Context::new();
        ctx.insert("title", &project.title);
        ctx.insert("author", &project.author);
        ctx.insert("cover_extension", &cover_extension);
        ctx.insert("files", &files);
        ctx.insert("fonts", &fonts);

        self.write_template(
            &PathBuf::from("OEBPS/content.opf"),
            &tera,
            "content.opf",
            &ctx,
        )?;

        let chaps: Vec<_> = project.chapters.iter().enumerate()
            .map(|(idx, chapter)| json!({
                "index": idx,
                "title": chapter.title,
            }))
            .collect();

        let mut ctx = Context::new();
        ctx.insert("chapters", &chaps);
        self.write_template(
            &PathBuf::from("OEBPS/toc.ncx"),
            &tera,
            "toc.ncx",
            &ctx,
        )?;

        Ok(())
    }
}

fn template_dir(assets : &PathBuf) -> Result<String, Error> {
    let mut res = assets.clone();

    res.push("templates");
    res.push("**");
    res.push("*");

    res.to_str().map(String::from).ok_or(Error(format!("Compute template dir")))
}

fn fonts_dir(assets : &PathBuf) -> Result<PathBuf, Error> {
    let mut res = assets.clone();

    res.push("fonts");

    Ok(res)
}

pub struct Fs;

const BUILD_DIR : &'static str = "_build";

impl Fs {
    pub fn init() -> Result<Fs, Error> {
        remove_dir_all(BUILD_DIR).or_raise("cannot clean up _build/")?;
        create_dir(BUILD_DIR).or_raise("cannot create _build/")?;
        set_current_dir(BUILD_DIR).or_raise("cannot set current directory to _build/")?;

        Ok(Fs)
    }

    fn create_parent(&mut self, dst : &PathBuf) -> Result<(), Error> {
        let directory : &Path = dst.parent().ok_or(Error(String::from("is not a file")))?;

        if !directory.exists() {
            create_dir_all(directory)
                .or_raise(&format!("cannot create directory {:?}", directory))?;
        }

        Ok(())
    }
}

impl EpubWriter for Fs {
    fn write_template(
        &mut self,
        dst : &PathBuf,
        tera : & Tera,
        template : &str,
        ctx : &Context,
    ) -> Result<(), Error> {
        self.create_parent(dst)?;

        let content = tera.render(template, ctx)
            .or_raise(&format!("cannot render {}", template))?;

        fs::write(dst, content).or_raise(&format!("cannot create {:?}", dst))?;

        Ok(())
    }

    fn write_str(
        &mut self,
        dst : &PathBuf,
        input : &str
    ) -> Result<(), Error> {
        self.create_parent(dst)?;

        fs::write(dst, input).or_raise(&format!("cannot create {:?}", dst))
    }

    fn write_file(
        &mut self,
        dst : &PathBuf,
        src : &PathBuf,
    ) -> Result<(), Error> {
        self.create_parent(dst)?;

        fs::copy(src, dst).or_raise(&format!("cannot copy {:?} to {:?}", src, dst))?;

        Ok(())
    }
}
