pub mod builder;
pub mod error;
pub mod schema;
pub mod value;
pub use gurkle_parser as ast;
pub use macros;
pub use maplit;

use schema::{resolve::QLContext, Schema};
use value::DataValue;

use crate::error::{Error, Result};

// pub fn execute(
//     context: QLContext,
//     graphql_request: &str,
//     schema: &Schema,
//     operation_name: Option<String>,
// ) -> Result<DataValue> {
//     match gurkle_parser::parse_query(graphql_request) {
//         Ok(doc) => schema.execute_document(context, doc, operation_name),
//         Err(err) => Err(Error::ParseError(format!("{:?}", err))),
//     }
// }
