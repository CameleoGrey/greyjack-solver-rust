
use std::collections::HashMap;
use crate::cotwin::CotwinValueTypes;

pub trait CotwinEntityTrait {
    fn to_vec(&self) -> Vec<(String, CotwinValueTypes)>;
}