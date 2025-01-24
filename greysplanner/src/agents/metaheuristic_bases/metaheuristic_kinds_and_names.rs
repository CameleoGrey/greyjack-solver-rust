

#[derive(Clone)]
pub enum MetaheuristicKind {
    Population,
    LocalSearch
}

#[derive(Clone)]
pub enum MetaheuristicNames {
    GeneticAlgorithm,
    TabuSearch,
    LateAcceptance,
}