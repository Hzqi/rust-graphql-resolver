use std::{array::IntoIter, collections::BTreeMap, iter::FromIterator};

use rust_graphql_resolver::{
    builder::{field::CustomTypeBuilder, query::QueryBuilder, schema::SchemaBuilder},
    error::{BuildResult, Result},
    execute,
    schema::{
        field::Field,
        query::Query,
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

fn build_schema() -> BuildResult<Schema> {
    SchemaBuilder::new("hello_worlld_schema")
        // query: helloWorld { hello, greeting }
        .add_query("helloWorld", |_sch| -> BuildResult<Query> {
            let field_type = CustomTypeBuilder::new("HelloWorld")
                // fields: { hello, greeting }
                .add_field("hello", Field::basic_str())
                .add_field("greeting", Field::basic_str())
                .build_field();
            QueryBuilder::new()
                .set_type(field_type)
                .set_resolve(Box::new(|_context, _param| -> Result<BoxedValue> {
                    // result: { "name": "foo_name", "foo": "hello world" }
                    Ok(Box::new(HelloWorld {
                        hello: "rust".to_string(),
                        greeting: "graphql-resolver".to_string(),
                    }))
                }))
                .build()
        })?
        .build()
}

fn main() {
    let schema = build_schema().unwrap();
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
