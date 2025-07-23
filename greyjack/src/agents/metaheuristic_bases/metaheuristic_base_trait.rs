
use crate::score_calculation::scores::ScoreTrait;
use crate::agents::base::Individual;
use crate::score_calculation::score_requesters::VariablesManager;
use std::ops::{AddAssign, Sub};
use std::fmt::Debug;
use super::metaheuristic_kinds_and_names::{MetaheuristicKind, MetaheuristicNames};

pub trait MetaheuristicBaseTrait<ScoreType>
where ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send {

    fn sample_candidates_plain(
        &mut self,
        population: &mut Vec<Individual<ScoreType>>, 
        current_top_individual: &Individual<ScoreType>,
        variables_manager: &VariablesManager
    ) -> Vec<Vec<f64>>;

    fn sample_candidates_incremental(
        &mut self,
        population: &mut Vec<Individual<ScoreType>>, 
        current_top_individual: &Individual<ScoreType>,
        variables_manager: &VariablesManager
    ) -> (Vec<f64>, Vec<Vec<(usize, f64)>>);

    fn build_updated_population(
        &mut self, 
        current_population: &Vec<Individual<ScoreType>>, 
        candidates: &mut Vec<Individual<ScoreType>>
    ) -> Vec<Individual<ScoreType>>;

    fn build_updated_population_incremental(
        &mut self, 
        current_population: &Vec<Individual<ScoreType>>, 
        sample: &mut Vec<f64>,
        deltas: Vec<Vec<(usize, f64)>>,
        scores: Vec<ScoreType>,
    ) -> Vec<Individual<ScoreType>>;

    fn get_metaheuristic_kind(&self) -> MetaheuristicKind;

    fn get_metaheuristic_name(&self) -> MetaheuristicNames;
}