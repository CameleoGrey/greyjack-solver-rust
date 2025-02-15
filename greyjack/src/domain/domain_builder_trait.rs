
use serde_json::Value;

pub trait DomainBuilderTrait<DomainType>
where 
    DomainType: Clone + Send {
    
    // Default function to build domain model without using existing solution
    fn build_domain_from_scratch(&self) -> DomainType;

    // For multistage solving or extracting human-understandable representation for
    // post-solving actions (for example: print metrics, check correctness of solution,
    // serializing to JSON the whole doma and sending to another service).
    // means raw solution JSON from Solver::solve()
    fn build_from_solution(&self, solution: &Value) -> DomainType;

    // For multistage solving cases, when you need to take solution
    // from the N-1 stage, build domain from solution, then change that domain
    // by some logic (for example: freeze some variables to prevent changes in the Nth stage)
    // and then use it as initial solution.
    // Suggest just to use return domain.clone() in most cases due to the described logic above.
    fn build_from_domain(&self, domain: &DomainType) -> DomainType {
        return domain.clone();
    }
}
