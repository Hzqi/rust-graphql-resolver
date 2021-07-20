use crate::{
    error::{Error, Result},
    value::DataValue,
};
use dyn_clone::{clone_trait_object, DynClone};
use gurkle_parser::query::{
    self, Definition, Document, OperationDefinition, Selection, Value as CommonValue,
};
use std::{
    array::IntoIter,
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    fmt::Debug,
    iter::FromIterator,
    rc::{Rc, Weak},
};

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
    pub query: QueryMap,
    /// TODO: Mutation
    pub mutation: NotSupported,
    /// TODO: Subscrition
    pub subscrition: NotSupported,
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
                            let query_result = self
                                .query
                                .queries
                                .get(&name)
                                .ok_or(Error::NotFoundError(format!("Query api {}", &name)))?
                                .execute(context.clone(), field)?;
                            result.insert(name, query_result);
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
                            let query_result = self
                                .query
                                .queries
                                .get(&name)
                                .ok_or(Error::NotFoundError(format!("Query api {}", &name)))?
                                .execute(context.clone(), field)?;
                            result.insert(name, query_result);
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
            OperationDefinition::Mutation(_mutations) => Err(Error::UnSupportedYetError(
                "'Mutation' in schema request".to_string(),
            )),
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

/// QueryMap
#[derive(Clone, Debug)]
pub struct QueryMap {
    pub queries: HashMap<String, Query>,
    pub objects: HashMap<String, Rc<RefCell<CustomType>>>,
    pub enums: HashMap<String, Rc<QLEnum>>,
    pub inputs: HashMap<String, Rc<RefCell<QLInput>>>,
}

/// Query
#[derive(Clone)]
pub struct Query {
    pub field_type: FieldType,
    pub arguments: ArgumentMap,
    pub description: String,
    pub resolve: Box<dyn ApiResolveFunc>,
}

impl Query {
    pub(crate) fn execute(&self, context: QLContext, field: query::Field) -> Result<DataValue> {
        let parameter = QLApiParam {
            arguments: field.arguments.into_iter().collect(),
            selection_sets: field.selection_set.items,
        };
        let resolve_result = self.resolve.call(context.clone(), parameter.clone())?;
        self.field_type.execute(context, parameter, resolve_result)
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
    pub arguments: HashMap<String, CommonValue>,
    pub selection_sets: Vec<Selection>,
}

/// FieldType
#[derive(Clone, Debug)]
pub enum FieldType {
    StaticType(StaticType),
    NonNullType(Box<FieldType>),
    List(Box<FieldType>),
    Enum(QLEnum),
    ReferenceEnum(Rc<QLEnum>),
    CustomType(CustomType),
    ReferenceCustom(Weak<RefCell<CustomType>>),
}

impl FieldType {
    pub(crate) fn execute(
        &self,
        context: QLContext,
        parameter: QLApiParam,
        data: DataValue,
    ) -> Result<DataValue> {
        match self {
            FieldType::StaticType(t) => t.execute(data),
            FieldType::NonNullType(t) => match data {
                DataValue::Null => Err(Error::DataTypeMisMatchError(
                    "NonNull<...>".to_string(),
                    "Null".to_string(),
                )),
                _ => t.execute(context, parameter, data),
            },
            FieldType::List(list_type) => match data {
                DataValue::List(data_list) => {
                    let mut result = vec![];
                    for dat in data_list {
                        let item = list_type.execute(context.clone(), parameter.clone(), dat)?;
                        result.push(item)
                    }
                    Ok(DataValue::List(result))
                }
                _ => Err(Error::DataTypeMisMatchError(
                    "List<...>".to_string(),
                    "NonNull".to_string(),
                )),
            },
            FieldType::Enum(_enum_type) => match data {
                DataValue::String(_) => Ok(data),
                _ => Err(Error::DataTypeMisMatchError(
                    "Enum with String value".to_string(),
                    "NonString".to_string(),
                )),
            },
            FieldType::ReferenceEnum(_enum_type) => match data {
                DataValue::String(_) => Ok(data),
                _ => Err(Error::DataTypeMisMatchError(
                    "Enum with String value".to_string(),
                    "NonString".to_string(),
                )),
            },
            FieldType::CustomType(custom_type) => custom_type.execute(context, parameter, data),
            FieldType::ReferenceCustom(custom_type_rc) => custom_type_rc
                .upgrade()
                .ok_or(Error::MissingReferenceCustomTypeError)?
                .borrow()
                .execute(context, parameter, data),
        }
    }
}

/// StaticType
#[derive(Clone, Debug)]
pub enum StaticType {
    ID,
    String,
    Int,
    Float,
    Boolean,
    DateTime,
}

impl StaticType {
    pub(crate) fn execute(&self, data: DataValue) -> Result<DataValue> {
        match (self, data) {
            (StaticType::ID, r @ DataValue::String(_)) => Ok(r),
            (StaticType::String, r @ DataValue::String(_)) => Ok(r),
            (StaticType::Int, r @ DataValue::Int(_)) => Ok(r),
            (StaticType::Float, r @ DataValue::Float(_)) => Ok(r),
            (StaticType::Boolean, r @ DataValue::Boolean(_)) => Ok(r),
            (StaticType::DateTime, r @ DataValue::DateTime(_)) => Ok(r),
            (_, data) => Err(Error::DataTypeMisMatchError(
                format!("{:?}", self),
                format!("{:?}", data),
            )),
        }
    }
}

/// CustomType
#[derive(Clone, Debug)]
pub struct CustomType {
    pub name: String,
    pub fields: BTreeMap<String, Field>,
    pub description: String,
}

impl CustomType {
    pub(crate) fn execute(
        &self,
        context: QLContext,
        parameter: QLApiParam,
        data: DataValue,
    ) -> Result<DataValue> {
        match data {
            DataValue::Object(map) => self.execute_object(context, parameter.selection_sets, map),
            _ => Err(Error::DataTypeMisMatchError(
                "Object(CustomType)".to_string(),
                "NonObject".to_string(),
            )),
        }
    }

