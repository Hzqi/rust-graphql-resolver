use std::{
    array::IntoIter,
    collections::{BTreeMap, HashMap},
    iter::FromIterator,
};

use rust_graphql_resolver::{
    error::Result,
    execute,
    schema::{
        field::{ArgumentMap, CustomType, Field, FieldType},
        query::{Query, QueryMap},
        resolve::QLContext,
        Schema,
    },
    value::DataValue,
};

fn main() {
    let schema = Schema {
        id: "hello_world_schema".to_string(),
        subscritions: None,
        mutations: None,
        queries: QueryMap::from_iter(IntoIter::new([(
            // query: foo { name, foo }
            "foo".to_string(),
            Query {
                field_type: FieldType::CustomType(CustomType {
                    name: "Foo".to_string(),
                    description: String::default(),
                    // fields: {name, foo}
                    fields: BTreeMap::from_iter(IntoIter::new([
                        // string field: "name"
                        ("name".to_string(), Field::basic_str()),
                        // string field: "foo"
                        ("foo".to_string(), Field::basic_str()),
                    ])),
                }),
                arguments: ArgumentMap::default(),
                description: String::default(),
                resolve: Box::new(|_context, _param| -> Result<DataValue> {
                    // result: { "name": "foo_name", "foo": "hello world" }
                    Ok(DataValue::Object(BTreeMap::from_iter(IntoIter::new([
                        (
                            "name".to_string(),
                            DataValue::String("foo_name".to_string()),
                        ),
                        (
                            "foo".to_string(),
                            DataValue::String("hello world".to_string()),
                        ),
                    ]))))
                }),
            },
        )])),
        enums: HashMap::default(),
        inputs: HashMap::default(),
        objects: HashMap::default(),
    };
    {
        let request = r#"
        { 
            foo { 
                name, 
                foo 
            } 
        }
        "#;
        let result = execute(QLContext::default(), request, &schema).unwrap();
        println!(
            "result: {}",
            serde_json::ser::to_string_pretty(&result).unwrap()
        );
    }

    println!("job done.")
}
