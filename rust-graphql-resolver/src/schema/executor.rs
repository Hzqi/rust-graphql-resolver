use std::collections::HashMap;

use crate::{error::Result, value::DataValue};

use super::{
    resolve::{QLApiParam, QLContext},
    Schema,
};
use gurkle_parser::query as ast;

/// RootExecutor provide execute function for root apis
pub(crate) trait RootExecutor {
    fn execute<'schema, 'a, 'b>(
        &self,
        schema: &'schema Schema,
        context: &'a mut QLContext,
        fragments: &'b HashMap<String, ast::FragmentDefinition>,
        field: ast::Field,
    ) -> Result<DataValue>;
}

/// TypeExecutor provide execute function for field type
pub(crate) trait TypeExecutor {
    fn execute<'schema, 'a, 'b>(
        &self,
        schema: &'schema Schema,
        context: &'a mut QLContext,
        fragments: &'b HashMap<String, ast::FragmentDefinition>,
        parameter: &'b QLApiParam,
        data: DataValue,
    ) -> Result<DataValue>;
}

/// FieldExecutor provide
pub(crate) trait FieldExecutor {
    fn execute<'schema, 'a, 'b>(
        &self,
        schema: &'schema Schema,
        context: &'a mut QLContext,
        fragments: &'b HashMap<String, ast::FragmentDefinition>,
        field: &'b ast::Field,
        source: &'b DataValue,
    ) -> Result<DataValue>;
}
