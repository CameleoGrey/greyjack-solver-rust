

mod domain;
mod cotwin;
mod score;
mod persistence;

use std::fs;
use std::path::{PathBuf, Path};
use greyjack::domain::DomainBuilderTrait;
use greyjack::cotwin::CotwinBuilderTrait;
use greyjack::score_calculation::scores::HardSoftScore;
use persistence::{CotwinBuilder, DomainBuilder};
use greyjack::solver::{ObserverTrait, Solver, SolverLoggingLevels};
use greyjack::agents::{GeneticAlgorithm, LateAcceptance, TabuSearch};
use greyjack::agents::AgentBuildersVariants::*;
use greyjack::agents::termination_strategies::*;
use greyjack::agents::termination_strategies::TerminationStrategiesVariants::*;


fn main() {

    let mut file_path = vec!["data", "tsp", "data", "import"];
    file_path.append(&mut vec!["belgium", "air", "belgium-n50.tsp"]); //optimum: ~12.2; first_fit: ~
    //file_path.append(&mut vec!["cook", "air", "st70.tsp"]);
    //file_path.append(&mut vec!["tsplib", "a280.tsp"]);
    //file_path.append(&mut vec!["cook", "air", "pcb442.tsp"]); //optimum: 50778; first_fit: ~63k
    //file_path.append(&mut vec!["cook", "air", "lu980.tsp"]);
    //file_path.append(&mut vec!["belgium", "air", "belgium-n1000.tsp"]);
    //file_path.append(&mut vec!["other", "air", "usa_tx_2743.tsp"]); //optimum: ~282; first_fit: ~338
    //file_path.append(&mut vec!["belgium", "air", "belgium-n2750.tsp"]);
    //file_path.append(&mut vec!["tsplib", "fnl4461.tsp"]); //optimum: 182566; frist_fit: 
    //file_path.append(&mut vec!["cook", "air", "gr9882.tsp"]); //optimum: 300899; first_fit: ~400k
    //file_path.append(&mut vec!["other", "air", "usa115475.tsp"]);

    let file_path: PathBuf = file_path.iter().collect();
    let file_path = file_path.as_os_str().to_str().unwrap();

    let domain_builder = DomainBuilder::new(file_path);
    let cotwin_builder = CotwinBuilder::new(true);

    //let termination_strategy = ScL(ScoreLimit::new(HardSoftScore::new(0.0, 0.0)));
    //let termination_strategy = TSL(TimeSpentLimit::new(60*60*1000));
    //let termination_strategy = StL(StepsLimit::new(100));
    let termination_strategy = SNI(ScoreNoImprovement::new(5*1000));
    
    let agent_builder = TS(TabuSearch::new(128, 0.2, Some(0.0), 10, termination_strategy));
    //let agent_builder = GA(GeneticAlgorithm::new(128, 0.5, 0.05, 0.2, Some(0.0), 0.00001, 10, termination_strategy));
    //let agent_builder = LA(LateAcceptance::new(64, 0.2, Some(0.0), 10000, termination_strategy));

    // to make possible to build huge round-robin (use n_jobs >= cpus count) of communicating agents
    //rayon::ThreadPoolBuilder::new().num_threads(100).build_global().unwrap();

    let solution = Solver::solve(
        domain_builder.clone(), cotwin_builder, agent_builder, 
        10, Some(vec![3, 3]), SolverLoggingLevels::FreshOnly, None, None,
    );

    let domain = domain_builder.build_from_solution(&solution);
    domain.print_metrics();
    domain.print_path();

    println!("done");
}
