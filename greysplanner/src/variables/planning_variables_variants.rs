
use super::GPFloatVar;
use super::GPIntegerVar;

#[derive(Debug, Clone)]
pub enum PlanningVariablesVariants {
    GPFloatVar(GPFloatVar),
    GPIntegerVar(GPIntegerVar),
}