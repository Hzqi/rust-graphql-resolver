use std::collections::BTreeMap;

use crate::{
    error::BuildResult,
    schema::field::{
        CustomType, Field, FieldType, InputField, InputFieldType, QLEnum, QLEnumValue, QLInput,
    },
};

pub struct CustomTypeBuilder {
    status: CustomType,
}

impl CustomTypeBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            status: CustomType {
                name: name.to_string(),
                fields: BTreeMap::new(),
                description: String::default(),
            },
        }
    }

    pub fn build(self) -> CustomType {
        self.status
    }

    pub fn build_ok(self) -> BuildResult<CustomType> {
        Ok(self.status)
    }

    pub fn build_field(self) -> FieldType {
        FieldType::CustomType(self.status)
    }

    pub fn add_field(mut self, name: &str, field: Field) -> Self {
        self.status.fields.insert(name.to_string(), field);
        self
    }

    pub fn set_description(mut self, desc: &str) -> Self {
        self.status.description = desc.to_string();
        self
    }
}

pub struct QLEnumBuilder {
    status: QLEnum,
}

impl QLEnumBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            status: QLEnum {
                name: name.to_string(),
                description: String::default(),
                values: vec![],
            },
        }
    }

    pub fn build(self) -> QLEnum {
        self.status
    }

    pub fn build_ok(self) -> BuildResult<QLEnum> {
        Ok(self.status)
    }

    pub fn add_value(mut self, value: &str) -> Self {
        self.status.values.push(QLEnumValue {
            value: value.to_string(),
            description: String::default(),
        });
        self
    }

    pub fn add_value_with_desc(mut self, value: &str, desc: &str) -> Self {
        self.status.values.push(QLEnumValue {
            value: value.to_string(),
            description: desc.to_string(),
        });
        self
    }

    pub fn set_description(mut self, desc: &str) -> Self {
        self.status.description = desc.to_string();
        self
    }
}

pub struct QLInputBuilder {
    status: QLInput,
}

impl QLInputBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            status: QLInput {
                name: name.to_string(),
                fields: BTreeMap::new(),
                description: String::default(),
            },
        }
    }

    pub fn build(self) -> QLInput {
        self.status
    }

    pub fn build_field(self) -> InputFieldType {
        InputFieldType::QLInput(self.status)
    }

    pub fn build_ok(self) -> BuildResult<QLInput> {
        Ok(self.build())
    }

    pub fn add_field(mut self, name: &str, field: InputField) -> Self {
        self.status.fields.insert(name.to_string(), field);
        self
    }

    pub fn set_description(mut self, desc: &str) -> Self {
        self.status.description = desc.to_string();
        self
    }
}
