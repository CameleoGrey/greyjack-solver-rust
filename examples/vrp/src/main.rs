

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
fn main() {

    let mut file_path = vec!["data", "vrp", "data", "import"];
    // 1-depot datasets (plain CVRP)
    //file_path.append(&mut vec!["belgium", "basic", "air", "belgium-n50-k10.vrp"]);
    //file_path.append(&mut vec!["belgium", "basic", "air", "belgium-n500-k20.vrp"]);
    //file_path.append(&mut vec!["belgium", "basic", "air", "belgium-n1000-k40.vrp"]); //optimum: ~57.7; first_fit: ~195.3; RoutingModel: from 67.3 to 74 (depends on time) 
    //file_path.append(&mut vec!["belgium", "basic", "air", "belgium-n2750-k55.vrp"]);
    // multidepot datasets
    //file_path.append(&mut vec!["belgium", "multidepot", "air", "belgium-d2-n50-k10.vrp"]);
    //file_path.append(&mut vec!["belgium", "multidepot", "air", "belgium-d5-n500-k20.vrp"]);
    //file_path.append(&mut vec!["belgium", "multidepot", "air", "belgium-d8-n1000-k40.vrp"]);
    //file_path.append(&mut vec!["belgium", "multidepot", "air", "belgium-d10-n2750-k55.vrp"]);
    // multidepot datasets with timewindow constraint
    //file_path.append(&mut vec!["belgium", "multidepot-timewindowed", "air", "belgium-tw-d2-n50-k10.vrp"]); //optimum: ~15.98; first_fit: ~27.89
    file_path.append(&mut vec!["belgium", "multidepot-timewindowed", "air", "belgium-tw-d5-n500-k20.vrp"]); //optimum: ~43.3; first_fit: ~124.884
    //file_path.append(&mut vec!["belgium", "multidepot-timewindowed", "air", "belgium-tw-d8-n1000-k40.vrp"]); //optimum: ~58.1; first_fit: ~154.565
    //has some locations with different coordinates, but with the same name (for example: Antwerpen)
    //file_path.append(&mut vec!["belgium", "multidepot-timewindowed", "air", "belgium-tw-d10-n2750-k55.vrp"]); //optimum: ~111; first_fit: ~380.9

    let file_path: PathBuf = file_path.iter().collect();
    let file_path = file_path.as_os_str().to_str().unwrap();

    let domain_builder = DomainBuilder::new(file_path);
    let cotwin_builder = CotwinBuilder::new(true, true);

    // 1-st stage
    //let termination_strategy = StL(StepsLimit::new(100));
    //let termination_strategy = TSL(TimeSpentLimit::new(60*1000));
    let termination_strategy = SNI(ScoreNoImprovement::new(15*1000));
    let agent_builder = TS(TabuSearch::new(128, 0.0, true, None, Some(vec![0.5, 0.5, 0.0, 0.0, 0.0, 0.0]), 10, termination_strategy));
    //let agent_builder = LA(LateAcceptance::new(128, 0.2, None, None, 10000, termination_strategy));
    //let agent_builder = GA(GeneticAlgorithm::new(128, 0.5, 0.2, 0.05, Some(1.0), None, 0.00001, 10, termination_strategy)); 
    
    // to make possible to build huge round-robin (use n_jobs >= cpus count) of communicating agents
    //rayon::ThreadPoolBuilder::new().num_threads(100).build_global().unwrap();
    let solution = Solver::solve(domain_builder.clone(), cotwin_builder.clone(), agent_builder, 
    10, Some(vec![0, 0, 3]), SolverLoggingLevels::FreshOnly, 
    None, None);

    let domain = domain_builder.build_from_solution(&solution, None);
    domain.print_metrics();
    domain.print_trip_paths();

    println!("done");
}


