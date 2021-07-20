#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("parse graphql request error: {0}")]
    ParseError(String),
    #[error("Unsupported yet error: {0}")]
    UnSupportedYetError(String),
    #[error("This is default resulover, should not be invoked")]
    DefaultResolveError,
    #[error("NotFound: {0}")]
    NotFoundError(String),
    #[error("DataTypeMisMatchError expect: {0}, actul: {1}")]
    DataTypeMisMatchError(String, String),
    #[error("Missing reference custom type")]
    MissingReferenceCustomTypeError,
}

pub type Result<T> = std::result::Result<T, Error>;
