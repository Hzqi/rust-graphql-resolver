use std::{collections::HashMap, fmt::Debug};

use crate::{error::Result, value::DataValue};

use super::{
    field::{ArgumentMap, FieldType},
    resolve::{ApiResolveFunc, ArgumentValueMap, QLApiParam, QLContext},
};

use gurkle_parser::query as parser;

/// MutationMap
pub type MutationMap = HashMap<String, Mutation>;

/// Mutation
#[derive(Clone)]
pub struct Mutation {
    pub field_type: FieldType,
    pub arguments: ArgumentMap,
    pub description: String,
    pub resolve: Box<dyn ApiResolveFunc>,
}

impl Mutation {
    pub(crate) fn execute(&self, context: QLContext, field: parser::Field) -> Result<DataValue> {
        let parameter = QLApiParam {
            arguments: ArgumentValueMap::from(field.arguments),
            selection_sets: field.selection_set.items,
        };
        let resolve_result = self.resolve.call(context.clone(), parameter.clone())?;
        self.field_type.execute(context, parameter, resolve_result)
    }
}

impl Debug for Mutation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Mutation{{field_type: {:?}, description: {}, resolve: <ApiResolveFunc>}}",
            self.field_type, self.description
        )
    }
}