// multi-stage solving
/*fn main() {

    let mut file_path = vec!["data", "vrp", "data", "import"];

    // 1-depot datasets (plain CVRP)
    //file_path.append(&mut vec!["belgium", "basic", "air", "belgium-n50-k10.vrp"]);
    //file_path.append(&mut vec!["belgium", "basic", "air", "belgium-n500-k20.vrp"]);
    //file_path.append(&mut vec!["belgium", "basic", "air", "belgium-n1000-k40.vrp"]); //optimum: ~57.7; first_fit: ~195.3
    //file_path.append(&mut vec!["belgium", "basic", "air", "belgium-n2750-k55.vrp"]);
    // multidepot datasets
    //file_path.append(&mut vec!["belgium", "multidepot", "air", "belgium-d2-n50-k10.vrp"]);
    //file_path.append(&mut vec!["belgium", "multidepot", "air", "belgium-d5-n500-k20.vrp"]);
    //file_path.append(&mut vec!["belgium", "multidepot", "air", "belgium-d8-n1000-k40.vrp"]);
    //file_path.append(&mut vec!["belgium", "multidepot", "air", "belgium-d10-n2750-k55.vrp"]);
    // multidepot datasets with timewindow constraint
    file_path.append(&mut vec!["belgium", "multidepot-timewindowed", "air", "belgium-tw-d2-n50-k10.vrp"]); //optimum: ~15.98; first_fit: ~27.89
    //file_path.append(&mut vec!["belgium", "multidepot-timewindowed", "air", "belgium-tw-d5-n500-k20.vrp"]); //optimum: ~43.3; first_fit: ~124.884
    //file_path.append(&mut vec!["belgium", "multidepot-timewindowed", "air", "belgium-tw-d8-n1000-k40.vrp"]); //optimum: ~58.1; first_fit: ~154.565
    //has some locations with different coordinates, but with the same name (for example: Antwerpen)
    //file_path.append(&mut vec!["belgium", "multidepot-timewindowed", "air", "belgium-tw-d10-n2750-k55.vrp"]); //optimum: ~111; first_fit: ~380.9

    let file_path: PathBuf = file_path.iter().collect();
    let file_path = file_path.as_os_str().to_str().unwrap();

    let domain_builder = DomainBuilder::new(file_path);
    let cotwin_builder = CotwinBuilder::new(true, true);

    // to make possible to build huge round-robin (use n_jobs >= cpus count) of communicating agents
    //rayon::ThreadPoolBuilder::new().num_threads(100).build_global().unwrap();

    // 1-st stage
    //let termination_strategy = StL(StepsLimit::new(100));
    //let termination_strategy = TSL(TimeSpentLimit::new(60*60*1000));
    let termination_strategy = SNI(ScoreNoImprovement::new(5*1000));
    //let agent_builder = TS(TabuSearch::new(32, 0.2, true, None, Some(vec![0.5, 0.5, 0.0, 0.0, 0.0, 0.0]), 10, termination_strategy));
    let agent_builder = TS(TabuSearch::new(128, 0.2, true, None, Some(vec![0.5, 0.5, 0.0, 0.0, 0.0, 0.0]), 10, termination_strategy));
    //let agent_builder = TS(TabuSearch::new(128, 0.2, true, None, None, 10, termination_strategy));
    //let agent_builder = LA(LateAcceptance::new(20, 0.2, None, Some(vec![0.5, 0.5, 0.0, 0.0, 0.0, 0.0]), 10000, termination_strategy));
    //let agent_builder = GA(GeneticAlgorithm::new(128, 0.5, 0.2, 0.05, Some(1.0), None, 0.00001, 10, termination_strategy)); 
    let solution = Solver::solve(domain_builder.clone(), cotwin_builder.clone(), agent_builder, 
    10, Some(vec![0, 0, 3]), SolverLoggingLevels::FreshOnly, 
    None, None);

    //////////////////////////////////////////////////////////////////////////////////////////////////////////////
    // to try replanning and frozening (pinning entities) scenarios and using existing domain as initial solution
    let mut interim_domain = domain_builder.build_from_solution(&solution, None);
    println!();
    interim_domain.print_metrics();
    interim_domain.print_trip_paths();
    interim_domain.vehicles.remove(0);
    println!();
    interim_domain.print_metrics();
    interim_domain.print_trip_paths();
    println!();
    (0..1).into_iter().for_each(|k| interim_domain.vehicles[k].customers.iter_mut().for_each(|customer| customer.frozen = true));

    let termination_strategy = SNI(ScoreNoImprovement::new(2*1000)); 
    let agent_builder = TS(TabuSearch::new(128, 0.2, true, None, Some(vec![0.5, 0.5, 0.0, 0.0, 0.0, 0.0]), 10, termination_strategy));
    //let agent_builder = LA(LateAcceptance::new(2, 0.2, true, None, None, 1000, termination_strategy));
    let solution = Solver::solve(domain_builder.clone(), cotwin_builder.clone(), agent_builder, 
                    10, Some(vec![0, 0, 3]), SolverLoggingLevels::FreshOnly, 
                    None, Some(InitialSolutionVariants::DomainObject(interim_domain.clone())));
    let domain = domain_builder.build_from_solution(&solution, Some(interim_domain));
    domain.print_metrics();
    domain.print_trip_paths();

    //////////////////////////////////////////////////////////////////////////////////////////////////////////////

    // 2-nd stage
    //let cotwin_builder = CotwinBuilder::new(false, true);
    /*let termination_strategy = SNI(ScoreNoImprovement::new(30*1000)); 
    let agent_builder = TS(TabuSearch::new(6000, 0.2, true, Some(1.0), None, 10, termination_strategy));
    //let agent_builder = LA(LateAcceptance::new(128, 0.2, None, Some(vec![0.5, 0.5, 0.0, 0.0, 0.0, 0.0]), 10000, termination_strategy));
    //let agent_builder = GA(GeneticAlgorithm::new(128, 0.5, 0.2, 0.05, Some(1.0), None, 0.00001, 10, termination_strategy)); 
    let solution = Solver::solve(domain_builder.clone(), cotwin_builder.clone(), agent_builder, 
        10, Some(vec![0, 0, 3]), SolverLoggingLevels::FreshOnly,
        None, Some(InitialSolutionVariants::CotwinValuesVector(solution)));

    // 3-rd stage (just to try increase quality and reuse solution from 2-nd stage)
    /*let termination_strategy = SNI(ScoreNoImprovement::new(30*1000));
    let agent_builder = TS(TabuSearch::new(512, 0.2, true, None, None, 10, termination_strategy));    
    //let agent_builder = LA(LateAcceptance::new(200, 0.2, None, None, 10000, termination_strategy));
    let solution = Solver::solve(domain_builder.clone(), cotwin_builder.clone(), agent_builder, 
    10, Some(vec![0, 0, 3]), SolverLoggingLevels::FreshOnly, 
    None, Some(InitialSolutionVariants::CotwinValuesVector(solution)));*/

    let domain = domain_builder.build_from_solution(&solution, None);
    domain.print_metrics();
    domain.print_trip_paths();*/

    println!("done");
}*/