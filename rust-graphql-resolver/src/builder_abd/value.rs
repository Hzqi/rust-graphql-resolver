use std::collections::BTreeMap;

use chrono::{DateTime, Utc};

use crate::value::DataValue;

pub struct DataValueObjectBuilder {
    status: BTreeMap<String, DataValue>,
}

impl DataValueObjectBuilder {
    pub fn new() -> Self {
        Self {
            status: BTreeMap::new(),
        }
    }

    pub fn build(self) -> DataValue {
        DataValue::Object(self.status)
    }

    pub fn add_any_field(mut self, name: &str, value: DataValue) -> Self {
        self.status.insert(name.to_string(), value);
        self
    }

    pub fn add_str_field(mut self, name: &str, value: String) -> Self {
        self.status
            .insert(name.to_string(), DataValue::String(value));
        self
    }

    pub fn add_id_field(mut self, name: &str, value: String) -> Self {
        self.status.insert(name.to_string(), DataValue::ID(value));
        self
    }
    pub fn add_int_field(mut self, name: &str, value: i64) -> Self {
        self.status.insert(name.to_string(), DataValue::Int(value));
        self
    }
    pub fn add_float_field(mut self, name: &str, value: f64) -> Self {
        self.status
            .insert(name.to_string(), DataValue::Float(value));
        self
    }
    pub fn add_bool_field(mut self, name: &str, value: bool) -> Self {
        self.status
            .insert(name.to_string(), DataValue::Boolean(value));
        self
    }
    pub fn add_null_field(mut self, name: &str) -> Self {
        self.status.insert(name.to_string(), DataValue::Null);
        self
    }
    pub fn add_datetime_field(mut self, name: &str, value: DateTime<Utc>) -> Self {
        self.status
            .insert(name.to_string(), DataValue::DateTime(value));
        self
    }
    pub fn add_list_field(mut self, name: &str, value: Vec<DataValue>) -> Self {
        self.status.insert(name.to_string(), DataValue::List(value));
        self
    }
    pub fn add_object_field(mut self, name: &str, value: DataValue) -> Self {
        match value {
            v @ DataValue::Object(_) => {
                self.status.insert(name.to_string(), v);
                self
            }
            _ => panic!(
                "DataValueObjectBuilder.add_object_field value must be a DataValue::Object(..)"
            ),
        }
    }
}
