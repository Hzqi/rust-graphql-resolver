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
        field::{CustomType, Field, FieldType, InputFieldType, StaticType},
        mutation::Mutation,
        query::{Query, QueryMap},
        resolve::{ApiResolveFunc, QLApiParam, QLContext},
        Schema,
    },
    value::DataValue,
};

type Storage = Rc<RefCell<HashMap<String, DataValue>>>;

fn init_schema(datas: Storage) -> Schema {
    let mut schema = Schema {
        id: "mutations_schema".to_string(),
        queries: QueryMap::new(),
        mutations: None,
        subscritions: None,
        objects: HashMap::default(),
        enums: HashMap::default(),
        inputs: HashMap::default(),
    };

    let sample_object = CustomType {
        name: "SampleObject".to_string(),
        description: String::default(),
        fields: BTreeMap::from_iter(IntoIter::new([
            ("id".to_string(), Field::basic_id()),
            ("foo".to_string(), Field::basic_str()),
        ])),
    };
    schema.objects.insert(
        "SampleObject".to_string(),
        Rc::new(RefCell::new(sample_object)),
    );

    let query_by_id = Query {
        field_type: FieldType::ReferenceCustom(Rc::downgrade(
            schema.objects.get("SampleObject").unwrap(),
        )),
        arguments: BTreeMap::from_iter(IntoIter::new([(
            "id".to_string(),
            InputFieldType::ObjectFieldType(FieldType::StaticType(StaticType::String)),
        )])),
        description: "query a data by id".to_string(),
        resolve: create_query_func(datas.clone()),
    };
    schema.queries.insert("sample".to_string(), query_by_id);

    let add_data_mutation = Mutation {
        field_type: FieldType::ReferenceCustom(Rc::downgrade(
            schema.objects.get("SampleObject").unwrap(),
        )),
        arguments: BTreeMap::from_iter(IntoIter::new([
            (
                "id".to_string(),
                InputFieldType::ObjectFieldType(FieldType::StaticType(StaticType::String)),
            ),
            (
                "foo".to_string(),
                InputFieldType::ObjectFieldType(FieldType::StaticType(StaticType::String)),
            ),
        ])),
        description: "add a new data".to_string(),
        resolve: create_mutation_func(datas),
    };

    let mut mutations = HashMap::new();
    mutations.insert("addSample".to_string(), add_data_mutation);
    schema.mutations = Some(mutations);

    schema
}

fn create_query_func(datas: Storage) -> Box<dyn ApiResolveFunc> {
    println!("[debug] query api building...");
    let result = Box::new(
        move |_context, parameter: QLApiParam| -> Result<DataValue> {
            let dv = parameter
                .arguments
                .get(&"id".to_string())
                .ok_or(Error::NotFoundError("argument: 'id'".to_string()))?;
            let id = match dv {
                DataValue::String(id) => id,
                _ => {
                    return Err(Error::DataTypeMisMatchError(
                        "String/ID".to_string(),
                        "unchecked".to_string(),
                    ))
                }
            };
            match datas.borrow().get(id) {
                Some(res) => Ok(res.to_owned()),
                None => Ok(DataValue::Null),
            }
        },
    );
    println!("[debug] query api built");
    result
}

fn create_mutation_func(datas: Storage) -> Box<dyn ApiResolveFunc> {
    println!("[debug] muttaion api building...");
    let result = Box::new(
        move |_context, parameter: QLApiParam| -> Result<DataValue> {
            let id = parameter
                .arguments
                .get(&"id".to_string())
                .ok_or(Error::NotFoundError("argument: 'id'".to_string()))?;
            let foo = parameter
                .arguments
                .get(&"foo".to_string())
                .ok_or(Error::NotFoundError("argument: 'foo'".to_string()))?;
            match (id, foo) {
                (DataValue::String(i), DataValue::String(f)) => {
                    let v = DataValue::Object(BTreeMap::from_iter(IntoIter::new([
                        ("id".to_string(), DataValue::ID(i.to_owned())),
                        ("foo".to_string(), DataValue::String(f.to_owned())),
                    ])));
                    datas.borrow_mut().insert(i.to_owned(), v.clone());
                    Ok(v)
                }
                _ => Err(Error::DataTypeMisMatchError(
                    "(ID, String)".to_string(),
                    "others case".to_string(),
                )),
            }
        },
    );
    println!("[debug] mutation api built");
    result
}

fn mutation_and_query() {
    let datas = Rc::new(RefCell::new(HashMap::new()));
    let schema = init_schema(datas);
    let context = QLContext::default();

    let request1 = r#"
    mutation {
        add1:addSample(id: "1", foo: "foo1"){
            id
            foo
        }
        add2:addSample(id: "2", foo: "foo2"){
            id
            foo
        }
    }
    "#;
    let result1 = execute(context.clone(), request1, &schema).unwrap();
    println!(
        "result: {}",
        serde_json::to_string_pretty(&result1).unwrap()
    );

    let request2 = r#"
    {
        get1:sample(id: "1"){
            id
            foo
        }
        get2:sample(id: "2"){
            id
            foo
        }
        get3:sample(id: "3"){
            id
            foo
        }
    }
    "#;
    let result2 = execute(context.clone(), request2, &schema).unwrap();
    println!(
        "result: {}",
        serde_json::to_string_pretty(&result2).unwrap()
    );
}

fn main() {
    mutation_and_query();
    println!("job done.")
}
