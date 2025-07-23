

use crate::variables::GJFloat;
use crate::variables::GJInteger;
use polars::datatypes::{AnyValue, *};

#[derive(Debug, Clone)]
pub enum CotwinValueTypes<'a> {
    GJF(GJFloat),
    GJI(GJInteger),
    PAV(AnyValue<'a>) //PolarsAnyValue,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cotwin_value_types() {
        let some_value = CotwinValueTypes::PAV(AnyValue::Int64(2));

        println!("{:?}", some_value);
    }
}
