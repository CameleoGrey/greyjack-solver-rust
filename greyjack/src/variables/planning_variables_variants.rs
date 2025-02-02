
use super::GJFloat;
use super::GJInteger;

#[derive(Debug, Clone)]
pub enum PlanningVariablesVariants {
    GJF(GJFloat),
    GJI(GJInteger),
}