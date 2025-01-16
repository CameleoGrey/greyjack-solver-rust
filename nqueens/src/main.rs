
mod domain;
mod cotwin;
mod score;
mod persistence;

use std::collections::HashMap;

use persistence::{
    cotwin_builder::CotwinBuilder, 
    domain_generator::DomainGenerator
};

use greysplanner::core::agents::genetic_algorithm::GeneticAlgorithm;
use greysplanner::core::agents::termination_strategies::steps_limit::StepsLimit;
use greysplanner::core::agents::termination_strategies::termination_strategies_variants::TerminationStrategiesVariants::SL;
use ndarray::{Array1};
use polars::datatypes::AnyValue;


fn main() {

    let nqueens_domain = DomainGenerator::generate_domain(4096, 45);
    //println!("{}", nqueens_domain);

    let mut nqueens_cotwin = CotwinBuilder::build_cotwin(&nqueens_domain);

    let mut agent = GeneticAlgorithm::new(&mut nqueens_cotwin, 128, 0.5, Some(1.0), 0.05, 0.000001, 10, SL(StepsLimit::new(10000)));

    agent.solve();

    println!("done");
}
