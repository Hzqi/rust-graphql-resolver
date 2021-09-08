use std::{collections::HashMap, fmt::Debug};

use crate::{error::Result, value::DataValue};

use super::{
    executor::RootExecutor,
    resolve::ApiResolveFunc,
    types::{ArgumentMap, FieldType},
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

// impl RootExecutor for RootField {
//     fn execute<'schema, 'a, 'b>(
//         &self,
//         schema: &'schema super::Schema,
//         context: &'a mut super::resolve::QLContext,
//         fragments: &'b HashMap<String, gurkle_parser::query::FragmentDefinition>,
//         field: gurkle_parser::query::Field,
//     ) -> Result<DataValue> {

//     }
// }

// TODO: execute_introspection()
