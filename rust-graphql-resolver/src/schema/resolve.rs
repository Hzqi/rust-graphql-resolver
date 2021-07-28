use std::collections::HashMap;

use dyn_clone::{clone_trait_object, DynClone};
use gurkle_parser::query::{Selection, Value as ParserValue};

use crate::{
    error::{Error, Result},
    value::DataValue,
};

/// ApiResolveFunc
///
/// This is a function (closure) to resolve graphql api restule data
/// * context: storage and transfer key-value through invoking nested
/// * parameter: arguments and selection_sets from graphql request
pub trait ApiResolveFunc: DynClone {
    fn call(&self, context: QLContext, parameter: QLApiParam) -> Result<DataValue>;
}
clone_trait_object!(ApiResolveFunc);

impl<F> ApiResolveFunc for F
where
    F: Fn(QLContext, QLApiParam) -> Result<DataValue> + Clone,
{
    fn call(&self, context: QLContext, parameter: QLApiParam) -> Result<DataValue> {
        self(context, parameter)
    }
}

/// QLContext
pub type QLContext = HashMap<String, DataValue>;

/// QLApiParam
#[derive(Clone, Debug)]
pub struct QLApiParam {
    pub arguments: ArgumentValueMap,
    pub selection_sets: Vec<Selection>,
}

/// ArgumentValueMap
#[derive(Clone, Debug)]
pub struct ArgumentValueMap(HashMap<String, DataValue>);

impl ArgumentValueMap {
    pub fn get(&self, key: &String) -> Option<&DataValue> {
        self.0.get(key)
    }
}

impl From<Vec<(String, ParserValue)>> for ArgumentValueMap {
    fn from(list: Vec<(String, ParserValue)>) -> Self {
        let map = list
            .into_iter()
            .map(|(k, v)| (k, DataValue::from(v)))
            .collect::<HashMap<String, DataValue>>();
        Self(map)
    }
}

/// FieldResolveFunc
///
/// This is a function (closure) that for field to resolve its data result.
/// * context: storage and transfer key-value through invoking nested
/// * source: parent data value result, you can get the data from last layer, but only one layer
/// * parameter: arguments and selection_sets from graphql request
pub trait FieldResolveFunc: DynClone {
    fn call(
        &self,
        context: QLContext,
        source: DataValue,
        parameter: QLApiParam,
    ) -> Result<DataValue>;
}
clone_trait_object!(FieldResolveFunc);

impl<F> FieldResolveFunc for F
where
    F: Fn(QLContext, DataValue, QLApiParam) -> Result<DataValue> + Clone,
{
    fn call(
        &self,
        context: QLContext,
        source: DataValue,
        parameter: QLApiParam,
    ) -> Result<DataValue> {
        self(context, source, parameter)
    }
}

/// DefaultFieldResolveFunc return Err(...)
#[derive(Debug, Clone)]
pub struct DefaultFieldResolveFunc;

impl FieldResolveFunc for DefaultFieldResolveFunc {
    fn call(
        &self,
        _context: QLContext,
        _source: DataValue,
        _parameter: QLApiParam,
    ) -> Result<DataValue> {
        Err(Error::DefaultResolveError)
    }
}