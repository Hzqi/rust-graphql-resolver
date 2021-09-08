use std::fmt::Debug;

use super::{
    resolve::FieldResolveFunc,
    types::{FieldType, InputType},
};

/// Field
#[derive(Clone, Debug)]
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
    pub description: String,
    pub resolve: FieldResolve,
}

/// FieldResolve
#[derive(Clone)]
pub enum FieldResolve {
    DefaultResolve,
    CustomResolve(Box<dyn FieldResolveFunc>),
}

impl Debug for FieldResolve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DefaultResolve => write!(f, "DefaultResolve"),
            Self::CustomResolve(_) => write!(f, "CustomResolve(...)"),
        }
    }
}

/// InputField
#[derive(Clone, Debug)]
pub struct InputField {
    pub name: String,
    pub field_type: InputType,
    pub description: String,
}
