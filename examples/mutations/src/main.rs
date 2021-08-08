use std::{
    array::IntoIter,
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    iter::FromIterator,
    rc::Rc,
};

use rust_graphql_resolver::{
    builder::{
        field::CustomTypeBuilder, mutation::MutationBuilder, query::QueryBuilder,
        schema::SchemaBuilder,
    },
    error::{BuildResult, Error, Result},
    execute,
    schema::{
        field::{Field, InputFieldType},
        mutation::Mutation,
        query::Query,
        resolve::{ApiResolveFunc, BoxedValue, QLApiParam, QLContext},
        Schema,
    },
    value::DataValue,
};

type Storage = Rc<RefCell<HashMap<String, DataValue>>>;

fn build_schema(datas: Storage) -> BuildResult<Schema> {
    SchemaBuilder::new("mutations_schema")
        .add_object(
            CustomTypeBuilder::new("SampleObject")
                .add_field("id", Field::basic_id())
                .add_field("foo", Field::basic_str())
                .build(),
        )
        .add_query("sample", |sch| -> BuildResult<Query> {
            let field_type = sch.get_object_type("SampleObject")?;
            QueryBuilder::new()
                .set_type(field_type)
                .set_description("query a data by id")
                .add_argument("id", InputFieldType::basic_id())
                .set_resolve(create_query_func(datas.clone()))
                .build()
        })?
        .add_mutation("addSample", |sch| -> BuildResult<Mutation> {
            let field_type = sch.get_object_type("SampleObject")?;
            MutationBuilder::new()
                .set_type(field_type)
                .set_description("add a new data")
                .add_argument("id", InputFieldType::basic_id())
                .add_argument("foo", InputFieldType::basic_str())
                .set_resolve(create_mutation_func(datas.clone()))
                .build()
        })?
        .build()
}

fn create_query_func(datas: Storage) -> Box<dyn ApiResolveFunc> {
    println!("[debug] query api building...");
    let result = Box::new(
        move |_context: &mut QLContext, parameter: &QLApiParam| -> Result<BoxedValue> {
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
                Some(res) => Ok(Box::new(res.to_owned())),
                None => Ok(Box::new(DataValue::Null)),
            }
        },
    );
    println!("[debug] query api built");
    result
}

fn create_mutation_func(datas: Storage) -> Box<dyn ApiResolveFunc> {
    println!("[debug] muttaion api building...");
    let result = Box::new(
        move |_context: &mut QLContext, parameter: &QLApiParam| -> Result<BoxedValue> {
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
                    Ok(Box::new(v))
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
    let schema = build_schema(datas).unwrap();
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
    let result1 = execute(context.clone(), request1, &schema, None).unwrap();
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
    let result2 = execute(context.clone(), request2, &schema, None).unwrap();
    println!(
        "result: {}",
        serde_json::to_string_pretty(&result2).unwrap()
    );
}

fn main() {
    mutation_and_query();
    println!("job done.")
}
