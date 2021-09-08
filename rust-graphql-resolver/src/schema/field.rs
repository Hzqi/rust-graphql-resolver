use std::{collections::HashMap, fmt::Debug};

use gurkle_parser::query as ast;

use crate::{error::Result, value::DataValue};

use super::{
    executor::{FieldExecutor, TypeExecutor},
    resolve::{ArgumentValueMap, FieldResolveFunc, QLApiParam, QLContext},
    types::{FieldType, InputType},
    Schema,
};

/// Field
#[derive(Clone)]
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
    pub description: String,
    pub resolve: Box<dyn FieldResolveFunc>,
}

impl FieldExecutor for Field {
    fn execute<'schema, 'a, 'b>(
        &self,
        schema: &'schema Schema,
        context: &'a mut QLContext,
        fragments: &'b HashMap<String, ast::FragmentDefinition>,
        field: &'b ast::Field,
        source: &'b DataValue,
    ) -> Result<DataValue> {
        let parameter = QLApiParam {
            arguments: ArgumentValueMap::from(field.arguments.to_owned()),
            selection_sets: field.selection_set.items.clone(),
        };
        let resolve_result = self.resolve.call(context, source, &parameter)?;
        self.field_type
            .execute(schema, context, fragments, &parameter, resolve_result)
    }
}

impl Debug for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Field{{name: {}, field_type: {:?}, description: {}, resolve: <FieldResolveFunc>}}",
            self.name, self.field_type, self.description
        )
    }
}

/// InputField
#[derive(Clone, Debug)]
pub struct InputField {
    pub name: String,
    pub field_type: InputType,
    pub description: String,
}
