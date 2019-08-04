use crate::project::{Error, Project, Chapter};
use std::fs::{create_dir_all};
use std::path::{Path, PathBuf};
use tera::{Tera, Context};

const EPUB_MIMETYPE: &'static str = "application/epub+zip";

fn write_template_to(tera : &Tera, template : &str, ctx : &Context, path : &PathBuf) -> Result<(), Error> {
    let directory : &Path = path.parent().ok_or(Error(String::from("is not a file")))?;

    if !directory.exists() {
        create_dir_all(directory)
            .map_err(|_| Error(format!("cannot create directory {:?}", directory)))?;
    }

    let content = tera.render(template, ctx)
        .map_err(|e| Error(format!("cannot render {}: {}", template, e)))?;

    std::fs::write(path, content)
        .map_err(|_| Error(format!("cannot create {:?}", path)))?;

    Ok(())
}

fn create_mimetype() -> Result<(), Error> {
    std::fs::write("mimetype", EPUB_MIMETYPE)
        .map_err(|_| Error(String::from("cannot create mimetype")))?;

    Ok(())
}

fn create_container(tera : &Tera) -> Result<(), Error> {
    write_template_to(
        tera,
        "container.xml",
        &Context::default(),
        &PathBuf::from("META-INF/container.xml")
    )?;


    Ok(())
}

fn create_chapters(tera : &Tera, chapters : &Vec<Chapter<String>>) -> Result<Vec<String>, Error> {

    chapters.iter().enumerate()
        .map(|(idx, c)| {
            let mut ctx = Context::new();
            ctx.insert("chapter", &c);

            let path : String = format!("{}.xhtml", idx);

            write_template_to(
                tera,
                "chapter.xhtml",
                &ctx,
                &PathBuf::from(format!("OEBPS/Text/{}", path))
            )?;

            Ok(path)
        })
        .collect::<Result<Vec<String>, Error>>()
}

pub fn generate(project : &Project<String>) -> Result<(), Error> {

    let tera = compile_templates!("../templates/**/*");

    create_mimetype()?;
    create_container(&tera)?;

    let files = create_chapters(&tera, &project.chapters)?;

    let mut ctx = Context::new();
    ctx.insert("title", &project.title);
    ctx.insert("author", &project.author);
    ctx.insert("files", &files);

    write_template_to(
        &tera,
        "content.opf",
        &ctx,
        &PathBuf::from("OEBPS/content.opf")
    )?;

    write_template_to(
        &tera,
        "main.css",
        &Context::new(),
        &PathBuf::from("OEBPS/Style/main.css")
    )?;

    Ok(())
}
