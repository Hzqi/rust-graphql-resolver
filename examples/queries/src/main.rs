use std::{
    array::IntoIter,
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    iter::FromIterator,
    rc::Rc,
};

use rust_graphql_resolver::{
    error::{Error, Result},
    execute,
    schema::{
        field::{
            ArgumentMap, CustomType, Field, FieldType, InputField, InputFieldType, QLEnum,
            QLEnumValue, QLInput,
        },
        query::{Query, QueryMap},
        resolve::{ApiResolveFunc, BoxedValue, DefaultFieldResolveFunc, QLApiParam, QLContext},
        Schema,
    },
    value::DataValue,
};

fn init_schema(datas: Vec<DataValue>) -> Schema {
    let mut schema = Schema {
        id: "queries_schema".to_string(),
        subscritions: None,
        mutations: None,
        queries: QueryMap::new(),
        objects: HashMap::new(),
        enums: HashMap::new(),
        inputs: HashMap::new(),
    };

    // Color Enum
    let color = QLEnum {
        name: "Color".to_string(),
        description: String::default(),
        values: vec![
            QLEnumValue {
                value: "Red".to_string(),
                description: String::default(),
            },
            QLEnumValue {
                value: "Yellow".to_string(),
                description: String::default(),
            },
            QLEnumValue {
                value: "Green".to_string(),
                description: String::default(),
            },
        ],
    };
    schema.enums.insert("Color".to_string(), Rc::new(color));

    // FullObject
    let full_object = CustomType {
        name: "FullObject".to_string(),
        description: String::default(),
        fields: BTreeMap::from_iter(IntoIter::new([
            ("id".to_string(), Field::basic_id()),
            ("str".to_string(), Field::basic_str()),
            ("int".to_string(), Field::basic_int()),
            ("float".to_string(), Field::basic_float()),
            ("bool".to_string(), Field::basic_bool()),
            ("datetime".to_string(), Field::basic_datetime()),
            (
                "color".to_string(),
                Field {
                    name: "color".to_string(),
                    field_type: FieldType::ReferenceEnum(
                        schema.enums.get(&"Color".to_string()).unwrap().clone(),
                    ),
                    description: String::default(),
                    resolve: Box::new(DefaultFieldResolveFunc),
                },
            ),
            (
                "extra".to_string(),
                Field {
                    name: "extra".to_string(),
                    description: String::default(),
                    field_type: FieldType::CustomType(CustomType {
                        name: "ExtraObject".to_string(),
                        description: String::default(),
                        fields: BTreeMap::from_iter(IntoIter::new([
                            ("col1".to_string(), Field::basic_str()),
                            ("col2".to_string(), Field::basic_int()),
                        ])),
                    }),
                    resolve: Box::new(
                        |context: HashMap<String, DataValue>,
                         _source,
                         _parameter|
                         -> Result<BoxedValue> {
                            println!("[debug] resolving extra...");
                            let col1 = match context.get(&"col1".to_string()) {
                                Some(r @ DataValue::String(_)) => r.to_owned(),
                                _ => DataValue::Null,
                            };
                            let col2 = match context.get(&"col2".to_string()) {
                                Some(r @ DataValue::Int(_)) => r.to_owned(),
                                _ => DataValue::Null,
                            };
                            Ok(DataValue::boxed_object(BTreeMap::from_iter(IntoIter::new(
                                [("col1".to_string(), col1), ("col2".to_string(), col2)],
                            ))))
                        },
                    ),
                },
            ),
        ])),
    };
    schema
        .objects
        .insert("FullObject".to_string(), Rc::new(RefCell::new(full_object)));

    // SearchFullObjectInput
    let search_full_object_input = QLInput {
        name: "SearchFullObjectInput".to_string(),
        description: String::default(),
        fields: BTreeMap::from_iter(IntoIter::new([
            ("id".to_string(), InputField::basic_id()),
            ("str".to_string(), InputField::basic_str()),
            ("int".to_string(), InputField::basic_int()),
            ("float".to_string(), InputField::basic_float()),
            ("bool".to_string(), InputField::basic_bool()),
            ("datetime".to_string(), InputField::basic_datetime()),
            (
                "color".to_string(),
                InputField {
                    name: "color".to_string(),
                    field_type: InputFieldType::ObjectFieldType(FieldType::ReferenceEnum(
                        schema.enums.get(&"Color".to_string()).unwrap().clone(),
                    )),
                    description: String::default(),
                },
            ),
        ])),
    };
    schema.inputs.insert(
        "SearchFullObjectInput".to_string(),
        Rc::new(RefCell::new(search_full_object_input)),
    );

    let query_full_object_datas = Query {
        field_type: FieldType::List(Box::new(FieldType::ReferenceCustom(Rc::downgrade(
            schema.objects.get(&"FullObject".to_string()).unwrap(),
        )))),
        arguments: ArgumentMap::default(),
        description: String::default(),
        resolve: create_func(datas.clone()),
    };
    schema
        .queries
        .insert("fullObjects".to_string(), query_full_object_datas);

    schema
}

