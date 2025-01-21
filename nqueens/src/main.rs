
mod domain;
mod cotwin;
mod score;
mod persistence;

use greysplanner::score_calculation::scores::SimpleScore;
use greysplanner::cotwin::CotwinBuilderTrait;
use greysplanner::solver::Solver;
use greysplanner::agents::GeneticAlgorithm;
use greysplanner::agents::AgentBuildersVariants::*;
use greysplanner::agents::termination_strategies::*;
use greysplanner::agents::termination_strategies::TerminationStrategiesVariants::*;
use greysplanner::utils::math_utils::*;
use persistence::DomainUpdater;
use persistence::{CotwinBuilder, DomainGenerator};
use polars::datatypes::AnyValue;
use serde_json;

fn main() {

    let mut nqueens_domain = DomainGenerator::generate_domain(4096, 45);
    let nqueens_cotwin_builder = CotwinBuilder::new();
    //println!("{}", nqueens_domain);


    let n_jobs = 10;
    let mut agent_builders = Vec::new();
    for i in 0..n_jobs {
        let termination_strategy = ScL(ScoreLimit::new(SimpleScore::new(0.0)));
        //let termination_strategy = TSL(TimeSpentLimit::new(180*1000));
        //let termination_strategy = StL(StepsLimit::new(20));
        //let termination_strategy = SNI(ScoreNoImprovement::new(5*1000));
        let agent_builder = GA(GeneticAlgorithm::new(128, 0.5, Some(1.0), 0.05, 0.00001, 1, termination_strategy));
        agent_builders.push(agent_builder);
    }

    let solution = Solver::solve(&nqueens_domain, nqueens_cotwin_builder, agent_builders, None);

    DomainUpdater::update_domain(&mut nqueens_domain, solution);
    //println!("{}", nqueens_domain);

    println!("done");
}