use crate::{
    error::BuildResult,
    schema::{
        field::{ArgumentMap, FieldType, InputFieldType, StaticType},
        mutation::Mutation,
        resolve::{ApiResolveFunc, DefaultApiResolveFunc},
    },
};

pub struct MutationBuilder {
    status: Mutation,
}

impl MutationBuilder {
    pub fn new() -> Self {
        Self {
            status: uninitialized_query(),
        }
    }

    pub fn build(self) -> BuildResult<Mutation> {
        Ok(self.status)
    }

    pub fn set_type(mut self, field_type: FieldType) -> Self {
        self.status.field_type = field_type;
        self
    }

    pub fn add_argument(mut self, name: &str, argument_type: InputFieldType) -> Self {
        self.status
            .arguments
            .insert(name.to_string(), argument_type);
        self
    }

    pub fn set_description(mut self, desc: &str) -> Self {
        self.status.description = desc.to_string();
        self
    }

    pub fn set_resolve(mut self, resolve: Box<dyn ApiResolveFunc>) -> Self {
        self.status.resolve = resolve;
        self
    }
}

fn uninitialized_query() -> Mutation {
    Mutation {
        field_type: FieldType::StaticType(StaticType::Boolean),
        arguments: ArgumentMap::default(),
        description: String::default(),
        resolve: Box::new(DefaultApiResolveFunc),
    }
}
