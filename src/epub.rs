use crate::project::{Error, Project, Chapter};
use std::fs::{create_dir_all};
use std::path::{Path, PathBuf};
use tera::{Tera, Context};
use serde_json::json;

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
            ctx.insert("number", &(idx + 1));
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

fn install_fonts(assets : &PathBuf, fonts : &Vec<&str>) -> Result<(), Error> {
    create_dir_all("OEBPS/Fonts/")
        .map_err(|_| Error(String::from("cannot create directory OEBPS/Fonts/")))?;

    for f in fonts {
        let mut src = fonts_dir(assets)?;
        src.push(f);
        let mut dst = PathBuf::from("OEBPS/Fonts");
        dst.push(f);

        std::fs::copy(src, dst)
            .map_err(|_| Error(format!("cannot copy {}", f)))?;

    }

    Ok(())
}

fn install_cover(cover : &PathBuf) -> Result<String, Error> {
    let extension = cover.extension()
        .ok_or(Error(String::from("cover lacks an extension")))?
        .to_str()
        .ok_or(Error(String::from("cover extension is not valid utf-8")))?;

    let mut dst = PathBuf::from("OEBPS");
    dst.push(format!("cover.{}", extension));

    std::fs::copy(cover, dst)
        .map_err(|_| Error(format!("cannot copy {:?}", cover)))?;

    Ok(extension.into())
}

pub fn generate(project : &Project<String>, assets : &PathBuf) -> Result<(), Error> {

    let tera = compile_templates!(template_dir(assets)?.as_str());

    create_mimetype()?;
    create_container(&tera)?;

    let files = create_chapters(&tera, &project.chapters)?;

    write_template_to(
        &tera,
        "main.css",
        &Context::new(),
        &PathBuf::from("OEBPS/Style/main.css")
    )?;

    let cover_extension = project.cover.clone().map(|cov| install_cover(&cov))
    // from Option<Result<_, E>> to Result<Option<_>, E>
        .map_or(Ok(None), |r| r.map(Some))?;

    let fonts = vec![
        "et-book-roman-line-figures.ttf",
        "et-book-bold-line-figures.ttf",
        "et-book-display-italic-old-style-figures.ttf",
    ];

    install_fonts(assets, &fonts)?;

    let mut ctx = Context::new();
    ctx.insert("title", &project.title);
    ctx.insert("author", &project.author);
    ctx.insert("cover_extension", &cover_extension);
    ctx.insert("files", &files);
    ctx.insert("fonts", &fonts);
    write_template_to(
        &tera,
        "content.opf",
        &ctx,
        &PathBuf::from("OEBPS/content.opf")
    )?;

    let chaps: Vec<_> = project.chapters.iter().enumerate()
        .map(|(idx, chapter)| json!({
            "index": idx,
            "title": chapter.title,
        }))
        .collect();

    let mut ctx = Context::new();
    ctx.insert("chapters", &chaps);
    write_template_to(
        &tera,
        "toc.ncx",
        &ctx,
        &PathBuf::from("OEBPS/toc.ncx")
    )?;

    Ok(())
}
