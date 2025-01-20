
mod domain;
mod cotwin;
mod score;
mod persistence;

use std::collections::HashMap;
use greysplanner::agents::AgentBuildersVariants;
use greysplanner::score_calculation::scores::SimpleScore;
use greysplanner::cotwin::CotwinBuilderTrait;
use greysplanner::solver::Solver;
use greysplanner::agents::GeneticAlgorithm;
use greysplanner::agents::AgentBuildersVariants::*;
use greysplanner::agents::termination_strategies::*;
use greysplanner::agents::termination_strategies::TerminationStrategiesVariants::*;
use persistence::{CotwinBuilder, DomainGenerator};
use ndarray::{Array1};
use polars::datatypes::AnyValue;
use chrono::{prelude::*, TimeDelta};

fn main() {

    let nqueens_domain = DomainGenerator::generate_domain(256, 45);
    let nqueens_cotwin_builder = CotwinBuilder::new();
    println!("{}", nqueens_domain);


    let n_jobs = 10;
    let mut agent_builders = Vec::new();
    for i in 0..n_jobs {
        //let termination_strategy = ScL(ScoreLimit::new(SimpleScore::new(0.0)));
        let termination_strategy = TSL(TimeSpentLimit::new(60*1000));
        //let termination_strategy = StL(StepsLimit::new(100));
        //let termination_strategy = SNI(ScoreNoImprovement::new(5*1000));
        let agent_builder = GA(GeneticAlgorithm::new(128, 0.5, Some(1.0), 0.05, 0.000001, 10, termination_strategy));
        agent_builders.push(agent_builder);
    }

    /*let cotwin = nqueens_cotwin_builder.build_cotwin(nqueens_domain);
    let mut agent;
    match &agent_builders[0] {
        AgentBuildersVariants::GA(ga) => agent = ga.build_agent(cotwin),
    }
    agent.solve();*/
    //agent_builders[0].solve();
    //agent_builders[1].solve();
    let solver = Solver::solve(nqueens_domain, nqueens_cotwin_builder, agent_builders, vec![0]);

    println!("done");
}
