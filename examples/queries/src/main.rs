use std::{
    array::IntoIter,
    collections::{BTreeMap, HashMap},
    iter::FromIterator,
};

use chrono::{DateTime, Utc};
use rust_graphql_resolver::{
    builder::{
        field::{CustomTypeBuilder, QLEnumBuilder, QLInputBuilder},
        query::QueryBuilder,
        schema::SchemaBuilder,
        value::DataValueObjectBuilder,
    },
    error::{BuildResult, Error, Result},
    execute,
    schema::{
        field::{CustomType, Field, FieldType, InputField, QLInput},
        query::Query,
        resolve::{ApiResolveFunc, BoxedValue, FieldResolveFunc, QLApiParam, QLContext},
        Schema,
    },
    value::{DataValue, ToDataValue},
};

#[derive(Debug, Clone)]
struct FullObject {
    id: String,
    str_value: String,
    int_value: i64,
    float_value: f64,
    bool_value: bool,
    datetime: DateTime<Utc>,
    color: String,
}

impl ToDataValue for FullObject {
    fn to_data_value(&self) -> DataValue {
        DataValueObjectBuilder::new()
            .add_id_field("id", self.id.to_owned())
            .add_str_field("str", self.str_value.to_owned())
            .add_int_field("int", self.int_value.to_owned())
            .add_float_field("float", self.float_value.to_owned())
            .add_bool_field("bool", self.bool_value.to_owned())
            .add_datetime_field("datetime", self.datetime.to_owned())
            .add_str_field("color", self.color.to_owned())
            .build()
    }
}

#[derive(Debug, Clone)]
struct ExtraObject {
    col1: String,
    col2: i64,
}

impl ToDataValue for ExtraObject {
    fn to_data_value(&self) -> DataValue {
        DataValueObjectBuilder::new()
            .add_str_field("col1", self.col1.to_owned())
            .add_int_field("col2", self.col2.to_owned())
            .build()
    }
}

fn build_schema(datas: Vec<FullObject>) -> BuildResult<Schema> {
    SchemaBuilder::new("queries_schema")
        .add_enum(
            QLEnumBuilder::new("Color")
                .add_value("Red")
                .add_value("Yellow")
                .add_value("Green")
                .build(),
        )
        .add_object(
            CustomTypeBuilder::new("ExtraObject")
                .add_field("col1", Field::basic_str())
                .add_field("col2", Field::basic_int())
                .build(),
        )
        .add_object_with_status(|sch| -> BuildResult<CustomType> {
            CustomTypeBuilder::new("FullObject")
                .add_field("id", Field::basic_id())
                .add_field("str", Field::basic_str())
                .add_field("int", Field::basic_int())
                .add_field("float", Field::basic_float())
                .add_field("bool", Field::basic_bool())
                .add_field("datetime", Field::basic_datetime())
                .add_field("color", Field::simple("color", sch.get_enum_type("Color")?))
                .add_field(
                    "extra",
                    Field::simple_with_resolve(
                        "extra",
                        sch.get_object_type("ExtraObject")?,
                        extra_resolve(),
                    ),
                )
                .build_ok()
        })?
        .add_input_object_with_status(|sch| -> BuildResult<QLInput> {
            QLInputBuilder::new("SearchFullObjectInput")
                .add_field("id", InputField::basic_id())
                .add_field("str", InputField::basic_str())
                .add_field("int", InputField::basic_int())
                .add_field("float", InputField::basic_float())
                .add_field("bool", InputField::basic_bool())
                .add_field("datetime", InputField::basic_datetime())
                .add_field(
                    "color",
                    InputField::simple("color", sch.get_enum_input_type("Color")?),
                )
                .build_ok()
        })?
        .add_query("fullObjects", |sch| -> BuildResult<Query> {
            // Type: [FullObject!]
            let field_type = FieldType::List(Box::new(FieldType::NonNullType(Box::new(
                sch.get_object_type("FullObject")?,
            ))));
            QueryBuilder::new()
                .set_type(field_type)
                .set_resolve(create_func(datas.clone()))
                .build()
        })?
        .build()
}

fn extra_resolve() -> Box<dyn FieldResolveFunc> {
    Box::new(
        |context: HashMap<String, DataValue>, _source, _parameter| -> Result<BoxedValue> {
            println!("[debug] resolving extra...");

            let col1 = match context.get(&"col1".to_string()) {
                Some(r @ DataValue::String(_)) => r.to_owned(),
                _ => DataValue::Null,
            };
            let col2 = match context.get(&"col2".to_string()) {
                Some(r @ DataValue::Int(_)) => r.to_owned(),
                _ => DataValue::Null,
            };

            Ok(DataValueObjectBuilder::new()
                .add_any_field("col1", col1)
                .add_any_field("col2", col2)
                .build()
                .to_boxed())
        },
    )
}