    pub(crate) fn execute_object(
        &self,
        context: QLContext,
        selection_sets: Vec<Selection>,
        mut data_map: BTreeMap<String, DataValue>,
    ) -> Result<DataValue> {
        for set in selection_sets {
            match set {
                Selection::Field(field) => {
                    let name = field.name.clone();
                    // self data does't have that key, but self fields has
                    if !data_map.contains_key(&name) && self.fields.contains_key(&name) {
                        let source = DataValue::Object(data_map.clone());
                        let field_result = self.fields.get(&name).unwrap().execute(
                            context.clone(),
                            source,
                            field,
                        )?;
                        data_map.insert(name, field_result);
                    } else if !data_map.contains_key(&name) && !self.fields.contains_key(&name) {
                        data_map.insert(name, DataValue::Null);
                    } else if data_map.contains_key(&name) && !self.fields.contains_key(&name) {
                        data_map.remove(&name);
                    } else {
                        continue;
                    }
                }
                Selection::FragmentSpread(_) => {
                    return Err(Error::UnSupportedYetError(
                        "'FragmentSpread' in custom field".to_string(),
                    ))
                }
                Selection::InlineFragment(_) => {
                    return Err(Error::UnSupportedYetError(
                        "'InlineFragment' in custom field".to_string(),
                    ))
                }
            }
        }
        Ok(DataValue::Object(data_map))
    }
}

/// Field
#[derive(Clone)]
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
    pub description: String,
    pub resolve: Box<dyn FieldResolveFunc>,
}

impl Field {
    pub(crate) fn execute(
        &self,
        context: QLContext,
        source: DataValue,
        field: query::Field,
    ) -> Result<DataValue> {
        let parameter = QLApiParam {
            arguments: field.arguments.into_iter().collect(),
            selection_sets: field.selection_set.items.clone(),
        };
        let resolve_result = self
            .resolve
            .call(context.clone(), source, parameter.clone())?;
        self.field_type.execute(context, parameter, resolve_result)
    }

    /// create a basic id field without resolve
    pub fn basic_id() -> Self {
        Self {
            name: String::default(),
            field_type: FieldType::StaticType(StaticType::ID),
            description: String::default(),
            resolve: Box::new(DefaultFieldResolveFunc),
        }
    }

    /// create a basic int field without resolve
    pub fn basic_int() -> Self {
        Self {
            name: String::default(),
            field_type: FieldType::StaticType(StaticType::Int),
            description: String::default(),
            resolve: Box::new(DefaultFieldResolveFunc),
        }
    }

