use std::{collections::HashMap, fmt::Debug};

use crate::{error::Result, value::DataValue};

use super::{
    field::{ArgumentMap, FieldType},
    resolve::{ApiResolveFunc, ArgumentValueMap, QLApiParam, QLContext},
};

use gurkle_parser::query as ast;

/// QueryMap
pub type QueryMap = HashMap<String, Query>;

/// Query
#[derive(Clone)]
pub struct Query {
    pub field_type: FieldType,
    pub arguments: ArgumentMap,
    pub description: String,
    pub resolve: Box<dyn ApiResolveFunc>,
}

impl Query {
    pub(crate) fn execute<'a, 'b>(
        &self,
        context: &'a mut QLContext,
        field: ast::Field,
    ) -> Result<DataValue> {
        let parameter = QLApiParam {
            arguments: ArgumentValueMap::from(field.arguments),
            selection_sets: field.selection_set.items,
        };
        let resolve_result = self.resolve.call(context, &parameter)?.to_data_value();
        self.field_type.execute(context, &parameter, resolve_result)
    }
}

impl Debug for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Query{{field_type: {:?}, description: {}, resolve: <ApiResolveFunc>}}",
            self.field_type, self.description
        )
    }
}
