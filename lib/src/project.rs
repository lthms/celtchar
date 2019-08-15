use ogmarkup::typography::Typography;
use serde_derive::{Deserialize, Serialize};

use crate::render::Html;
use crate::error::{Raise, Error};

#[derive(Debug, Serialize, Deserialize)]
pub struct Cover {
    pub extension : String,
    pub content : Vec<u8>,
}

pub trait Loader {
    type CovId;
    type DocId;
    type ProjId;

    fn load_cover(
        &self,
        id : &Self::CovId
    ) -> Result<Cover, Error>;

    fn load_document(
        &self,
        id : &Self::DocId
    ) -> Result<String, Error>;

    fn load_project(
        &self,
        id : &Self::ProjId
    ) -> Result<Project<Self::CovId, Vec<Self::DocId>>, Error>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chapter<I> {
    pub title : Option<String>,
    pub content : I,
}

impl<I> Chapter<Vec<I>> {
    fn load_and_render<T, L>(
        &self,
        loader : &L,
        typo : &T,
    ) -> Result<Chapter<String>, Error>
    where
        T : Typography,
        L : Loader<DocId = I>
    {
        let title = &self.title;
        let content = &self.content;

        let doc = content.iter()
            .map(|ref x| {
                let input = loader.load_document(x)?;
                ogmarkup::compile(&input, typo)
                    .or_raise("Cannot parse an ogmarkup document for some reason")
                    .map(Html::to_string)
            })
            .collect::<Result<Vec<String>, Error>>()?
            .join("");

        Ok(Chapter {
            title: title.clone(),
            content: doc,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Project<C, I> {
    pub author: String,
    pub title: String,
    pub chapters: Vec<Chapter<I>>,
    pub cover: Option<C>,
    pub numbering: Option<bool>,
}

impl<C, I> Project<C, Vec<I>> {
    pub fn load_and_render<'input, T, L> (
        self,
        loader : &L,
        typo : &T,
    ) -> Result<Project<Cover, String>, Error>
    where
        T : Typography,
        L : Loader<CovId = C, DocId = I>,
    {
        let numbering = self.numbering;
        let author = self.author;
        let title = self.title;
        let cover = self.cover
            .map(|x| loader.load_cover(&x).or_raise("cannot load the cover"))
            .map_or(Ok(None), |r| r.map(Some))?;

        self.chapters.into_iter()
            .map(|chapter| chapter.load_and_render(loader, typo))
            .collect::<Result<Vec<Chapter<String>>, Error>>()
            .map(|x| Project {
                author: author,
                title: title,
                chapters: x,
                cover: cover,
                numbering: numbering,
            })
    }
}
