use std::{
    array::IntoIter,
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    iter::FromIterator,
    rc::Rc,
};

use gurkle_parser::query::{Definition, Document, OperationDefinition, Selection};

use crate::{
    error::{Error, Result},
    value::DataValue,
};

use self::{
    field::{CustomType, QLEnum, QLInput},
    mutation::MutationMap,
    query::QueryMap,
    resolve::QLContext,
};

pub mod field;
pub mod mutation;
pub mod query;
pub mod resolve;

/// NotSupported
#[derive(Clone, Debug)]
pub struct NotSupported;

/// Schema: The main logic struct
/// * query: definition for query apis, and resolve by request
/// * mutation: todo
/// * subscrition: todo
#[derive(Clone, Debug)]
pub struct Schema {
    /// for tracting instance's id
    pub id: String,
    pub queries: QueryMap,
    pub mutations: Option<MutationMap>,
    /// TODO: Subscrition
    pub subscritions: Option<NotSupported>,

    /// storage of reference object types
    pub objects: HashMap<String, Rc<RefCell<CustomType>>>,
    /// storage of reference enum types
    pub enums: HashMap<String, Rc<QLEnum>>,
    /// storage of reference input object types
    pub inputs: HashMap<String, Rc<RefCell<QLInput>>>,
}

impl Schema {
    pub(crate) fn execute_document(
        &self,
        context: QLContext,
        mut doc: Document,
    ) -> Result<DataValue> {
        if doc.definitions.is_empty() {
            return Ok(DataValue::Null);
        }
        let def = doc.definitions.remove(0);
        match def {
            Definition::Operation(op) => {
                let data = self.execute_operate(context, op)?;
                Ok(DataValue::Object(BTreeMap::from_iter(IntoIter::new([(
                    "data".to_string(),
                    data,
                )]))))
            }
            Definition::Fragment(_) => {
                return Err(Error::UnSupportedYetError(
                    "'Fragment' in schema request".to_string(),
                ))
            }
        }
    }

    pub(crate) fn execute_operate(
        &self,
        context: QLContext,
        op: OperationDefinition,
    ) -> Result<DataValue> {
        match op {
            OperationDefinition::SelectionSet(sets) => {
                let mut result = BTreeMap::<String, DataValue>::new();
                for set in sets.items {
                    match set {
                        Selection::Field(field) => {
                            let name = field.name.clone();
                            let insert_key = if let Some(alias) = field.alias.clone() {
                                alias
                            } else {
                                name.clone()
                            };
                            let query_result = self
                                .queries
                                .get(&name)
                                .ok_or(Error::NotFoundError(format!("Query api {}", &name)))?
                                .execute(context.clone(), field)?;
                            result.insert(insert_key, query_result);
                        }
                        Selection::FragmentSpread(_) => {
                            return Err(Error::UnSupportedYetError(
                                "'FragmentSpread' in query(selection sets)".to_string(),
                            ))
                        }
                        Selection::InlineFragment(_) => {
                            return Err(Error::UnSupportedYetError(
                                "'InlineFragment' in query(selection sets)".to_string(),
                            ))
                        }
                    }
                }
                return Ok(DataValue::Object(result));
            }
            OperationDefinition::Query(queries) => {
                let mut result = BTreeMap::<String, DataValue>::new();
                for set in queries.selection_set.items {
                    match set {
                        Selection::Field(field) => {
                            let name = field.name.clone();
                            let insert_key = if let Some(alias) = field.alias.clone() {
                                alias
                            } else {
                                name.clone()
                            };
                            let query_result = self
                                .queries
                                .get(&name)
                                .ok_or(Error::NotFoundError(format!("Query api {}", &name)))?
                                .execute(context.clone(), field)?;
                            result.insert(insert_key, query_result);
                        }
                        Selection::FragmentSpread(_) => {
                            return Err(Error::UnSupportedYetError(
                                "'FragmentSpread' in query(selection sets)".to_string(),
                            ))
                        }
                        Selection::InlineFragment(_) => {
                            return Err(Error::UnSupportedYetError(
                                "'InlineFragment' in query(selection sets)".to_string(),
                            ))
                        }
                    }
                }
                return Ok(DataValue::Object(result));
            }
            OperationDefinition::Mutation(mutations) => {
                let mut result = BTreeMap::<String, DataValue>::new();
                for set in mutations.selection_set.items {
                    match set {
                        Selection::Field(field) => {
                            let name = field.name.clone();
                            let insert_key = if let Some(alias) = field.alias.clone() {
                                alias
                            } else {
                                name.clone()
                            };
                            let mutation_result = self
                                .mutations
                                .as_ref()
                                .ok_or(Error::MutationSchemaNotDefined)?
                                .get(&name)
                                .ok_or(Error::NotFoundError(format!("Mutation api {}", &name)))?
                                .execute(context.clone(), field)?;
                            result.insert(insert_key, mutation_result);
                        }
                        Selection::FragmentSpread(_) => {
                            return Err(Error::UnSupportedYetError(
                                "'FragmentSpread' in mutation(selection sets)".to_string(),
                            ))
                        }
                        Selection::InlineFragment(_) => {
                            return Err(Error::UnSupportedYetError(
                                "'InlineFragment' in mutation(selection sets)".to_string(),
                            ))
                        }
                    }
                }
                return Ok(DataValue::Object(result));
            }
            OperationDefinition::Subscription(_sub) => Err(Error::UnSupportedYetError(
                "'Subscription' in schema request".to_string(),
            )),
        }
    }
}

impl Drop for Schema {
    fn drop(&mut self) {
        println!("schema [{}] dropped", self.id)
    }
}
