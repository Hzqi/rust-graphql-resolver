use chrono::{DateTime, Utc};

use serde::{
    ser::{SerializeMap, SerializeSeq},
    Serialize,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
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
