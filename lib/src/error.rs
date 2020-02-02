#[derive(Debug)]
pub struct Error(pub String);

impl Error {
    pub fn new(str : &str) -> Error {
        Error(String::from(str))
    }
}

pub trait Raise {
    type Out;

    fn or_raise(self, msg : &str) -> Self::Out;
}

impl<T> Raise for Option<T> {
    type Out = Result<T, Error>;

    fn or_raise(self, msg : &str) -> Result<T, Error> {
        self.ok_or(Error(String::from(msg)))
    }
}

impl<T, E> Raise for Result<T, E> {
    type Out = Result<T, Error>;

    fn or_raise(self, msg : &str) -> Result<T, Error> {
        self.map_err(|_| Error(String::from(msg)))
    }
}
