use std::collections::HashMap;

use super::types::{DirectEnum, DirectInputObject, DirectObject};

/// TypeStorage
#[derive(Clone, Debug)]
pub struct TypeStorage {
    pub(crate) enums: HashMap<String, DirectEnum>,
    pub(crate) objects: HashMap<String, DirectObject>,
    pub(crate) inputs: HashMap<String, DirectInputObject>,
    // TODO: interface and union
    // pub(crate) interfaces: HashMap<String, DirectInterface>,
    // pub(crate) unions: HashMap<String, DirectUnion>,
}
