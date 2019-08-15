use std::io::{Read, Write};
use std::path::PathBuf;

use serde_json::json;

use tera::{Tera, Context};

use crate::error::{Raise, Error};
use crate::project::{Project, Chapter, Cover};

use zip::write::FileOptions;
use zip::ZipWriter;
use std::fs::File;
use std::collections::HashSet;

const EPUB_MIMETYPE: &'static str = "application/epub+zip";

pub trait EpubWriter {
    fn write_template(
        &mut self,
        dst : &PathBuf,
        tera : & Tera,
        template : &str,
        ctx : &Context,
    ) -> Result<(), Error> {
        let content = tera.render(template, ctx)
            .or_raise(&format!("cannot render {}", template))?;

        self.write_bytes(dst, content.as_bytes())
    }

    fn write_file(
        &mut self,
        dst : &PathBuf,
        src : &PathBuf,
    ) -> Result<(), Error>;

    fn write_bytes(
        &mut self,
        dst : &PathBuf,
        input : &[u8]
    ) -> Result<(), Error> ;

    fn create_mimetype(&mut self) -> Result<(), Error> {
        self.write_bytes(&PathBuf::from("mimetype"), EPUB_MIMETYPE.as_bytes())
    }

    fn create_container(&mut self, tera : &Tera) -> Result<(), Error> {
        self.write_template(
            &PathBuf::from("META-INF/container.xml"),
            tera,
            "container.xml",
            &Context::default(),
        )
    }

    fn create_chapters(
        &mut self,
        tera : &Tera,
        chapters : &Vec<Chapter<String>>,
        numbering : bool
    ) -> Result<(), Error> {
        chapters.iter().enumerate()
            .map(|(idx, c)| {
                let mut ctx = Context::new();
                ctx.insert("number", &(idx + 1));
                ctx.insert("chapter", &c);
                ctx.insert("numbering", &numbering);

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

    fn install_cover(&mut self, cover : &Cover) -> Result<(), Error> {
        let dst = PathBuf::from("OEBPS").join(format!("cover.{}", cover.extension));
        self.write_bytes(&dst, cover.content.as_slice())
    }

    fn generate(&mut self, project : &Project<Cover, String>, assets : &PathBuf) -> Result<(), Error> {

        let tera = compile_templates!(template_dir(assets)?.as_str());

        self.create_mimetype()?;
        self.create_container(&tera)?;

        self.create_chapters(&tera, &project.chapters, project.numbering.unwrap_or(false))?;

        self.write_template(
            &PathBuf::from("OEBPS/Style/main.css"),
            &tera,
            "main.css",
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

        let files = project.chapters.iter().enumerate()
            .map(|(idx, _)| idx)
            .collect::<Vec<usize>>();

        let mut ctx = Context::new();
        ctx.insert("title", &project.title);
        ctx.insert("author", &project.author);
        ctx.insert("cover_extension", &project.cover.as_ref().map(|x| x.extension.clone()));
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
            if  self.dirs.contains(dir) {
                self.output.add_directory_from_path(dir, FileOptions::default())
                    .or_raise(&format!("Could not create directory {:?}", dir))?;
                self.dirs.insert(dir.to_path_buf());
            }
        }

        Ok(())
    }
}

impl EpubWriter for Zip {
    fn write_bytes(
        &mut self,
        dst : &PathBuf,
        input : &[u8]
    ) -> Result<(), Error> {
        self.create_parent(dst)?;

        self.output.start_file_from_path(dst, FileOptions::default())
            .or_raise(&format!("Could not add file {:?} to archive", dst))?;

        self.output.write_all(input)
            .or_raise(&format!("Could not write {:?} content", dst))?;

        Ok(())
    }

    fn write_file(
        &mut self,
        dst : &PathBuf,
        src : &PathBuf,
    ) -> Result<(), Error> {
        let mut buffer = Vec::new();
        let mut f = File::open(src).or_raise(&format!("Could not open {:?}", src))?;
        f.read_to_end(&mut buffer).or_raise(&format!("Could not read {:?} content", src))?;

        self.create_parent(dst)?;

        self.output.start_file_from_path(dst, FileOptions::default())
            .or_raise(&format!("Could not add file {:?} to archive", dst))?;

        self.output.write_all(buffer.as_ref())
            .or_raise(&format!("Could not write {:?} content", dst))?;

        Ok(())
    }
}
