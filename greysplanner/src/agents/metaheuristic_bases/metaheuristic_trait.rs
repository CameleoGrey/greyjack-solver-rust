
use crate::score_calculation::scores::ScoreTrait;
use crate::agents::base::Individual;
use crate::score_calculation::score_requesters::VariablesManager;
use ndarray::Array1;
use std::ops::AddAssign;
use std::fmt::Debug;

pub trait MetaheuristicBaseTrait<ScoreType>
where ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send {

    fn sample_candidates(
        &mut self,
        population: &mut Vec<Individual<ScoreType>>, 
        current_top_individual: &Individual<ScoreType>,
        variables_manager: &VariablesManager
    ) -> Vec<Array1<f64>>;

    fn build_updated_population(
        &mut self, 
        current_population: &Vec<Individual<ScoreType>>, 
        candidates: &Vec<Individual<ScoreType>>
    ) -> Vec<Individual<ScoreType>>;
}