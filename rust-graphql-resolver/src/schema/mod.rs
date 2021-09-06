pub mod field;
pub mod mutation;
pub mod query;
pub mod resolve;
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
