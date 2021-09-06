use std::collections::HashMap;

use dyn_clone::{clone_trait_object, DynClone};
use gurkle_parser::query::{Selection, Value as ParserValue};

use crate::{
    error::{Error, Result},
    value::{DataValue, ToDataValue},
};

pub type BoxedValue = Box<dyn ToDataValue>;

/// ApiResolveFunc
///
/// This is a function (closure) to resolve graphql api restule data
/// * context: storage and transfer key-value through invoking nested
/// * parameter: arguments and selection_sets from graphql request
pub trait ApiResolveFunc: DynClone {
    fn call<'a, 'b>(
        &self,
        context: &'a mut QLContext,
        parameter: &'b QLApiParam,
    ) -> Result<BoxedValue>;
}
clone_trait_object!(ApiResolveFunc);

impl<F> ApiResolveFunc for F
where
    F: Fn(&'_ mut QLContext, &'_ QLApiParam) -> Result<BoxedValue> + Clone,
{
    fn call<'a, 'b>(
        &self,
        context: &'a mut QLContext,
        parameter: &'b QLApiParam,
    ) -> Result<BoxedValue> {
        self(context, parameter)
    }
}

/// DefaultFieldResolveFunc return Err(...)
#[derive(Debug, Clone)]
pub struct DefaultApiResolveFunc;

impl ApiResolveFunc for DefaultApiResolveFunc {
    fn call<'a, 'b>(
        &self,
        _context: &'a mut QLContext,
        _parameter: &'b QLApiParam,
    ) -> Result<BoxedValue> {
        Err(Error::DefaultResolveError)
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
    fn call<'a, 'b>(
        &self,
        context: &'a mut QLContext,
        source: &'b DataValue,
        parameter: &'b QLApiParam,
    ) -> Result<BoxedValue>;
}
clone_trait_object!(FieldResolveFunc);

impl<F> FieldResolveFunc for F
where
    F: Fn(&'_ mut QLContext, &'_ DataValue, &'_ QLApiParam) -> Result<BoxedValue> + Clone,
{
    fn call<'a, 'b>(
        &self,
        context: &'a mut QLContext,
        source: &'b DataValue,
        parameter: &'b QLApiParam,
    ) -> Result<BoxedValue> {
        self(context, source, parameter)
    }
}

/// DefaultFieldResolveFunc return Err(...)
#[derive(Debug, Clone)]
pub struct DefaultFieldResolveFunc;

impl FieldResolveFunc for DefaultFieldResolveFunc {
    fn call<'a, 'b>(
        &self,
        _context: &'a mut QLContext,
        _source: &'b DataValue,
        _parameter: &'b QLApiParam,
    ) -> Result<BoxedValue> {
        Err(Error::DefaultResolveError)
    }
}
