

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
    //file_path.append(&mut vec!["belgium", "air", "belgium-n50.tsp"]);
    //file_path.append(&mut vec!["cook", "air", "st70.tsp"]); // 682.57 - best possible
    //file_path.append(&mut vec!["belgium", "road-km", "belgium-road-km-n100.tsp"]); // 1727.262 - best possible, 2089 - greed heuristic, 1909 - nice result without init
    //file_path.append(&mut vec!["tsplib", "a280.tsp"]); // 2579 - optimal
    //file_path.append(&mut vec!["cook", "air", "pcb442.tsp"]); //~54757 - best possible, 63881 - greed heuristic
    //file_path.append(&mut vec!["cook", "air", "lu980.tsp"]);
    //file_path.append(&mut vec!["belgium", "air", "belgium-n1000.tsp"]);
    //file_path.append(&mut vec!["other", "air", "usa_tx_2743.tsp"]); // ~282 - best possible, 319 - greed heuristic, 330 - nice result without init
    //file_path.append(&mut vec!["belgium", "air", "belgium-n2750.tsp"]);
    file_path.append(&mut vec!["tsplib", "rl11849.tsp"]);
    //file_path.append(&mut vec!["other", "air", "usa115475.tsp"]);

    let file_path: PathBuf = file_path.iter().collect();
    let file_path = file_path.as_os_str().to_str().unwrap();

    let mut domain = DomainBuilder::build_domain(file_path);
    let cotwin_builder = CotwinBuilder::new(true);

    //let termination_strategy = ScL(ScoreLimit::new(HardSoftScore::new(0.0, 0.0)));
    //let termination_strategy = TSL(TimeSpentLimit::new(60*1000));
    let termination_strategy = StL(StepsLimit::new(100));
    //let termination_strategy = SNI(ScoreNoImprovement::new(15*1000));
    
    let agent_builder = TS(TabuSearch::new(1, 128, 0, 0.2, Some(0.0), 0.00001, 10, termination_strategy));
    //let agent_builder = GA(GeneticAlgorithm::new(128, 0.5, Some(1.0), 0.05, 0.00001, 10, termination_strategy));
    //let agent_builder = LA(LateAcceptance::new(1, 20, Some(1.0), 0.00001, 100, termination_strategy));

    // change logging_level to SolverLoggingLevels::Silent for max performance
    let solution = Solver::solve(
        &domain, cotwin_builder, agent_builder, 
        10, None, SolverLoggingLevels::Info, None,
    );

    DomainUpdater::update_domain(&mut domain, solution);

    domain.print_metrics();
    domain.print_path();

    println!("done");
}
