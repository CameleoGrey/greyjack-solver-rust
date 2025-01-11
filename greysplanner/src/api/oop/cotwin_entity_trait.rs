
use std::collections::HashMap;
use crate::api::oop::cotwin_value_types::CotwinValueTypes;

pub trait CotwinEntityTrait {
    fn to_hash_map(&self) -> HashMap<String, CotwinValueTypes>;
}