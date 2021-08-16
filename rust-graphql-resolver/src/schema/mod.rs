use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    rc::Rc,
};

use gurkle_parser::query::{
    Definition, Document, FragmentDefinition, Mutation as AstMutation, OperationDefinition,
    Query as AstQuery, Selection, SelectionSet, Subscription as AstSubscription,
};

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
    pub description: Option<String>,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum OperationKey {
    Anonymous,
    RealNamed(String),
}

impl ToString for OperationKey {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

pub(crate) struct OperationGroup {
    selection_set: Option<SelectionSet>,
    queries: HashMap<OperationKey, AstQuery>,
    mutations: HashMap<OperationKey, AstMutation>,
    subscriptions: HashMap<OperationKey, AstSubscription>,
    fragments: HashMap<String, FragmentDefinition>,
}

impl OperationGroup {
    fn count(&self) -> usize {
        let selection_set_count = if self.selection_set.is_some() {
            1_usize
        } else {
            0
        };
        selection_set_count + self.queries.len() + self.mutations.len() + self.subscriptions.len()
    }

    fn contains_anonymous(&self) -> bool {
        self.selection_set.is_some()
            && (self.queries.contains_key(&OperationKey::Anonymous)
                || self.mutations.contains_key(&OperationKey::Anonymous))
    }
}

fn opt_to_operation_key(opt: Option<String>) -> OperationKey {
    match opt {
        Some(name) => OperationKey::RealNamed(name),
        None => OperationKey::Anonymous,
    }
}

impl Default for OperationGroup {
    fn default() -> Self {
        Self {
            selection_set: None,
            queries: HashMap::default(),
            mutations: HashMap::default(),
            subscriptions: HashMap::default(),
            fragments: HashMap::default(),
        }
    }
}

impl Schema {
    pub(crate) fn grouping_document(&self, doc: Document) -> Result<OperationGroup> {
        let mut group = OperationGroup::default();
        for def in doc.definitions {
            match def {
                Definition::Operation(op) => match op {
                    OperationDefinition::SelectionSet(selection_set) => {
                        group.selection_set = Some(selection_set)
                    }
                    OperationDefinition::Query(query) => {
                        let key = opt_to_operation_key(query.name.clone());
                        let res = group.queries.insert(key.clone(), query).is_some();
                        if res {
                            return Err(Error::OnlyOneOperationCanNamed(key.to_string()));
                        }
                    }
                    OperationDefinition::Mutation(mutation) => {
                        let key = opt_to_operation_key(mutation.name.clone());
                        let res = group.mutations.insert(key.clone(), mutation).is_some();
                        if res {
                            return Err(Error::OnlyOneOperationCanNamed(key.to_string()));
                        }
                    }
                    OperationDefinition::Subscription(sub) => {
                        let key = opt_to_operation_key(sub.name.clone());
                        let res = group.subscriptions.insert(key.clone(), sub).is_some();
                        if res {
                            return Err(Error::OnlyOneOperationCanNamed(key.to_string()));
                        }
                    }
                },
                Definition::Fragment(frag) => {
                    group.fragments.insert(frag.name.clone(), frag);
                }
            }
        }
        Ok(group)
    }

    pub(crate) fn execute_document(
        &self,
        context: QLContext,
        doc: Document,
        operation_name: Option<String>,
    ) -> Result<DataValue> {
        let group = self.grouping_document(doc)?;

        let key = opt_to_operation_key(operation_name);

        if group.count() > 1 {
            if let OperationKey::Anonymous = key {
                return Err(Error::MultipleOperationNeedTarget);
            }
            if group.contains_anonymous() {
                return Err(Error::MustBeDefinedAnonymousOperation);
            }
        }

        if let Some(selection_set) = group.selection_set {
            return self.execute_selection_set(context, selection_set, &group.fragments);
        }

        if let Some(query) = group.queries.get(&key) {
            return self.execute_query(context, query.to_owned(), &group.fragments);
        }

        if let Some(mutation) = group.mutations.get(&key) {
            return self.execute_mutation(context, mutation.to_owned(), &group.fragments);
        }

        if let Some(_sub) = group.subscriptions.get(&key) {
            return Err(Error::UnSupportedYetError(
                "'Subscription' in schema request".to_string(),
            ));
        }

        Err(Error::NotFoundError(format!(
            "Operation named '{}'",
            key.to_string()
        )))
    }

    pub(crate) fn execute_selection_set(
        &self,
        mut context: QLContext,
        sets: SelectionSet,
        fragments: &HashMap<String, FragmentDefinition>,
    ) -> Result<DataValue> {
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
                        .execute(&mut context, fragments, field)?;
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
        Ok(DataValue::Object(result))
    }

    pub(crate) fn execute_query(
        &self,
        context: QLContext,
        query: AstQuery,
        fragments: &HashMap<String, FragmentDefinition>,
    ) -> Result<DataValue> {
        self.execute_selection_set(context, query.selection_set, fragments)
    }

    pub(crate) fn execute_mutation(
        &self,
        mut context: QLContext,
        mutation: AstMutation,
        fragments: &HashMap<String, FragmentDefinition>,
    ) -> Result<DataValue> {
        let mut result = BTreeMap::<String, DataValue>::new();
        for set in mutation.selection_set.items {
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
                        .execute(&mut context, fragments, field)?;
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
}

impl Drop for Schema {
    fn drop(&mut self) {
        println!("schema [{}] dropped", self.id)
    }
}
