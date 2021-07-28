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
        resolve::{BoxedValue, QLContext},
        Schema,
    },
    value::{DataValue, ToDataValue},
};

#[derive(Debug, Clone)]
struct HelloWorld {
    hello: String,
    greeting: String,
}

impl ToDataValue for HelloWorld {
    fn to_data_value(&self) -> DataValue {
        DataValue::Object(BTreeMap::from_iter(IntoIter::new([
            ("hello".to_string(), self.hello.to_data_value()),
            ("greeting".to_string(), self.greeting.to_data_value()),
        ])))
    }
}

fn main() {
    let schema = Schema {
        id: "hello_world_schema".to_string(),
        subscritions: None,
        mutations: None,
        queries: QueryMap::from_iter(IntoIter::new([(
            // query: helloWorld { hello, greeting }
            "helloWorld".to_string(),
            Query {
                field_type: FieldType::CustomType(CustomType {
                    name: "helloWorld".to_string(),
                    description: String::default(),
                    // fields: { hello, greeting }
                    fields: BTreeMap::from_iter(IntoIter::new([
                        // string field: "hello"
                        ("hello".to_string(), Field::basic_str()),
                        // string field: "greeting"
                        ("greeting".to_string(), Field::basic_str()),
                    ])),
                }),
                arguments: ArgumentMap::default(),
                description: String::default(),
                resolve: Box::new(|_context, _param| -> Result<BoxedValue> {
                    // result: { "name": "foo_name", "foo": "hello world" }
                    Ok(Box::new(HelloWorld {
                        hello: "rust".to_string(),
                        greeting: "graphql-resolver".to_string(),
                    }))
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
            helloWorld { 
                hello, 
                greeting 
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
