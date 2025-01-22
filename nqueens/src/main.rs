
mod domain;
mod cotwin;
mod score;
mod persistence;

use greysplanner::score_calculation::scores::SimpleScore;
use greysplanner::cotwin::CotwinBuilderTrait;
use greysplanner::solver::{Solver, SolverLoggingLevels};
use greysplanner::agents::GeneticAlgorithm;
use greysplanner::agents::AgentBuildersVariants::*;
use greysplanner::agents::termination_strategies::*;
use greysplanner::agents::termination_strategies::TerminationStrategiesVariants::*;
use greysplanner::utils::math_utils::*;
use persistence::NQueensDomainUpdater;
use persistence::{NQueensCotwinBuilder, NQueensDomainBuilder};
use polars::datatypes::AnyValue;
use serde_json;

fn main() {

    let mut nqueens_domain = NQueensDomainBuilder::build_domain(256, 45);
    let nqueens_cotwin_builder = NQueensCotwinBuilder::new();
    //println!("{}", nqueens_domain);

    //let termination_strategy = ScL(ScoreLimit::new(SimpleScore::new(0.0)));
    let termination_strategy = TSL(TimeSpentLimit::new(60*1000));
    //let termination_strategy = StL(StepsLimit::new(20));
    //let termination_strategy = SNI(ScoreNoImprovement::new(5*1000));
    let agent_builder = GA(GeneticAlgorithm::new(128, 0.0, Some(1.0), 0.05, 0.00001, 10, termination_strategy));
    let solution = Solver::solve(
        &nqueens_domain, nqueens_cotwin_builder, agent_builder, 
        10, None, SolverLoggingLevels::Info
    );

    NQueensDomainUpdater::update_domain(&mut nqueens_domain, solution);
    //println!("{}", nqueens_domain);

    println!("done");
}