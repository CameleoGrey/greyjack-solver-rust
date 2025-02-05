

mod domain;
mod cotwin;
mod score;
mod persistence;

use std::fs;
use std::path::{PathBuf, Path};
use greyjack::cotwin::CotwinBuilderTrait;
use greyjack::score_calculation::scores::HardSoftScore;
use persistence::{CotwinBuilder, DomainBuilder, DomainUpdater};
use greyjack::solver::{ObserverTrait, Solver, SolverLoggingLevels};
use greyjack::agents::{GeneticAlgorithm, LateAcceptance, TabuSearch};
use greyjack::agents::AgentBuildersVariants::*;
use greyjack::agents::termination_strategies::*;
use greyjack::agents::termination_strategies::TerminationStrategiesVariants::*;


fn main() {

    let mut file_path = vec!["data", "tsp", "data", "import"];
    file_path.append(&mut vec!["belgium", "air", "belgium-n50.tsp"]);
    //file_path.append(&mut vec!["cook", "air", "st70.tsp"]);
    //file_path.append(&mut vec!["tsplib", "a280.tsp"]);
    //file_path.append(&mut vec!["cook", "air", "pcb442.tsp"]); 
    //file_path.append(&mut vec!["cook", "air", "lu980.tsp"]);
    //file_path.append(&mut vec!["belgium", "air", "belgium-n1000.tsp"]);
    //file_path.append(&mut vec!["other", "air", "usa_tx_2743.tsp"]); 
    //file_path.append(&mut vec!["belgium", "air", "belgium-n2750.tsp"]);
    //file_path.append(&mut vec!["cook", "air", "gr9882.tsp"]);
    //file_path.append(&mut vec!["other", "air", "usa115475.tsp"]);

    let file_path: PathBuf = file_path.iter().collect();
    let file_path = file_path.as_os_str().to_str().unwrap();

    let mut domain = DomainBuilder::build_domain(file_path);
    let cotwin_builder = CotwinBuilder::new(true);

    //let termination_strategy = ScL(ScoreLimit::new(HardSoftScore::new(0.0, 0.0)));
    //let termination_strategy = TSL(TimeSpentLimit::new(20*1000));
    //let termination_strategy = StL(StepsLimit::new(5000));
    let termination_strategy = SNI(ScoreNoImprovement::new(30*1000));
    
    let agent_builder = TS(TabuSearch::new(1, 128, 0, 0.2, Some(1.0), 0.00001, 100, termination_strategy));
    //let agent_builder = GA(GeneticAlgorithm::new(128, 0.5, Some(1.0), 0.05, 0.00001, 100, termination_strategy));
    //let agent_builder = LA(LateAcceptance::new(1, 20, Some(1.0), 0.00001, 1000, termination_strategy));

    // change logging_level to SolverLoggingLevels::Silent for max performance
    // use n_jobs near your CPU count for increase probability to find global optimum. 
    // if n_jobs > available CPU, solver dies.
    let solution = Solver::solve(
        &domain, cotwin_builder, agent_builder, 
        20, None, SolverLoggingLevels::Info, None,
    );

    DomainUpdater::update_domain(&mut domain, solution);

    domain.print_metrics();
    domain.print_path();

    println!("done");
}
