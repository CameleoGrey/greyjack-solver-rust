
mod domain;
mod cotwin;
mod persistence;
mod score;

use std::path::{PathBuf, Path};
use greyjack::domain::DomainBuilderTrait;
use greyjack::cotwin::CotwinBuilderTrait;
use persistence::{CotwinBuilder, DomainBuilder, CalculatorType};
use greyjack::solver::{ObserverTrait, Solver, SolverLoggingLevels};
use greyjack::agents::*;
use greyjack::agents::AgentBuildersVariants::*;
use greyjack::agents::termination_strategies::*;
use greyjack::agents::termination_strategies::TerminationStrategiesVariants::*;
use greyjack::score_calculation::scores::HardSoftScore;

fn main() {
    let mut file_path = vec!["data", "cloudbalancing"];
    //file_path.append(&mut vec!["4computers-12processes.json"]);
    //file_path.append(&mut vec!["100computers-300processes.json"]);
    //file_path.append(&mut vec!["200computers-600processes.json"]);
    file_path.append(&mut vec!["400computers-1200processes.json"]);
    //file_path.append(&mut vec!["800computers-2400processes.json"]);
    //file_path.append(&mut vec!["1600computers-4800processes.json"]);

    let file_path: PathBuf = file_path.iter().collect();
    let file_path = file_path.as_os_str().to_str().unwrap();

    let domain_builder = DomainBuilder::new(file_path);
    let cotwin_builder = CotwinBuilder::new(CalculatorType::Greynet, true);

    //let termination_strategy = StL(StepsLimit::new(1000));
    let termination_strategy = TSL(TimeSpentLimit::new(120*1000));
    //let termination_strategy = SNI(ScoreNoImprovement::new(15*1000));
    //let termination_strategy = ScL(ScoreLimit::new(HardSoftScore::new(0.0, 0.0)));

    let agent_builder = TS(TabuSearch::new(20, 0.2, None, Some(vec![0.5, 0.5, 0.0, 0.0, 0.0, 0.0]), 10, 99999, termination_strategy));
    //let agent_builder = LA(LateAcceptance::new(64, 0.0, None, Some(vec![0.5, 0.5, 0.0, 0.0, 0.0, 0.0]), 10000, 10, termination_strategy));
    //let agent_builder = SA(SimulatedAnnealing::new(vec![1.0, 1.0], Some(0.9999), 0.0, None, Some(vec![0.5, 0.5, 0.0, 0.0, 0.0, 0.0]), 9999999, 1, termination_strategy));

    let solution = Solver::solve(
        domain_builder.clone(), cotwin_builder, agent_builder,
        10, Some(vec![0, 0]), SolverLoggingLevels::FreshOnly,
        None, None,
    );

    let domain = domain_builder.build_from_solution(&solution, None);
    domain.print_metrics();

    println!("done");
}