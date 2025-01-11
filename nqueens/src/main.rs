
mod domain;
mod cotwin;
mod score;
mod persistence;

use std::collections::HashMap;

use persistence::{
    cotwin_builder::CotwinBuilder, 
    domain_generator::DomainGenerator
};

use greysplanner::core::score_calculation::score_requesters::oop_score_requester::OOPScoreRequester;
use ndarray::{Array1};
use polars::datatypes::AnyValue;

fn main() {

    let nqueens_domain = DomainGenerator::generate_domain(32, 45);
    println!("{}", nqueens_domain);

    let mut nqueens_cotwin = CotwinBuilder::build_cotwin(&nqueens_domain);

    let mut score_requester = OOPScoreRequester::new(&mut nqueens_cotwin);
    let mut samples: Vec<Array1<f64>> = Vec::new();
    for i in 0..10 {
        let generated_sample = score_requester.variables_manager.sample_variables();
        println!("{:?}", generated_sample);
        samples.push(generated_sample);
    }
    
    let scores = score_requester.request_score(&samples);

    for score in scores {
        println!("{:?}", score);
    }


    println!("done");
}
