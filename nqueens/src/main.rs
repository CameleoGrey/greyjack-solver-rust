
mod domain;
mod cotwin;
mod score;
mod persistence;

use std::collections::HashMap;

use greysplanner::core::agents::termination_strategies::score_limit::ScoreLimit;
use greysplanner::core::score_calculation::scores::simple_score::SimpleScore;
use persistence::{
    cotwin_builder::CotwinBuilder, 
    domain_generator::DomainGenerator
};

use greysplanner::core::agents::{genetic_algorithm::GeneticAlgorithm, termination_strategies::score_no_improvement::ScoreNoImprovement};
use greysplanner::core::agents::termination_strategies::steps_limit::StepsLimit;
use greysplanner::core::agents::termination_strategies::time_spent_limit::TimeSpentLimit;
use greysplanner::core::agents::termination_strategies::termination_strategies_variants::TerminationStrategiesVariants::*;
use ndarray::{Array1};
use polars::datatypes::AnyValue;
use chrono::{prelude::*, TimeDelta};

fn main() {

    let nqueens_domain = DomainGenerator::generate_domain(256, 45);
    println!("{}", nqueens_domain);

    let mut nqueens_cotwin = CotwinBuilder::build_cotwin(&nqueens_domain);

    let termination_strategy = ScL(ScoreLimit::new(SimpleScore::new(0.0)));
    //let termination_strategy = TSL(TimeSpentLimit::new(60*1000));
    //let termination_strategy = StL(StepsLimit::new(300));
    //let termination_strategy = SNI(ScoreNoImprovement::new(15*1000));
    let mut agent = GeneticAlgorithm::new(
        &mut nqueens_cotwin, 128, 0.5, Some(1.0), 0.05, 
        0.000001, 10, termination_strategy
    );

    agent.solve();

    println!("done");
}
