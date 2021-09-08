use self::{
    root::{MutationMap, QueryMap},
    storage::TypeStorage,
};

pub mod executor;
pub mod field;
pub mod resolve;
pub mod root;
pub mod storage;
pub mod types;

/// NotSupported
#[derive(Clone, Debug)]
pub struct NotSupported;

/// Schema: The main logic struct
/// * query: definition for query apis, and resolve by request
/// * mutation: definition for mutation apis, and resolve by request
/// * subscrition: todo
#[derive(Clone, Debug)]
pub struct Schema {
    /// for tracting instance's id
    pub(crate) id: String,
    pub(crate) queries: QueryMap,
    pub(crate) mutations: Option<MutationMap>,
    /// TODO: Subscrition
    pub(crate) subscritions: Option<NotSupported>,

    pub(crate) type_storage: TypeStorage,
}

// TODO: execute_introspection()
// schema 只需要生成一个磨人的query字段"__schema"即可，
// 其字段结构就包含了所有内省的字段结构，而每个的处理方式
// 就是通过其字段类型/root接口的execute_introspection()处理所得
