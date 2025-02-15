
mod cotwin;
mod score;
mod persistence;
mod observers_examples;
mod domain;

use greyjack::score_calculation::scores::SimpleScore;
use greyjack::domain::DomainBuilderTrait;
use greyjack::cotwin::{CotwinBuilderTrait};
use greyjack::solver::{ObserverTrait, Solver, SolverLoggingLevels};
use greyjack::agents::{GeneticAlgorithm, LateAcceptance, TabuSearch};
use greyjack::agents::AgentBuildersVariants::*;
use greyjack::agents::termination_strategies::*;
use greyjack::agents::termination_strategies::TerminationStrategiesVariants::*;
use greyjack::utils::math_utils::*;
use persistence::{CotwinBuilder, DomainBuilder};
use crate::observers_examples::NQueensObserver;

fn main() {

    let domain_builder = DomainBuilder::new(16, 45);
    let cotwin_builder = CotwinBuilder::new(true);
    //println!("{}", domain_builder);

    let termination_strategy = ScL(ScoreLimit::new(SimpleScore::new(0.0)));
    //let termination_strategy = TSL(TimeSpentLimit::new(60*1000));
    //let termination_strategy = StL(StepsLimit::new(20));
    //let termination_strategy = SNI(ScoreNoImprovement::new(5*1000));
    
    let agent_builder = TS(TabuSearch::new(128, 0.0, Some(0.0), 10, termination_strategy));
    //let agent_builder = GA(GeneticAlgorithm::new(128, 0.5, 0.05, 0.2, Some(0.0), 0.00001, 1, termination_strategy));
    //let agent_builder = LA(LateAcceptance::new(5, 0.2, Some(0.0), 10000, termination_strategy));

    // example of optional observers
    //let mut observers: Vec<Box<dyn ObserverTrait + Send>> = Vec::new();
    //observers.push(Box::new(NQueensObserver::new(0)));

    // change logging_level to SolverLoggingLevels::Silent for max performance
    let solution = Solver::solve(
        domain_builder.clone(), cotwin_builder, agent_builder, 
        10, None, SolverLoggingLevels::FreshOnly, None, None,
    );

    let domain = domain_builder.build_from_solution(&solution);
    println!("{}", domain);

    println!("done");
}