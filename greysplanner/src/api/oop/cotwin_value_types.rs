

use crate::core::variables::gp_float_var::GPFloatVar;
use crate::core::variables::gp_integer_var::GPIntegerVar;
use polars::datatypes::AnyValue;

#[derive(Debug, Clone)]
pub enum CotwinValueTypes<'a> {
    GPFloatVar(GPFloatVar),
    GPIntegerVar(GPIntegerVar),
    PolarsAnyValue(AnyValue<'a>)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cotwin_value_types() {
        let some_value = CotwinValueTypes::PolarsAnyValue(AnyValue::Int64(2));

        println!("{:?}", some_value);
    }
}
