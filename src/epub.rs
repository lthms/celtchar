use crate::project::{Error, Project, Chapter};
use std::fs::{create_dir, create_dir_all};
use std::path::PathBuf;

const EPUB_MIMETYPE: &'static str = "application/epub+zip";
const CONTAINER_XML: &'static str = r#"<?xml version="1.0"?>
<container xmlns="urn:oasis:names:tc:opendocument:xmlns:container" version="1.0">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#;

fn create_mimetype() -> Result<(), Error> {
    std::fs::write("mimetype", EPUB_MIMETYPE)
        .map_err(|_| Error(String::from("cannot create mimetype")))?;

    Ok(())
}

fn create_container() -> Result<(), Error> {
    create_dir("META-INF")
        .map_err(|_| Error(String::from("cannot create META-INF")))?;

    std::fs::write("META-INF/container.xml", CONTAINER_XML)
        .map_err(|_| Error(String::from("cannot create container.xml")))?;

    Ok(())
}

fn create_chapters(chapters : &Vec<Chapter<String>>) -> Result<Vec<String>, Error> {

    chapters.iter().enumerate()
        .map(|(idx, c)| {
            let path : String = format!("{}.xhtml", idx);
            std::fs::write(
                PathBuf::from(format!("OEBPS/Text/{}", path)).as_path(),
                format!(
                    "<html><head></head><body>{}</body></html>",
                    c.content.as_str()
                )
            )
                .map_err(|_| Error(format!("cannot create {:?}", path)))?;

            Ok(path)
        })
        .collect::<Result<Vec<String>, Error>>()
}

pub fn generate(project : &Project<String>) -> Result<(), Error> {
    create_mimetype()?;
    create_container()?;

    create_dir_all("OEBPS/Text")
        .map_err(|_| Error(String::from("cannot create OEBPS")))?;

    let files = create_chapters(&project.chapters)?;

    let mut filesxml = String::from("");
    for f in files.iter() {
        filesxml.push_str(
            format!(
                r#"<item href="Text/{}" id="{}" media-type="application/xhtml+xml" />"#,
                f, f
            ).as_str()
        );
    }

    let mut spine = String::from("");
    for f in files.iter() {
        spine.push_str(
            format!(
                r#"<itemref idref="{}" />"#,
                f
            ).as_str()
        );
    }

    std::fs::write(
        "OEBPS/content.opf",
        format!(
            r#"<?xml version="1.0" encoding="utf-8" standalone="yes"?>
<package xmlns="http://www.idpf.org/2007/opf" unique-identifier="BookId" version="2.0" xmlns:opf="http://www.idpf.org/2007/opf">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>{}</dc:title>
    <dc:language>fr</dc:language>
    <dc:creator opf:role="aut">{}</dc:creator>
    <dc:type>text</dc:type>
    <dc:description>Ceci est une description</dc:description>
  </metadata>
  <manifest>
    <item href="toc.ncx" id="ncx" media-type="application/x-dtbncx+xml" />
    {}
  </manifest>
  <spine toc="ncxtoc">
    {}
  </spine>
</package>
"#,
            project.title,
            project.author,
            filesxml,
            spine
        )
    )
        .map_err(|_| Error(String::from("cannot create content.opf")))?;

    Ok(())
}
