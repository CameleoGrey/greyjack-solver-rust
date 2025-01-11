
use super::gp_float_var::GPFloatVar;
use super::gp_integer_var::GPIntegerVar;

#[derive(Debug, Clone)]
pub enum PlanningVariablesTypes {
    GPFloatVar(GPFloatVar),
    GPIntegerVar(GPIntegerVar),
}