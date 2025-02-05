

mod domain;
mod cotwin;
mod score;
mod persistence;

use std::path::{PathBuf, Path};
use greyjack::cotwin::CotwinBuilderTrait;
use persistence::{CotwinBuilder, DomainBuilder, DomainUpdater};
use greyjack::solver::{ObserverTrait, Solver, SolverLoggingLevels};
use greyjack::agents::{GeneticAlgorithm, LateAcceptance, TabuSearch};
use greyjack::agents::AgentBuildersVariants::*;
use greyjack::agents::termination_strategies::*;
use greyjack::agents::termination_strategies::TerminationStrategiesVariants::*;


fn main() {

    let mut file_path = vec!["data", "vrp", "data", "import"];
    //file_path.append(&mut vec!["belgium", "multidepot-timewindowed", "air", "belgium-tw-d2-n50-k10.vrp"]);
    //file_path.append(&mut vec!["belgium", "multidepot-timewindowed", "air", "belgium-tw-d5-n500-k20.vrp"]);
    file_path.append(&mut vec!["belgium", "multidepot-timewindowed", "air", "belgium-tw-d8-n1000-k40.vrp"]);

    let file_path: PathBuf = file_path.iter().collect();
    let file_path = file_path.as_os_str().to_str().unwrap();

    let mut domain = DomainBuilder::build_domain(file_path);
    let cotwin_builder = CotwinBuilder::new(false);

    //let termination_strategy = ScL(ScoreLimit::new(HardSoftScore::new(0.0, 0.0)));
    let termination_strategy = TSL(TimeSpentLimit::new(60*60*1000));
    //let termination_strategy = StL(StepsLimit::new(10));
    //let termination_strategy = SNI(ScoreNoImprovement::new(30*1000));
    
    //let agent_builder = GA(GeneticAlgorithm::new(128, 0.5, Some(1.0), 0.05, 0.00001, 10, termination_strategy));
    let agent_builder = TS(TabuSearch::new(1, 128, 0, 0.2, Some(0.0), 0.00001, 10, termination_strategy));
    //let agent_builder = LA(LateAcceptance::new(1, 20, Some(1.0), 0.00001, 1000, termination_strategy));

    // change logging_level to SolverLoggingLevels::Silent for max performance
    let solution = Solver::solve(
        &domain, cotwin_builder, agent_builder, 
        10, None, SolverLoggingLevels::Info, None,
    );

    DomainUpdater::update_domain(&mut domain, solution);
    domain.print_metrics();
    domain.print_trip_paths();

    println!("done");
}
