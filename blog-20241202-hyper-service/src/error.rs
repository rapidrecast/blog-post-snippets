use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct MyError {
    pub payload: String,
}

impl MyError {}

impl Display for MyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Error: {:?}", self))
    }
}

impl std::error::Error for MyError {}
