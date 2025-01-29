
mod domain;
mod cotwin;
mod score;
mod persistence;
mod observers_examples;

use greysplanner::score_calculation::scores::SimpleScore;
use greysplanner::cotwin::CotwinBuilderTrait;
use greysplanner::solver::{ObserverTrait, Solver, SolverLoggingLevels};
use greysplanner::agents::{GeneticAlgorithm, LateAcceptance, TabuSearch};
use greysplanner::agents::AgentBuildersVariants::*;
use greysplanner::agents::termination_strategies::*;
use greysplanner::agents::termination_strategies::TerminationStrategiesVariants::*;
use greysplanner::utils::math_utils::*;
use persistence::NQueensDomainUpdater;
use persistence::{NQueensCotwinBuilder, NQueensDomainBuilder};
use crate::observers_examples::NQueensObserver;

fn main() {

    let mut nqueens_domain = NQueensDomainBuilder::build_domain(256, 45);
    let nqueens_cotwin_builder = NQueensCotwinBuilder::new();
    //println!("{}", nqueens_domain);

    let termination_strategy = ScL(ScoreLimit::new(SimpleScore::new(0.0)));
    //let termination_strategy = TSL(TimeSpentLimit::new(60*1000));
    //let termination_strategy = StL(StepsLimit::new(20));
    //let termination_strategy = SNI(ScoreNoImprovement::new(5*1000));
    
    //let agent_builder = TS(TabuSearch::new(1, 128, 0, 0.2, Some(1.0), 0.00001, 1, termination_strategy));
    let agent_builder = GA(GeneticAlgorithm::new(128, 0.5, Some(1.0), 0.05, 0.00001, 1, termination_strategy));
    //let agent_builder = LA(LateAcceptance::new(1, 10, Some(1.0), 0.00001, 1, termination_strategy));

    // example of optional observers
    //let mut observers: Vec<Box<dyn ObserverTrait + Send>> = Vec::new();
    //observers.push(Box::new(NQueensObserver::new(0)));

    // change logging_level to SolverLoggingLevels::Silent for max performance
    let solution = Solver::solve(
        &nqueens_domain, nqueens_cotwin_builder, agent_builder, 
        10, None, SolverLoggingLevels::Info, None,
    );

    NQueensDomainUpdater::update_domain(&mut nqueens_domain, solution);
    //println!("{}", nqueens_domain);

    println!("done");
}