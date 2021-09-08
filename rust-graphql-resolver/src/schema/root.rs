use std::{collections::HashMap, fmt::Debug};

use gurkle_parser::query as ast;

use crate::{error::Result, value::DataValue};

use super::{
    executor::{RootExecutor, TypeExecutor},
    resolve::{ApiResolveFunc, ArgumentValueMap, QLApiParam, QLContext},
    types::{ArgumentMap, FieldType},
    Schema,
};

/// QueryMap
pub type QueryMap = HashMap<String, RootField>;

/// MutationMap
pub type MutationMap = HashMap<String, RootField>;

#[derive(Clone)]
pub struct RootField {
    pub(crate) field_type: FieldType,
    pub(crate) arguments: ArgumentMap,
    pub(crate) description: String,
    pub(crate) resolve: Box<dyn ApiResolveFunc>,
}

impl Debug for RootField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RootField")
            .field("field_type", &self.field_type)
            .field("arguments", &self.arguments)
            .field("description", &self.description)
            .field("resolve", &"ApiResolveFunc(...)")
            .finish()
    }
}

impl RootExecutor for RootField {
    fn execute<'schema, 'a, 'b>(
        &self,
        schema: &'schema Schema,
        context: &'a mut QLContext,
        fragments: &'b HashMap<String, ast::FragmentDefinition>,
        field: ast::Field,
    ) -> Result<DataValue> {
        let parameter = QLApiParam {
            arguments: ArgumentValueMap::from(field.arguments),
            selection_sets: field.selection_set.items,
        };
        let resolve_result = self.resolve.call(context, &parameter)?;
        self.field_type
            .execute(schema, context, fragments, &parameter, resolve_result)
    }
}

// TODO: execute_introspection()
