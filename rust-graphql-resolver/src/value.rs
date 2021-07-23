use chrono::{DateTime, Utc};

use serde::{
    ser::{SerializeMap, SerializeSeq},
    Serialize,
};
use std::collections::BTreeMap;

use gurkle_parser::query::Value as ParserValue;

#[derive(Debug, Clone, PartialEq)]
pub enum DataValue {
    ID(String),
    String(String),
    Int(i64),
    Float(f64),
    Boolean(bool),
    DateTime(DateTime<Utc>),
    Null,
    List(Vec<DataValue>),
    Object(BTreeMap<String, DataValue>),
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
            DataValue::ID(str) => serializer.serialize_str(str),
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
