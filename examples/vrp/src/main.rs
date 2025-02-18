

mod domain;
mod cotwin;
mod score;
mod persistence;

use std::path::{PathBuf, Path};
use greyjack::domain::DomainBuilderTrait;
use greyjack::cotwin::CotwinBuilderTrait;
use persistence::{CotwinBuilder, DomainBuilder};
use greyjack::solver::{InitialSolutionVariants, ObserverTrait, Solver, SolverLoggingLevels};
use greyjack::agents::{GeneticAlgorithm, LateAcceptance, TabuSearch};
use greyjack::agents::AgentBuildersVariants::*;
use greyjack::agents::termination_strategies::*;
use greyjack::agents::termination_strategies::TerminationStrategiesVariants::*;
use rayon;

// one-stage solving
/*fn main() {

    let mut file_path = vec!["data", "vrp", "data", "import"];
    //file_path.append(&mut vec!["belgium", "multidepot-timewindowed", "air", "belgium-tw-d2-n50-k10.vrp"]); //optimum: ~16.3; first_fit: ~27.89
    file_path.append(&mut vec!["belgium", "multidepot-timewindowed", "air", "belgium-tw-d5-n500-k20.vrp"]); //optimum: ~45.2; first_fit: ~124.884
    //file_path.append(&mut vec!["belgium", "multidepot-timewindowed", "air", "belgium-tw-d8-n1000-k40.vrp"]); //optimum: ~64.7; first_fit: ~154.565
    //file_path.append(&mut vec!["belgium", "multidepot-timewindowed", "air", "belgium-tw-d10-n2750-k55.vrp"]); //optimum: ~; first_fit: ~380.9

    let file_path: PathBuf = file_path.iter().collect();
    let file_path = file_path.as_os_str().to_str().unwrap();

    let mut domain_builder = DomainBuilder::new(file_path);
    let cotwin_builder = CotwinBuilder::new(true);

    //let termination_strategy = ScL(ScoreLimit::new(HardMediumSoftScore::new(0.0, 0.0)));
    //let termination_strategy = TSL(TimeSpentLimit::new(60*60*1000));
    //let termination_strategy = StL(StepsLimit::new(1));
    let termination_strategy = SNI(ScoreNoImprovement::new(50*1000));
    
    //after bug-fixes very works very cool
    //let agent_builder = TS(TabuSearch::new(64, 0.2, None, 100, termination_strategy));
    //slower convergence, but better chances to find global optimum
    let agent_builder = LA(LateAcceptance::new(64, 0.2, None, 10000, termination_strategy));
    //good finds global optimum on small datasets, but slow due to need of using plain score calculator
    //let agent_builder = GA(GeneticAlgorithm::new(128, 0.5, 0.05, 0.2, Some(1.0), 0.00001, 10, termination_strategy)); 

    // to make possible to build huge round-robin (use n_jobs >= cpus count) of communicating agents
    //rayon::ThreadPoolBuilder::new().num_threads(128).build_global().unwrap();

    let solution = Solver::solve(
        domain_builder.clone(), cotwin_builder, agent_builder, 
        10, Some(vec![0, 0, 3]), SolverLoggingLevels::FreshOnly, None, None,
    );

    let domain = domain_builder.build_from_solution(&solution);
    domain.print_metrics();
    domain.print_trip_paths();

    println!("done");
}*/


