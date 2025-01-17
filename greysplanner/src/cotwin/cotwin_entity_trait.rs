
use std::collections::HashMap;
use crate::cotwin::CotwinValueTypes;

pub trait CotwinEntityTrait {
    fn to_hash_map(&self) -> HashMap<String, CotwinValueTypes>;
}