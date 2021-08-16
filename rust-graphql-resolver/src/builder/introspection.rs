use crate::{schema::Schema, value::DataValue};

use super::value::DataValueObjectBuilder;

/// Create the data of schema introspection
pub(crate) fn create_introspection_data(schema: &Schema) -> DataValue {
    let desc = match schema.description.clone() {
        Some(d) => d,
        None => String::default(),
    };

    let mut builder = DataValueObjectBuilder::new().add_str_field("description", desc);

    if schema.queries.is_empty() {
        builder = builder.add_null_field("queryType");
    } else {
        builder = builder.add_object_field(
            "queryType",
            DataValueObjectBuilder::new()
                .add_str_field("kind", "OBJECT".to_string())
                .add_str_field("name", "Query".to_string())
                .add_str_field("description", String::default())
                .build(),
        );
    }

    if schema.mutations.is_none() {
        builder = builder.add_null_field("mutationType");
    } else {
        builder = builder.add_object_field(
            "mutationType",
            DataValueObjectBuilder::new()
                .add_str_field("kind", "OBJECT".to_string())
                .add_str_field("name", "Mutation".to_string())
                .add_str_field("description", String::default())
                .build(),
        );
    }

    builder = builder.add_null_field("subscriptionType");

    let mut types = vec![];
    types.append(create_built_in_types());
    types.append(create_storaged_enums(schema));
    types.append(create_storaged_objects(schema));
    types.append(create_storaged_inputs(schema));
    types.append(create_api_directly_types(schema));

    builder = builder.add_list_field("types", types);

    builder.build()
}