    /// create a basic float field without resolve
    pub fn basic_float() -> Self {
        Self {
            name: String::default(),
            field_type: FieldType::StaticType(StaticType::Float),
            description: String::default(),
            resolve: Box::new(DefaultFieldResolveFunc),
        }
    }

    /// create a basic string field without resolve
    pub fn basic_str() -> Self {
        Self {
            name: String::default(),
            field_type: FieldType::StaticType(StaticType::String),
            description: String::default(),
            resolve: Box::new(DefaultFieldResolveFunc),
        }
    }

    /// create a basic bool field without resolve
    pub fn basic_bool() -> Self {
        Self {
            name: String::default(),
            field_type: FieldType::StaticType(StaticType::Boolean),
            description: String::default(),
            resolve: Box::new(DefaultFieldResolveFunc),
        }
    }

    /// create a basic datetime field without resolve
    pub fn basic_datetime() -> Self {
        Self {
            name: String::default(),
            field_type: FieldType::StaticType(StaticType::DateTime),
            description: String::default(),
            resolve: Box::new(DefaultFieldResolveFunc),
        }
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

/// QLEnum
///
/// Graphql enumuation definition. This can be storaged in `QueryMap`'s `enums`.
/// * If your enum is used for once (for definition), you can set `FieldType::Enum([YOUR-ENUM])` directly.
/// * If your enum is uesd for more than once, you can store the enum in `QueryMap`, and use `FieldType::Reference([YOUR-ENUM-RC])` to reference
#[derive(Clone, Debug)]
pub struct QLEnum {
    pub name: String,
    pub description: String,
    pub values: Vec<QLEnumValue>,
}

/// QLEnumValue
///
/// Graphql enumuation value definition
#[derive(Clone, Debug)]
pub struct QLEnumValue {
    pub value: String,
    pub description: String,
}

/// QLInput
///
/// Graphql enumuation definition. This can be storaged in `QueryMap`'s `inputs`.
/// * If your input is used for once (for definition), you can set it (`QLInput`) in your argument directly.
/// * If your input is uesd for more than once, you can store the input in `QueryMap`, and use its reference (`Rc<RefCell<QLInput>>`)
#[derive(Clone, Debug)]
pub struct QLInput {
    pub name: String,
    pub fields: BTreeMap<String, InputField>,
    pub description: String,
}

/// InputField
#[derive(Clone, Debug)]
pub struct InputField {
    pub name: String,
    pub field_type: InputFieldType,
    pub description: String,
}

impl InputField {
    /// create a basic id field for input object
    pub fn basic_id() -> Self {
        Self {
            name: String::default(),
            field_type: InputFieldType::ObjectFieldType(FieldType::StaticType(StaticType::ID)),
            description: String::default(),
        }
    }

    /// create a basic int field for input object
    pub fn basic_int() -> Self {
        Self {
            name: String::default(),
            field_type: InputFieldType::ObjectFieldType(FieldType::StaticType(StaticType::Int)),
            description: String::default(),
        }
    }

    /// create a basic float field for input object
    pub fn basic_float() -> Self {
        Self {
            name: String::default(),
            field_type: InputFieldType::ObjectFieldType(FieldType::StaticType(StaticType::Float)),
            description: String::default(),
        }
    }

    /// create a basic string field for input object
    pub fn basic_str() -> Self {
        Self {
            name: String::default(),
            field_type: InputFieldType::ObjectFieldType(FieldType::StaticType(StaticType::String)),
            description: String::default(),
        }
    }

    /// create a basic bool field for input object
    pub fn basic_bool() -> Self {
        Self {
            name: String::default(),
            field_type: InputFieldType::ObjectFieldType(FieldType::StaticType(StaticType::Boolean)),
            description: String::default(),
        }
    }

    /// create a basic datetime field for input object
    pub fn basic_datetime() -> Self {
        Self {
            name: String::default(),
            field_type: InputFieldType::ObjectFieldType(FieldType::StaticType(
                StaticType::DateTime,
            )),
            description: String::default(),
        }
    }
}

/// InputFieldType
#[derive(Debug, Clone)]
pub enum InputFieldType {
    ObjectFieldType(FieldType),
    QLInput(QLInput),
    ReferenceInput(Weak<RefCell<QLInput>>),
}

/// ArgumentMap
pub type ArgumentMap = BTreeMap<String, InputFieldType>;
