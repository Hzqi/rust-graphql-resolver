use std::{
    cell::RefCell,
    collections::BTreeMap,
    fmt::Debug,
    rc::{Rc, Weak},
};

use gurkle_parser::query::Selection;

use crate::{
    error::{Error, Result},
    value::DataValue,
};

use super::resolve::{
    ArgumentValueMap, DefaultFieldResolveFunc, FieldResolveFunc, QLApiParam, QLContext,
};

use gurkle_parser::query as parser;

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
    pub(crate) fn execute<'a, 'b>(
        &self,
        context: &'a mut QLContext,
        parameter: &'b QLApiParam,
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
                        let item = list_type.execute(context, parameter, dat)?;
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
    pub(crate) fn execute<'a, 'b>(
        &self,
        context: &'a mut QLContext,
        parameter: &'b QLApiParam,
        data: DataValue,
    ) -> Result<DataValue> {
        match data {
            DataValue::Object(map) => self.execute_object(context, &parameter.selection_sets, map),
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(Error::DataTypeMisMatchError(
                "Object(CustomType)".to_string(),
                data.get_type_name(),
            )),
        }
    }

    pub(crate) fn execute_object<'a, 'b>(
        &self,
        context: &'a mut QLContext,
        selection_sets: &'b Vec<Selection>,
        mut data_map: BTreeMap<String, DataValue>,
    ) -> Result<DataValue> {
        for set in selection_sets {
            match set {
                Selection::Field(field) => {
                    let name = field.name.clone();
                    // self data does't have that key, but self fields has
                    if !data_map.contains_key(&name) && self.fields.contains_key(&name) {
                        let source = DataValue::Object(data_map.clone());
                        let field_result = self
                            .fields
                            .get(&name)
                            .unwrap()
                            .execute(context, &source, field)?;
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
    pub(crate) fn execute<'a, 'b>(
        &self,
        context: &'a mut QLContext,
        source: &'b DataValue,
        field: &'b parser::Field,
    ) -> Result<DataValue> {
        let parameter = QLApiParam {
            arguments: ArgumentValueMap::from(field.arguments.to_owned()),
            selection_sets: field.selection_set.items.clone(),
        };
        let resolve_result = self
            .resolve
            .call(context, source, &parameter)?
            .to_data_value();
        self.field_type.execute(context, &parameter, resolve_result)
    }

    pub fn new(
        name: &str,
        field_type: FieldType,
        description: &str,
        resolve: Box<dyn FieldResolveFunc>,
    ) -> Self {
        Self {
            name: name.to_string(),
            field_type,
            description: description.to_string(),
            resolve,
        }
    }

    pub fn simple(name: &str, field_type: FieldType) -> Self {
        Self::new(name, field_type, "", Box::new(DefaultFieldResolveFunc))
    }

    pub fn simple_with_description(name: &str, field_type: FieldType, desc: &str) -> Self {
        Self::new(name, field_type, desc, Box::new(DefaultFieldResolveFunc))
    }

    pub fn simple_with_resolve(
        name: &str,
        field_type: FieldType,
        resolve: Box<dyn FieldResolveFunc>,
    ) -> Self {
        Self::new(name, field_type, "", resolve)
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
    pub fn new(name: &str, field_type: InputFieldType, description: &str) -> Self {
        Self {
            name: name.to_string(),
            field_type,
            description: description.to_string(),
        }
    }

    pub fn simple(name: &str, field_type: InputFieldType) -> Self {
        Self::new(name, field_type, "")
    }

    pub fn simple_with_description(name: &str, field_type: InputFieldType, desc: &str) -> Self {
        Self::new(name, field_type, desc)
    }

    /// create a basic id field for input object
    pub fn basic_id() -> Self {
        Self {
            name: String::default(),
            field_type: InputFieldType::StaticType(StaticType::ID),
            description: String::default(),
        }
    }

    /// create a basic int field for input object
    pub fn basic_int() -> Self {
        Self {
            name: String::default(),
            field_type: InputFieldType::StaticType(StaticType::Int),
            description: String::default(),
        }
    }

    /// create a basic float field for input object
    pub fn basic_float() -> Self {
        Self {
            name: String::default(),
            field_type: InputFieldType::StaticType(StaticType::Float),
            description: String::default(),
        }
    }

    /// create a basic string field for input object
    pub fn basic_str() -> Self {
        Self {
            name: String::default(),
            field_type: InputFieldType::StaticType(StaticType::String),
            description: String::default(),
        }
    }

    /// create a basic bool field for input object
    pub fn basic_bool() -> Self {
        Self {
            name: String::default(),
            field_type: InputFieldType::StaticType(StaticType::Boolean),
            description: String::default(),
        }
    }

    /// create a basic datetime field for input object
    pub fn basic_datetime() -> Self {
        Self {
            name: String::default(),
            field_type: InputFieldType::StaticType(StaticType::DateTime),
            description: String::default(),
        }
    }
}

/// InputFieldType
#[derive(Debug, Clone)]
pub enum InputFieldType {
    StaticType(StaticType),
    NonNullType(Box<InputFieldType>),
    List(Box<InputFieldType>),
    Enum(QLEnum),
    ReferenceEnum(Rc<QLEnum>),
    QLInput(QLInput),
    ReferenceInput(Weak<RefCell<QLInput>>),
}

impl InputFieldType {
    /// create a basic id field for input object
    pub fn basic_id() -> Self {
        Self::StaticType(StaticType::ID)
    }

    /// create a basic int field for input object
    pub fn basic_int() -> Self {
        Self::StaticType(StaticType::Int)
    }

    /// create a basic float field for input object
    pub fn basic_float() -> Self {
        Self::StaticType(StaticType::Float)
    }

    /// create a basic string field for input object
    pub fn basic_str() -> Self {
        Self::StaticType(StaticType::String)
    }

    /// create a basic bool field for input object
    pub fn basic_bool() -> Self {
        Self::StaticType(StaticType::Boolean)
    }

    /// create a basic datetime field for input object
    pub fn basic_datetime() -> Self {
        Self::StaticType(StaticType::DateTime)
    }
}

/// ArgumentMap
pub type ArgumentMap = BTreeMap<String, InputFieldType>;