// multi-stage solving
fn main() {

    let mut file_path = vec!["data", "vrp", "data", "import"];
    //file_path.append(&mut vec!["belgium", "multidepot-timewindowed", "air", "belgium-tw-d2-n50-k10.vrp"]); //optimum: ~16.3; first_fit: ~27.89
    file_path.append(&mut vec!["belgium", "multidepot-timewindowed", "air", "belgium-tw-d5-n500-k20.vrp"]); //optimum: ~45.2; first_fit: ~124.884
    //file_path.append(&mut vec!["belgium", "multidepot-timewindowed", "air", "belgium-tw-d8-n1000-k40.vrp"]); //optimum: ~64.7; first_fit: ~154.565
    //file_path.append(&mut vec!["belgium", "multidepot-timewindowed", "air", "belgium-tw-d10-n2750-k55.vrp"]); //optimum: ~; first_fit: ~380.9

    let file_path: PathBuf = file_path.iter().collect();
    let file_path = file_path.as_os_str().to_str().unwrap();

    let mut domain_builder = DomainBuilder::new(file_path);
    let cotwin_builder = CotwinBuilder::new(true);

    // to make possible to build huge round-robin (use n_jobs >= cpus count) of communicating agents
    //rayon::ThreadPoolBuilder::new().num_threads(100).build_global().unwrap();

    // 1-st stage (relatively fast and good for initial solution)
    //let termination_strategy = StL(StepsLimit::new(100));
    let termination_strategy = SNI(ScoreNoImprovement::new(5*1000)); 
    let agent_builder = TS(TabuSearch::new(32, 0.2, None, 1, termination_strategy));
    //let agent_builder = LA(LateAcceptance::new(2, 0.2, None, 1000, termination_strategy));
    let solution = Solver::solve(domain_builder.clone(), cotwin_builder.clone(), agent_builder, 
    10, Some(vec![0, 0, 3]), SolverLoggingLevels::FreshOnly, 
    None, None);

    //////////////////////////////////////////////////////////////////////////////////////////////////////////////
    // to try replanning and frozening (pinning entities) scenarios and using existing domain as initial solution
    /*let mut interim_domain = domain_builder.build_from_solution(&solution);
    println!();
    interim_domain.print_metrics();
    interim_domain.print_trip_paths();
    interim_domain.vehicles.remove(interim_domain.vehicles.len() - 1);
    println!();
    interim_domain.print_metrics();
    interim_domain.print_trip_paths();
    println!();
    (0..2).into_iter().for_each(|k| interim_domain.vehicles[k].customers.iter_mut().for_each(|customer| customer.frozen = true));

    let termination_strategy = SNI(ScoreNoImprovement::new(5*1000)); 
    let agent_builder = TS(TabuSearch::new(32, 0.2, None, 1, termination_strategy));
    //let agent_builder = LA(LateAcceptance::new(2, 0.2, None, 1000, termination_strategy));
    let solution = Solver::solve(domain_builder.clone(), cotwin_builder.clone(), agent_builder, 
    10, Some(vec![0, 0, 3]), SolverLoggingLevels::FreshOnly, 
    None, Some(InitialSolutionVariants::DomainObject(interim_domain)));*/
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////

    // 2-nd stage (relatively slow and more quality)
    let termination_strategy = SNI(ScoreNoImprovement::new(15*1000)); 
    //let agent_builder = TS(TabuSearch::new(128, 0.2, None, 10, termination_strategy));
    let agent_builder = LA(LateAcceptance::new(64, 0.2, None, 10000, termination_strategy));
    let solution = Solver::solve(domain_builder.clone(), cotwin_builder.clone(), agent_builder, 
    10, Some(vec![0, 0, 3]), SolverLoggingLevels::FreshOnly, 
    None, Some(InitialSolutionVariants::CotwinValuesVector(solution)));

    // 3-rd stage (just to try increase quality and reuse solution from 2-nd stage)
    /*let termination_strategy = SNI(ScoreNoImprovement::new(30*1000));
    let agent_builder = TS(TabuSearch::new(512, 0.2, None, 10, termination_strategy));    
    //let agent_builder = LA(LateAcceptance::new(200, 0.2, None, 10000, termination_strategy));
    let solution = Solver::solve(domain_builder.clone(), cotwin_builder.clone(), agent_builder, 
    10, Some(vec![0, 0, 3]), SolverLoggingLevels::FreshOnly, 
    None, Some(InitialSolutionVariants::CotwinValuesVector(solution)));*/

    let domain = domain_builder.build_from_solution(&solution);
    domain.print_metrics();
    domain.print_trip_paths();

    println!("done");
}