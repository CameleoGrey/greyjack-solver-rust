
use serde_json::Value;

#[derive(Clone, Debug)]
pub enum InitialSolutionVariants<DomainType>
where DomainType: Clone + Send {
    CotwinValuesVector(Value),
    DomainObject(DomainType)
}