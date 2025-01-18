
mod domain;
mod cotwin;
mod score;
mod persistence;

use std::collections::HashMap;
use greysplanner::score_calculation::scores::SimpleScore;
use persistence::{CotwinBuilder, DomainGenerator};

use greysplanner::solver::Solver;
use greysplanner::agents::GeneticAlgorithm;
use greysplanner::agents::termination_strategies::*;
use greysplanner::agents::termination_strategies::TerminationStrategiesVariants::*;
use ndarray::{Array1};
use polars::datatypes::AnyValue;
use chrono::{prelude::*, TimeDelta};

fn main() {

    let nqueens_domain = DomainGenerator::generate_domain(256, 45);
    println!("{}", nqueens_domain);


    let n_jobs = 12;
    let mut agents = Vec::new();
    
    for i in 0..n_jobs {
        //let termination_strategy = ScL(ScoreLimit::new(SimpleScore::new(0.0)));
        //let termination_strategy = TSL(TimeSpentLimit::new(60*1000));
        let termination_strategy = StL(StepsLimit::new(20000));
        //let termination_strategy = SNI(ScoreNoImprovement::new(15*1000));
        let agent = GeneticAlgorithm::new(
            CotwinBuilder::build_cotwin(&nqueens_domain), 128, 0.5, Some(1.0), 0.05, 
            0.000001, 10, termination_strategy
        );

        agents.push(agent);
    }

    //agents[0].solve();
    //agents[1].solve();
    let solver = Solver::solve(agents, vec![1]);

    println!("done");
}
