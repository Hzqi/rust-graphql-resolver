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
    #[error("Mutation schema not defined")]
    MutationSchemaNotDefined,

    #[error("Must provide operation name if query contains multiple operations")]
    MultipleOperationNeedTarget,
    #[error("There can only be one operation named {0}")]
    OnlyOneOperationCanNamed(String),
    #[error("This anonymous operation must be the only defined operation")]
    MustBeDefinedAnonymousOperation,

    #[error("No such Fragment {0}")]
    NoSuchFragment(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum BuildError {
    #[error("No such object type {0}, set it first")]
    NoSuchObjectType(String),
    #[error("No such enum type {0}, set it first")]
    NoSuchEnumType(String),
    #[error("No such input type {0}, set it first")]
    NoSuchInputType(String),
}

pub type BuildResult<T> = std::result::Result<T, BuildError>;