fn create_func(datas: Vec<FullObject>) -> Box<dyn ApiResolveFunc> {
    println!("[debug] invoke once...");
    Box::new(
        move |_context, parameter: QLApiParam| -> Result<BoxedValue> {
            println!("[debug] invoke every times...");
            let condition = parameter
                .arguments
                .get(&"condition".to_string())
                .ok_or(Error::NotFoundError("argument: 'condition'".to_string()))?;
            match condition {
                DataValue::Object(map) => query_data(datas.clone(), map),
                _ => Err(Error::DataTypeMisMatchError(
                    "input condition".to_string(),
                    "not match".to_string(),
                )),
            }
        },
    )
}

fn query_data(datas: Vec<FullObject>, map: &BTreeMap<String, DataValue>) -> Result<BoxedValue> {
    let target: Vec<FullObject> = datas
        .iter()
        .cloned()
        .filter(|dv| -> bool {
            let mut p = true;
            if let Some(DataValue::String(id)) = map.get("id") {
                p = dv.id == id.to_owned() && p;
            }
            if let Some(DataValue::String(s)) = map.get("str") {
                p = dv.str_value == s.to_owned() && p;
            }
            if let Some(DataValue::Int(i)) = map.get("int") {
                p = dv.int_value == i.to_owned() && p;
            }
            if let Some(DataValue::Float(f)) = map.get("flost") {
                p = dv.float_value == f.to_owned() && p;
            }
            if let Some(DataValue::Boolean(b)) = map.get("bool") {
                p = dv.bool_value == b.to_owned() && p;
            }
            if let Some(DataValue::String(color)) = map.get("color") {
                p = dv.color == color.to_owned() && p;
            }
            p
        })
        .collect();
    Ok(Box::new(target))
}

fn init_data() -> Vec<FullObject> {
    vec![
        FullObject {
            id: "1".to_string(),
            str_value: "str1".to_string(),
            int_value: 1,
            float_value: 1.1,
            bool_value: true,
            datetime: Utc::now(),
            color: "Red".to_string(),
        },
        FullObject {
            id: "2".to_string(),
            str_value: "str2".to_string(),
            int_value: 2,
            float_value: 2.2,
            bool_value: false,
            datetime: Utc::now(),
            color: "Yellow".to_string(),
        },
        FullObject {
            id: "3".to_string(),
            str_value: "str3".to_string(),
            int_value: 3,
            float_value: 3.3,
            bool_value: true,
            datetime: Utc::now(),
            color: "Green".to_string(),
        },
        FullObject {
            id: "4".to_string(),
            str_value: "str4".to_string(),
            int_value: 4,
            float_value: 4.4,
            bool_value: false,
            datetime: Utc::now(),
            color: "Red".to_string(),
        },
        FullObject {
            id: "5".to_string(),
            str_value: "str5".to_string(),
            int_value: 5,
            float_value: 5.5,
            bool_value: true,
            datetime: Utc::now(),
            color: "Yellow".to_string(),
        },
    ]
}

fn query() {
    let datas = init_data();
    let schema = build_schema(datas).unwrap();

    let context1 = QLContext::from_iter(IntoIter::new([
        (
            "col1".to_string(),
            DataValue::String("col1: strings".to_string()),
        ),
        ("col2".to_string(), DataValue::Int(1234)),
    ]));
    let request1 = r#"
    { 
        fullObjects(condition: {}) { 
            id 
            str 
            int 
            float 
            bool 
            datetime 
            color 
            extra { 
                col1 
                col2 
            } 
        } 
    }
    "#;
    let result1 = execute(context1, request1, &schema).unwrap();
    println!(
        "result: {}",
        serde_json::ser::to_string_pretty(&result1).unwrap()
    );

    let context2 = QLContext::from_iter(IntoIter::new([
        (
            "col1".to_string(),
            DataValue::String("col1: stringsxxx".to_string()),
        ),
        ("col2".to_string(), DataValue::Int(1234)),
    ]));
    let request2 = r#"
    { 
        fullObjects(condition: {color: Red}) { 
            id 
            str 
            int 
            float 
            bool 
            datetime 
            color 
            extra { 
                col1 
                col2 
            } 
        } 
    }
    "#;
    let result2 = execute(context2, request2, &schema).unwrap();
    println!(
        "result: {}",
        serde_json::ser::to_string_pretty(&result2).unwrap()
    );

    let request3 = r#"
    { 
        fullObjects(condition: {bool: true}) { 
            id 
            int 
            float 
            bool 
            color 
        } 
    }
    "#;
    let result3 = execute(QLContext::default(), request3, &schema).unwrap();
    println!(
        "result: {}",
        serde_json::ser::to_string_pretty(&result3).unwrap()
    );
}

fn main() {
    query();
    println!("job done.")
}
