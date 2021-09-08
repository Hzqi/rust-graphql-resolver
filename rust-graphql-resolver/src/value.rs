use chrono::{DateTime, Utc};

use serde::{
    ser::{SerializeMap, SerializeSeq},
    Serialize,
};
use std::collections::{BTreeMap, HashMap};

use gurkle_parser::query::Value as ParserValue;

#[derive(Debug, Clone, PartialEq)]
pub enum DataValue {
    String(String),
    Int(i64),
    Float(f64),
    Boolean(bool),
    DateTime(DateTime<Utc>),
    Null,
    List(Vec<DataValue>),
    Object(BTreeMap<String, DataValue>),
}

impl DataValue {
    pub fn boxed_string(s: String) -> Box<DataValue> {
        Box::new(Self::String(s))
    }
    pub fn boxed_int(i: i64) -> Box<DataValue> {
        Box::new(Self::Int(i))
    }
    pub fn boxed_float(f: f64) -> Box<DataValue> {
        Box::new(Self::Float(f))
    }
    pub fn boxed_bool(b: bool) -> Box<DataValue> {
        Box::new(Self::Boolean(b))
    }
    pub fn boxed_null() -> Box<DataValue> {
        Box::new(Self::Null)
    }
    pub fn boxed_datetime(dt: DateTime<Utc>) -> Box<DataValue> {
        Box::new(Self::DateTime(dt))
    }
    pub fn boxed_list(list: Vec<DataValue>) -> Box<DataValue> {
        Box::new(Self::List(list))
    }
    pub fn boxed_object(map: BTreeMap<String, DataValue>) -> Box<DataValue> {
        Box::new(Self::Object(map))
    }

    pub fn to_boxed(self) -> Box<DataValue> {
        Box::new(self)
    }

    pub fn get_type_name(&self) -> String {
        match self {
            DataValue::String(_) => "String".to_string(),
            DataValue::Int(_) => "Int".to_string(),
            DataValue::Float(_) => "Float".to_string(),
            DataValue::Boolean(_) => "Boolean".to_string(),
            DataValue::DateTime(_) => "DateTime".to_string(),
            DataValue::Null => "Null".to_string(),
            DataValue::List(item) => {
                if let Some(head) = item.first() {
                    format!("List({})", head.get_type_name())
                } else {
                    "List(...)".to_string()
                }
            }
            DataValue::Object(_) => "Object(...)".to_string(),
        }
    }
}

// From transform from parser value
impl From<ParserValue> for DataValue {
    fn from(value: ParserValue) -> Self {
        match value {
            ParserValue::Variable(str) => DataValue::String(str),
            ParserValue::Int(num) => DataValue::Int(num.as_i64().unwrap()),
            ParserValue::Float(f) => DataValue::Float(f),
            ParserValue::String(str) => DataValue::String(str),
            ParserValue::Boolean(b) => DataValue::Boolean(b),
            ParserValue::Null => DataValue::Null,
            ParserValue::Enum(str) => DataValue::String(str),
            ParserValue::List(list) => {
                DataValue::List(list.into_iter().map(|v| DataValue::from(v)).collect())
            }
            ParserValue::Object(map) => {
                let new_map = map
                    .into_iter()
                    .map(|(k, v)| (k, DataValue::from(v)))
                    .collect::<BTreeMap<String, DataValue>>();
                DataValue::Object(new_map)
            }
        }
    }
}

// Serialize for DataValue to json
impl Serialize for DataValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            DataValue::String(str) => serializer.serialize_str(str),
            DataValue::Int(i) => serializer.serialize_i64(*i),
            DataValue::Float(f) => serializer.serialize_f64(*f),
            DataValue::Boolean(b) => serializer.serialize_bool(*b),
            DataValue::DateTime(dt) => dt.serialize(serializer),
            DataValue::Null => serializer.serialize_none(),
            DataValue::List(list) => {
                let mut seq = serializer.serialize_seq(Some(list.len()))?;
                for e in list {
                    seq.serialize_element(e)?;
                }
                seq.end()
            }
            DataValue::Object(m) => {
                let mut map = serializer.serialize_map(Some(m.len()))?;
                for (k, v) in m {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
        }
    }
}

// For custom data struct to transform to DataValue
pub trait ToDataValue {
    fn to_data_value(&self) -> DataValue;
}

impl ToDataValue for DataValue {
    fn to_data_value(&self) -> DataValue {
        self.clone()
    }
}

impl ToDataValue for String {
    fn to_data_value(&self) -> DataValue {
        DataValue::String(self.to_owned())
    }
}

impl ToDataValue for dyn ToString {
    fn to_data_value(&self) -> DataValue {
        DataValue::String(self.to_string())
    }
}

impl ToDataValue for i64 {
    fn to_data_value(&self) -> DataValue {
        DataValue::Int(self.to_owned())
    }
}

impl ToDataValue for i32 {
    fn to_data_value(&self) -> DataValue {
        DataValue::Int(self.to_owned() as i64)
    }
}

impl ToDataValue for i16 {
    fn to_data_value(&self) -> DataValue {
        DataValue::Int(self.to_owned() as i64)
    }
}

impl ToDataValue for i8 {
    fn to_data_value(&self) -> DataValue {
        DataValue::Int(self.to_owned() as i64)
    }
}

impl ToDataValue for u64 {
    fn to_data_value(&self) -> DataValue {
        DataValue::Int(self.to_owned() as i64)
    }
}

impl ToDataValue for u32 {
    fn to_data_value(&self) -> DataValue {
        DataValue::Int(self.to_owned() as i64)
    }
}

impl ToDataValue for u16 {
    fn to_data_value(&self) -> DataValue {
        DataValue::Int(self.to_owned() as i64)
    }
}

impl ToDataValue for u8 {
    fn to_data_value(&self) -> DataValue {
        DataValue::Int(self.to_owned() as i64)
    }
}

impl ToDataValue for f64 {
    fn to_data_value(&self) -> DataValue {
        DataValue::Float(self.to_owned() as f64)
    }
}

impl ToDataValue for f32 {
    fn to_data_value(&self) -> DataValue {
        DataValue::Float(self.to_owned() as f64)
    }
}

impl ToDataValue for bool {
    fn to_data_value(&self) -> DataValue {
        DataValue::Boolean(self.to_owned())
    }
}

impl ToDataValue for DateTime<Utc> {
    fn to_data_value(&self) -> DataValue {
        DataValue::DateTime(self.to_owned())
    }
}

impl<K: ToString, V: ToDataValue> ToDataValue for HashMap<K, V> {
    fn to_data_value(&self) -> DataValue {
        let btree = self
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_data_value()))
            .collect();
        DataValue::Object(btree)
    }
}

impl<K: ToString, V: ToDataValue> ToDataValue for BTreeMap<K, V> {
    fn to_data_value(&self) -> DataValue {
        let btree = self
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_data_value()))
            .collect();
        DataValue::Object(btree)
    }
}

impl<T: ToDataValue> ToDataValue for Vec<T> {
    fn to_data_value(&self) -> DataValue {
        let list = self.into_iter().map(|v| v.to_data_value()).collect();
        DataValue::List(list)
    }
}

impl<T: ToDataValue> ToDataValue for Option<T> {
    fn to_data_value(&self) -> DataValue {
        match self {
            Some(v) => v.to_data_value(),
            None => DataValue::Null,
        }
    }
}
