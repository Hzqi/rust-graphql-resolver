use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    error::{BuildError, BuildResult},
    schema::{
        field::{CustomType, FieldType, InputFieldType, QLEnum, QLInput},
        mutation::{Mutation, MutationMap},
        query::{Query, QueryMap},
        Schema,
    },
};

/// For building a Schema using interfaces
pub struct SchemaBuilder {
    status: Schema,
}

impl SchemaBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            status: Schema {
                id: name.to_string(),
                queries: QueryMap::default(),
                mutations: None,
                subscritions: None,
                objects: HashMap::default(),
                enums: HashMap::default(),
                inputs: HashMap::default(),
            },
        }
    }

    /// Final action to build the Schema
    pub fn build(self) -> BuildResult<Schema> {
        Ok(self.status)
    }

    /// Add a query to Schema
    pub fn add_query<F>(mut self, name: &str, query_func: F) -> BuildResult<Self>
    where
        F: Fn(&Self) -> BuildResult<Query>,
    {
        let query = query_func(&self)?;
        self.status.queries.insert(name.to_string(), query);
        Ok(self)
    }

    /// Add a mutation to Schema
    pub fn add_mutation<F>(mut self, name: &str, mutation_func: F) -> BuildResult<Self>
    where
        F: Fn(&Self) -> BuildResult<Mutation>,
    {
        let mut mutations = if self.status.mutations.is_none() {
            MutationMap::new()
        } else {
            self.status.mutations.clone().unwrap()
        };
        let mutation = mutation_func(&self)?;
        mutations.insert(name.to_string(), mutation);
        self.status.mutations = Some(mutations);
        Ok(self)
    }

    /// Add a object (CustomType) to Schema for reference use
    pub fn add_object(mut self, custom_type: CustomType) -> Self {
        self.status
            .objects
            .insert(custom_type.name.clone(), Rc::new(RefCell::new(custom_type)));
        self
    }

    pub fn add_object_with_status<F>(mut self, custom_type_func: F) -> BuildResult<Self>
    where
        F: Fn(&Self) -> BuildResult<CustomType>,
    {
        let custom_type = custom_type_func(&self)?;
        self.status
            .objects
            .insert(custom_type.name.clone(), Rc::new(RefCell::new(custom_type)));
        Ok(self)
    }

    /// Add a enum type to Schema for reference use
    pub fn add_enum(mut self, enum_type: QLEnum) -> Self {
        self.status
            .enums
            .insert(enum_type.name.clone(), Rc::new(enum_type));
        self
    }

    /// Add a input object (QLInput type) to Schema for reference use
    pub fn add_input_object(mut self, input_object: QLInput) -> Self {
        self.status.inputs.insert(
            input_object.name.clone(),
            Rc::new(RefCell::new(input_object)),
        );
        self
    }

    pub fn add_input_object_with_status<F>(mut self, input_object_func: F) -> BuildResult<Self>
    where
        F: Fn(&Self) -> BuildResult<QLInput>,
    {
        let input_type = input_object_func(&self)?;
        self.status
            .inputs
            .insert(input_type.name.clone(), Rc::new(RefCell::new(input_type)));
        Ok(self)
    }

    /// Get the reference object type
    pub fn get_object_type(&self, name: &str) -> BuildResult<FieldType> {
        let ref_rc = self
            .status
            .objects
            .get(&name.to_string())
            .ok_or(BuildError::NoSuchObjectType(name.to_string()))?;
        Ok(FieldType::ReferenceCustom(Rc::downgrade(ref_rc)))
    }

    /// Get the reference enum type
    pub fn get_enum_type(&self, name: &str) -> BuildResult<FieldType> {
        let rc = self
            .status
            .enums
            .get(&name.to_string())
            .ok_or(BuildError::NoSuchEnumType(name.to_string()))?
            .clone();
        Ok(FieldType::ReferenceEnum(rc))
    }

    /// Get the reference enum type as input field
    pub fn get_enum_input_type(&self, name: &str) -> BuildResult<InputFieldType> {
        let rc = self
            .status
            .enums
            .get(&name.to_string())
            .ok_or(BuildError::NoSuchEnumType(name.to_string()))?
            .clone();
        Ok(InputFieldType::ReferenceEnum(rc))
    }

    /// Get the reference input type
    pub fn get_input_type(&self, name: &str) -> BuildResult<InputFieldType> {
        let ref_rc = self
            .status
            .inputs
            .get(&name.to_string())
            .ok_or(BuildError::NoSuchObjectType(name.to_string()))?;
        Ok(InputFieldType::ReferenceInput(Rc::downgrade(ref_rc)))
    }
}
