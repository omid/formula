use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Parser Error: Invalid `{0}` expression")]
    Parser(&'static str),
}
