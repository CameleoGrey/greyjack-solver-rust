
use crate::core::score_calculation::scores::score_trait::ScoreTrait;
use crate::core::agents::base::individual::Individual;
use ndarray::Array1;
use std::ops::AddAssign;

pub trait MetaheuristicBaseTrait<ScoreType>
where ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord {

    fn sample_candidates(
        &mut self, 
        population_size: usize, 
        population: &mut Vec<Individual<ScoreType>>, 
        current_top_individual: &Individual<ScoreType>
    ) -> Vec<Array1<f64>>;

    fn update_population(
        &mut self, 
        current_population: &mut Vec<Individual<ScoreType>>, 
        new_population: &mut Vec<Individual<ScoreType>>
    );
}