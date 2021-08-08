use std::{array::IntoIter, collections::BTreeMap, iter::FromIterator};

use macros::GraphQLDataValue;
use rust_graphql_resolver::value::{DataValue, ToDataValue};

#[derive(Debug, Clone, GraphQLDataValue)]
struct HelloWorld {
    hello: String,
    greeting: String,
}

#[test]
fn test_generate_datavalue_derive() {
    let h = HelloWorld {
        hello: "hello".to_string(),
        greeting: "rust-graphql-resolver".to_string(),
    };
    let dv = h.to_data_value();
    assert_eq!(
        dv,
        DataValue::Object(BTreeMap::from_iter(IntoIter::new([
            ("hello".to_string(), DataValue::String("hello".to_string())),
            (
                "greeting".to_string(),
                DataValue::String("rust-graphql-resolver".to_string())
            ),
        ])))
    )
}
