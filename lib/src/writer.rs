use std::path::PathBuf;
use tera::{Context, Tera};

use crate::error::Error;

pub trait BookWriter {
    fn write_file(&mut self, dst: &PathBuf, src: &PathBuf) -> Result<(), Error>;

    fn write_bytes(&mut self, dst: &PathBuf, input: &[u8]) -> Result<(), Error>;

    fn write_template(
        &mut self,
        dst: &PathBuf,
        tera: &Tera,
        template: &str,
        ctx: &Context,
    ) -> Result<(), Error> {
        let content = tera
            .render(template, ctx)
            .map_err(|err| Error::new(&format!("{:?}", err)))?;

        self.write_bytes(dst, content.as_bytes())
    }
}
