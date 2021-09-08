use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;

use gurkle_parser::query::{self as ast};

use dyn_clone::{clone_trait_object, DynClone};

use crate::{
    error::{Error, Result},
    value::DataValue,
};

use super::{
    executor::TypeExecutor,
    field::{Field, InputField},
    resolve::{QLApiParam, QLContext},
    Schema,
};

/// FieldType
#[derive(Clone, Debug)]
pub enum FieldType {
    ScalarType(ScalarType),
    NonNullType(Box<FieldType>),
    List(Box<FieldType>),
    EnumType(EnumType),
    ObjectType(ObjectType),
    // TODO: Interface and Union
    // InterfaceType(InterfaceType),
    // UnionType(UnionType),
}

impl TypeExecutor for FieldType {
    fn execute<'schema, 'a, 'b>(
        &self,
        schema: &'schema Schema,
        context: &'a mut QLContext,
        fragments: &'b HashMap<String, ast::FragmentDefinition>,
        parameter: &'b QLApiParam,
        data: DataValue,
    ) -> Result<DataValue> {
        match self {
            FieldType::ScalarType(t) => t.execute(schema, context, fragments, parameter, data),
            FieldType::NonNullType(t) => match data {
                DataValue::Null => Err(Error::DataTypeMisMatchError(
                    "NonNull<...>".to_string(),
                    "Null".to_string(),
                )),
                _ => t.execute(schema, context, fragments, parameter, data),
            },
            FieldType::List(t) => match data {
                DataValue::List(data_list) => {
                    let mut result = vec![];
                    for dat in data_list {
                        let item = t.execute(schema, context, fragments, parameter, dat)?;
                        result.push(item)
                    }
                    Ok(DataValue::List(result))
                }
                v => Err(Error::DataTypeMisMatchError(
                    "List<...>".to_string(),
                    v.get_type_name(),
                )),
            },
            FieldType::EnumType(t) => t.execute(schema, context, fragments, parameter, data),
            FieldType::ObjectType(t) => t.execute(schema, context, fragments, parameter, data),
        }
    }
}

/// ScalarType
#[derive(Clone, Debug)]
pub enum ScalarType {
    ID,
    String,
    Int,
    Float,
    Boolean,
    DateTime,
    CustomScalar(Box<dyn CustomScalar>),
}

impl TypeExecutor for ScalarType {
    fn execute<'schema, 'a, 'b>(
        &self,
        _schema: &'schema Schema,
        _context: &'a mut QLContext,
        _fragments: &'b HashMap<String, ast::FragmentDefinition>,
        _parameter: &'b QLApiParam,
        data: DataValue,
    ) -> Result<DataValue> {
        match (self, data) {
            (ScalarType::ID, r @ DataValue::String(_)) => Ok(r),
            (ScalarType::String, r @ DataValue::String(_)) => Ok(r),
            (ScalarType::Int, r @ DataValue::Int(_)) => Ok(r),
            (ScalarType::Float, r @ DataValue::Float(_)) => Ok(r),
            (ScalarType::Boolean, r @ DataValue::Boolean(_)) => Ok(r),
            (ScalarType::DateTime, r @ DataValue::DateTime(_)) => Ok(r),
            (ScalarType::CustomScalar(custom), r) => custom.execute(r),
            (_, data) => Err(Error::DataTypeMisMatchError(
                format!("{:?}", self),
                format!("{:?}", data),
            )),
        }
    }
}

pub trait CustomScalar: DynClone + Debug {
    fn execute(&self, data: DataValue) -> Result<DataValue>;
}
clone_trait_object!(CustomScalar);

/// EnumType
#[derive(Clone, Debug)]
pub enum EnumType {
    DirectEnum(DirectEnum),
    LinkEnum(String),
}

impl TypeExecutor for EnumType {
    fn execute<'schema, 'a, 'b>(
        &self,
        schema: &'schema Schema,
        context: &'a mut QLContext,
        fragments: &'b HashMap<String, ast::FragmentDefinition>,
        parameter: &'b QLApiParam,
        data: DataValue,
    ) -> Result<DataValue> {
        match self {
            EnumType::DirectEnum(e) => e.execute(schema, context, fragments, parameter, data),
            EnumType::LinkEnum(key) => match schema.type_storage.enums.get(key) {
                Some(obj) => obj.execute(schema, context, fragments, parameter, data),
                None => Err(Error::NotFoundError(format!("object type: {}", key))),
            },
        }
    }
}

/// DirectEnum
#[derive(Clone, Debug)]
pub struct DirectEnum {
    pub name: String,
    pub description: String,
    pub values: Vec<EnumValue>,
}

impl TypeExecutor for DirectEnum {
    fn execute<'schema, 'a, 'b>(
        &self,
        schema: &'schema Schema,
        context: &'a mut QLContext,
        fragments: &'b HashMap<String, ast::FragmentDefinition>,
        parameter: &'b QLApiParam,
        data: DataValue,
    ) -> Result<DataValue> {
        match data {
            DataValue::String(str) => self
                .values
                .iter()
                .find(|v| v.value == str)
                .and(Some(data))
                .ok_or(Error::NotFoundError(format!(
                    "enum value: {} in enum: {} ",
                    str, self.name
                ))),
            oth => Err(Error::DataTypeMisMatchError(
                "Enum with String value".to_string(),
                oth.get_type_name(),
            )),
        }
    }
}

