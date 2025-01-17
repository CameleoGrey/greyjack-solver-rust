
use super::GPFloatVar;
use super::GPIntegerVar;

#[derive(Debug, Clone)]
pub enum PlanningVariablesTypes {
    GPFloatVar(GPFloatVar),
    GPIntegerVar(GPIntegerVar),
}