use ogam::generator::Output;
use ogam::typography::{Typography, ENGLISH, FRENCH};
use serde_derive::{Deserialize, Serialize};

use crate::error::{Error, Raise};

#[derive(Debug, Serialize, Deserialize)]
pub enum Language {
    Fr,
    En,
}

impl Language {
    pub fn typography(&self) -> &dyn Typography {
        match self {
            Language::Fr => &FRENCH,
            Language::En => &ENGLISH,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cover {
    pub extension: String,
    pub content: Vec<u8>,
}

pub trait Loader {
    type CovId;
    type DocId;
    type ProjId;

    fn load_cover(&self, id: &Self::CovId) -> Result<Cover, Error>;

    fn load_document(&self, id: &Self::DocId) -> Result<String, Error>;

    fn load_project(&self, id: &Self::ProjId) -> Result<Project<Self::CovId, Self::DocId>, Error>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chapter<I> {
    pub title: Option<String>,
    pub content: Vec<I>,
}

impl<I> Chapter<I> {
    fn load_and_render<T, L, O>(&self, loader: &L, typo: &T) -> Result<Chapter<O>, Error>
    where
        T: Typography + ?Sized,
        L: Loader<DocId = I>,
        O: Output,
    {
        let title = &self.title;
        let content = &self.content;

        let doc = content
            .iter()
            .map(|ref x| {
                let input = loader.load_document(x)?;
                ogam::compile(&input, typo)
                    .or_raise("Cannot parse an ogmarkup document for some reason")
            })
            .collect::<Result<Vec<O>, Error>>()?;

        Ok(Chapter {
            title: title.clone(),
            content: doc,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Part<I> {
    pub title: Option<String>,
    #[serde(rename = "chapters")]
    pub content: Vec<Chapter<I>>,
}

impl<I> Part<I> {
    fn load_and_render<T, L, O>(&self, loader: &L, typo: &T) -> Result<Part<O>, Error>
    where
        T: Typography + ?Sized,
        L: Loader<DocId = I>,
        O: Output,
    {
        let title = &self.title;
        let content = &self.content;

        let doc = content
            .iter()
            .map(|ref chap| chap.load_and_render(loader, typo))
            .collect::<Result<Vec<Chapter<O>>, Error>>()?;

        Ok(Part {
            title: title.clone(),
            content: doc,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Content<I> {
    #[serde(rename = "parts")]
    WithParts(Vec<Part<I>>),
    #[serde(rename = "chapters")]
    WithChapters(Vec<Chapter<I>>),
}

impl<I> Content<I> {
    fn load_and_render<T, L, O>(&self, loader: &L, typo: &T) -> Result<Content<O>, Error>
    where
        T: Typography + ?Sized,
        L: Loader<DocId = I>,
        O: Output,
    {
        match self {
            Content::WithParts(ref parts) => {
                let parts = parts
                    .iter()
                    .map(|ref part| part.load_and_render(loader, typo))
                    .collect::<Result<Vec<Part<O>>, Error>>()?;

                Ok(Content::WithParts(parts))
            }
            Content::WithChapters(ref chapters) => {
                let chapters = chapters
                    .iter()
                    .map(|ref chap| chap.load_and_render(loader, typo))
                    .collect::<Result<Vec<Chapter<O>>, Error>>()?;

                Ok(Content::WithChapters(chapters))
            }
        }
    }

    pub fn chapters(&self) -> Vec<&Chapter<I>> {
        match self {
            Content::WithChapters(ref chaps) => chaps.iter().collect(),
            Content::WithParts(ref parts) => {
                parts.iter().map(|p| p.content.iter()).flatten().collect()
            }
        }
    }

    pub fn mut_chapters(&mut self) -> Vec<&mut Chapter<I>> {
        match self {
            Content::WithChapters(ref mut chaps) => chaps.iter_mut().collect(),
            Content::WithParts(ref mut parts) => parts
                .iter_mut()
                .map(|p| p.content.iter_mut())
                .flatten()
                .collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Project<C, I> {
    pub author: String,
    pub title: String,
    pub description: Option<String>,
    pub cover: Option<C>,
    pub numbering: Option<bool>,
    pub language: Language,
    #[serde(flatten)]
    pub content: Content<I>,
}

impl<O> Project<Cover, O> {
    pub fn load_and_render<'input, L>(
        id: &L::ProjId,
        loader: &L,
    ) -> Result<Project<Cover, O>, Error>
    where
        L: Loader,
        O: Output,
    {
        let project = loader.load_project(id)?;

        let lang = project.language;
        let typo = lang.typography();
        let numbering = project.numbering;
        let descr = project.description;
        let author = project.author;
        let title = project.title;
        let cover = project
            .cover
            .map(|x| loader.load_cover(&x).or_raise("cannot load the cover"))
            .map_or(Ok(None), |r| r.map(Some))?;

        let content = project.content.load_and_render(loader, typo)?;

        Ok(Project {
            author,
            title,
            description: descr,
            content,
            cover,
            numbering,
            language: lang,
        })
    }
}