fn create_func(datas: Vec<DataValue>) -> Box<dyn ApiResolveFunc> {
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

fn query_data(datas: Vec<DataValue>, map: &BTreeMap<String, DataValue>) -> Result<BoxedValue> {
    let target = datas
        .iter()
        .cloned()
        .filter(|dv| -> bool {
            match dv {
                DataValue::Object(m) => {
                    let mut p = true;
                    if map.contains_key("id") {
                        p = m.get("id").unwrap() == map.get("id").unwrap() && p;
                    }
                    if map.contains_key("str") {
                        p = m.get("str").unwrap() == map.get("str").unwrap() && p;
                    }
                    if map.contains_key("int") {
                        p = m.get("int").unwrap() == map.get("int").unwrap() && p;
                    }
                    if map.contains_key("float") {
                        p = m.get("float").unwrap() == map.get("float").unwrap() && p;
                    }
                    if map.contains_key("bool") {
                        p = m.get("bool").unwrap() == map.get("bool").unwrap() && p;
                    }
                    if map.contains_key("color") {
                        p = m.get("color").unwrap() == map.get("color").unwrap() && p;
                    }
                    p
                }
                _ => false,
            }
        })
        .collect();
    Ok(DataValue::boxed_list(target))
}

fn init_data() -> Vec<DataValue> {
    vec![
        DataValue::Object(BTreeMap::from_iter(IntoIter::new([
            ("id".to_string(), DataValue::ID("1".to_string())),
            ("str".to_string(), DataValue::String("str1".to_string())),
            ("int".to_string(), DataValue::Int(1)),
            ("float".to_string(), DataValue::Float(1.1)),
            ("bool".to_string(), DataValue::Boolean(true)),
            ("datetime".to_string(), DataValue::Null),
            ("color".to_string(), DataValue::String("Red".to_string())),
        ]))),
        DataValue::Object(BTreeMap::from_iter(IntoIter::new([
            ("id".to_string(), DataValue::ID("2".to_string())),
            ("str".to_string(), DataValue::String("str2".to_string())),
            ("int".to_string(), DataValue::Int(2)),
            ("float".to_string(), DataValue::Float(2.2)),
            ("bool".to_string(), DataValue::Boolean(false)),
            ("datetime".to_string(), DataValue::Null),
            ("color".to_string(), DataValue::String("Yellow".to_string())),
        ]))),
        DataValue::Object(BTreeMap::from_iter(IntoIter::new([
            ("id".to_string(), DataValue::ID("3".to_string())),
            ("str".to_string(), DataValue::String("str3".to_string())),
            ("int".to_string(), DataValue::Int(3)),
            ("float".to_string(), DataValue::Float(3.3)),
            ("bool".to_string(), DataValue::Boolean(true)),
            ("datetime".to_string(), DataValue::Null),
            ("color".to_string(), DataValue::String("Green".to_string())),
        ]))),
        DataValue::Object(BTreeMap::from_iter(IntoIter::new([
            ("id".to_string(), DataValue::ID("4".to_string())),
            ("str".to_string(), DataValue::String("str4".to_string())),
            ("int".to_string(), DataValue::Int(4)),
            ("float".to_string(), DataValue::Float(4.4)),
            ("bool".to_string(), DataValue::Boolean(false)),
            ("datetime".to_string(), DataValue::Null),
            ("color".to_string(), DataValue::String("Red".to_string())),
        ]))),
        DataValue::Object(BTreeMap::from_iter(IntoIter::new([
            ("id".to_string(), DataValue::ID("5".to_string())),
            ("str".to_string(), DataValue::String("str5".to_string())),
            ("int".to_string(), DataValue::Int(5)),
            ("float".to_string(), DataValue::Float(5.5)),
            ("bool".to_string(), DataValue::Boolean(true)),
            ("datetime".to_string(), DataValue::Null),
            ("color".to_string(), DataValue::String("Yellow".to_string())),
        ]))),
    ]
}

fn query() {
    let datas = init_data();
    let schema = init_schema(datas);

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
