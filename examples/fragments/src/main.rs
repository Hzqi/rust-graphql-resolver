use rust_graphql_resolver::{
    builder::{field::CustomTypeBuilder, query::QueryBuilder, schema::SchemaBuilder},
    error::{BuildResult, Result},
    execute,
    macros::GraphQLDataValue,
    schema::{
        field::Field,
        query::Query,
        resolve::{BoxedValue, QLApiParam, QLContext},
        Schema,
    },
};

#[derive(Debug, Clone, GraphQLDataValue)]
struct FooBar {
    id: String,
    name: String,
    age: i64,
    is_male: bool,
}

fn build_schema() -> BuildResult<Schema> {
    SchemaBuilder::new("fragment_schema")
        .add_query("foobar", |_sch| -> BuildResult<Query> {
            let field_type = CustomTypeBuilder::new("FooBar")
                .add_field("id", Field::basic_id())
                .add_field("name", Field::basic_str())
                .add_field("age", Field::basic_int())
                .add_field("is_male", Field::basic_bool())
                .build_field();
            QueryBuilder::new()
                .set_type(field_type)
                .set_resolve(Box::new(
                    |_context: &mut QLContext, _param: &QLApiParam| -> Result<BoxedValue> {
                        // result: { "name": "foo_name", "foo": "hello world" }
                        Ok(Box::new(FooBar {
                            id: "id001".to_string(),
                            name: "foo_bar_name".to_string(),
                            age: 1,
                            is_male: true,
                        }))
                    },
                ))
                .build()
        })?
        .build()
}

fn main() {
    let schema = build_schema().unwrap();
    let request1 = r#"
        { 
            foobar { 
                ...TheFooBar
            } 
        }
        fragment TheFooBar on FooBar {
            id
            name
            age
            is_male
        }
        "#;
    let result1 = execute(QLContext::default(), request1, &schema, None).unwrap();
    println!(
        "result1: {}",
        serde_json::ser::to_string_pretty(&result1).unwrap()
    );

    let request2 = r#"
    { 
        foobar { 
            id
            name
            ...leftoverFooBar
        } 
    }
    fragment leftoverFooBar on FooBar {
        name
        age
        is_male
    }
    "#;
    let result2 = execute(QLContext::default(), request2, &schema, None).unwrap();
    println!(
        "result2: {}",
        serde_json::ser::to_string_pretty(&result2).unwrap()
    );

    println!("job done.")
}