/// EnumValue
#[derive(Clone, Debug)]
pub struct EnumValue {
    pub value: String,
    pub description: String,
}

/// ObjectType
#[derive(Clone, Debug)]
pub enum ObjectType {
    DirectObject(DirectObject),
    LinkObject(String),
}

impl TypeExecutor for ObjectType {
    fn execute<'schema, 'a, 'b>(
        &self,
        schema: &'schema Schema,
        context: &'a mut QLContext,
        fragments: &'b HashMap<String, ast::FragmentDefinition>,
        parameter: &'b QLApiParam,
        data: DataValue,
    ) -> Result<DataValue> {
        match self {
            ObjectType::DirectObject(obj) => {
                obj.execute(schema, context, fragments, parameter, data)
            }
            ObjectType::LinkObject(key) => match schema.type_storage.objects.get(key) {
                Some(obj) => obj.execute(schema, context, fragments, parameter, data),
                None => Err(Error::NotFoundError(format!("object type: {}", key))),
            },
        }
    }
}

/// DirectObject
#[derive(Clone, Debug)]
pub struct DirectObject {
    pub name: String,
    pub fields: BTreeMap<String, Field>,
    // TODO: Interface
    //pub of_type: Option<InterfaceType>,
    pub description: String,
}

impl DirectObject {
    fn execute_object<'schema, 'a, 'b>(
        &self,
        schema: &'schema Schema,
        context: &'a mut QLContext,
        fragments: &'b HashMap<String, ast::FragmentDefinition>,
        selection_sets: &'b Vec<ast::Selection>,
        data_map: &BTreeMap<String, DataValue>,
    ) -> Result<BTreeMap<String, DataValue>> {
        let mut result = BTreeMap::new();
        for set in selection_sets {
            match set {
                ast::Selection::Field(field) => {
                    let name = field.name.clone();
                    // self data does't have that key, but self fields has
                    if !data_map.contains_key(&name) && self.fields.contains_key(&name) {
                        let source = DataValue::Object(data_map.clone());
                        let field_result = self
                            .fields
                            .get(&name)
                            .unwrap()
                            .execute(schema, context, fragments, field, &source)?;
                        result.insert(name, field_result);
                    } else if !data_map.contains_key(&name) && !self.fields.contains_key(&name) {
                        result.insert(name, DataValue::Null);
                    } else if data_map.contains_key(&name) && !self.fields.contains_key(&name) {
                        result.remove(&name);
                    }
                }
                ast::Selection::FragmentSpread(fs) => {
                    let fragment = fragments
                        .get(&fs.fragment_name)
                        .ok_or(Error::NoSuchFragment(fs.fragment_name.clone()))?;
                    let next_result = self.execute_object(
                        schema,
                        context,
                        fragments,
                        &fragment.selection_set.items,
                        data_map,
                    )?;
                    result.extend(next_result);
                }
                ast::Selection::InlineFragment(_) => {
                    return Err(Error::UnSupportedYetError(
                        "'InlineFragment' in object field".to_string(),
                    ))
                }
            }
        }
        Ok(result)
    }
}

impl TypeExecutor for DirectObject {
    fn execute<'schema, 'a, 'b>(
        &self,
        schema: &'schema Schema,
        context: &'a mut QLContext,
        fragments: &'b HashMap<String, ast::FragmentDefinition>,
        parameter: &'b QLApiParam,
        data: DataValue,
    ) -> Result<DataValue> {
        match data {
            DataValue::Object(map) => {
                let result = self.execute_object(
                    schema,
                    context,
                    fragments,
                    &parameter.selection_sets,
                    &map,
                )?;
                Ok(DataValue::Object(result))
            }
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(Error::DataTypeMisMatchError(
                "Object(CustomType)".to_string(),
                data.get_type_name(),
            )),
        }
    }
}

/// TODO: InterfaceType
// #[derive(Clone, Debug)]
// pub enum InterfaceType {
//     DirectInterface(DirectInterface),
//     LinkInterface(String),
// }

/// TODO: DirectInterface
// #[derive(Clone, Debug)]
// pub struct DirectInterface {
//     pub name: String,
//     pub fields: BTreeMap<String, Field>,
//     pub description: String,
// }

/// TODO: UnionType
// #[derive(Clone, Debug)]
// pub enum UnionType {
//     DirectUnion(DirectUnion),
//     LinkUnion(String),
// }

/// TODO: DirectUnion
// #[derive(Clone, Debug)]
// pub struct DirectUnion {
//     pub name: String,
//     pub objects: Vec<ObjectType>,
//     pub description: String,
// }

/// InputType
#[derive(Clone, Debug)]
pub enum InputType {
    ScalarType(ScalarType),
    NonNullType(Box<FieldType>),
    List(Box<FieldType>),
    EnumType(EnumType),
    ObjectType(InputObjectType),
}

/// InputObject
#[derive(Clone, Debug)]
pub enum InputObjectType {
    DirectInputObject(DirectInputObject),
    LinkInputObject(String),
}

/// DirectInputObject
#[derive(Clone, Debug)]
pub struct DirectInputObject {
    pub name: String,
    pub fields: BTreeMap<String, InputField>,
    pub description: String,
}

/// ArgumentMap
pub type ArgumentMap = BTreeMap<String, InputType>;

// TODO: execute_introspection()
// 每个类型都实现一个execute_introspection
