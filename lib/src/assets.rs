use crate::error::{Error, Raise};
use std::path::PathBuf;

pub fn template_dir(assets : &PathBuf) -> Result<String, Error> {
    let mut res = assets.clone();

    res.push("templates");
    res.push("**");
    res.push("*");

    res.to_str()
        .map(String::from)
        .or_raise("Compute template dir")
}

pub fn fonts_dir(assets : &PathBuf) -> Result<PathBuf, Error> {
    let mut res = assets.clone();

    res.push("fonts");

    Ok(res)
}
