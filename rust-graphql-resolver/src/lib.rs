pub mod error;
pub mod schema;
pub mod value;
pub use gurkle_parser as parser;

use schema::{resolve::QLContext, Schema};
use value::DataValue;

use crate::error::{Error, Result};

pub fn execute(context: QLContext, graphql_request: &str, schema: &Schema) -> Result<DataValue> {
    match gurkle_parser::parse_query(graphql_request) {
        Ok(doc) => schema.execute_document(context, doc),
        Err(err) => Err(Error::ParseError(format!("{:?}", err))),
    }
}
