

use crate::domain::TravelSchedule;
use crate::cotwin::CotStop;
use crate::score::{TSPPlainScoreCalculator, TSPIncrementalScoreCalculator};
use greyjack::cotwin::{Cotwin, CotwinEntityTrait, CotwinValueTypes, CotwinBuilderTrait};
use greyjack::score_calculation::score_calculators::score_calculator_variants::ScoreCalculatorVariants;
use greyjack::score_calculation::scores::HardSoftScore;
use greyjack::variables::GJInteger;
use std::collections::{HashMap, HashSet};
use polars::datatypes::AnyValue;


pub enum EntityVariants<'a> {
    CotStop(CotStop<'a>)
}

impl<'a> CotwinEntityTrait for EntityVariants<'a> {
    fn to_vec(&self) -> Vec<(String, CotwinValueTypes)> {
        match self {
            EntityVariants::CotStop(x) => return x.to_vec()
        }
    }
}

pub enum UtilityObjectVariants {
    DistanceMatrix(Vec<Vec<f64>>),
    DeltasMap(Vec<Vec<(usize, usize)>>),
}


#[derive(Clone)]
pub struct CotwinBuilder {

    use_incremental_score_calculation: bool,
    use_greed_init: bool,
    
}

impl CotwinBuilder {

    pub fn new(use_incremental_score_calculation: bool, use_greed_init: bool) -> Self {
        Self {
            use_incremental_score_calculation: use_incremental_score_calculation,
            use_greed_init: use_greed_init,
        }
    }
    
    fn build_planning_stops<'a>(&self, domain: &TravelSchedule, is_already_initialized: bool) -> Vec<EntityVariants<'a>> {

        let n_locations = domain.locations_vec.len();
        let n_stops = n_locations - 1;
        let mut planning_stops_vec: Vec<EntityVariants<'a>> = Vec::new();

        if is_already_initialized {
            panic!("Building cotwin for existing domain isn't already implemented for TSP problem. Look VRP example to understand how to inplement it yourself")
        }

        let mut initial_stop_ids: Vec<Option<i64>>;
        if self.use_greed_init {
            initial_stop_ids = Self::build_greed_initialized_stops(domain);
        } else {
            initial_stop_ids = Self::build_default_initialized_stops(n_stops);
        }

        for i in 0..n_stops {
            //let current_location = &domain.locations_vec[i];
            let current_stop = EntityVariants::CotStop(
                CotStop {
                stop_id: CotwinValueTypes::PAV(AnyValue::UInt64(i as u64)),
                //init: Some((i+1) as i64)
                locations_vec_id: CotwinValueTypes::GJI(GJInteger::new(&format!("stop_{}_id", i), initial_stop_ids[i], 1, (n_locations-1) as i64, false, None))
            });

            planning_stops_vec.push(current_stop);
        }

        return planning_stops_vec;
    }

    fn build_default_initialized_stops(n_stops: usize) -> Vec<Option<i64>> {
        //let mut initial_stop_ids = vec![None; n_stops];
        let mut initial_stop_ids: Vec<Option<i64>> = (0..n_stops).into_iter().map(|i| Some((i+1) as i64)).collect();
        return initial_stop_ids;
    }

    fn build_greed_initialized_stops(domain: &TravelSchedule) -> Vec<Option<i64>> {

        let distance_matrix = &domain.distance_matrix;
        let n_stops = distance_matrix.len() - 1;
        let mut remaining_stops: HashSet<usize> = (0..n_stops).into_iter().map(|i| i+1).collect();
        let mut greed_stop_ids: Vec<usize> = Vec::new();

        for i in (0..n_stops) {
            let mut previous_stop_id;
            if greed_stop_ids.len() == 0 {
                previous_stop_id = 0; // depot
            } else {
                previous_stop_id = greed_stop_ids[i-1];
            }

            let mut best_distance = f64::MAX;
            let mut best_candidate: usize = 999999999;
            for candidate_stop_id in &remaining_stops {
                let current_distance = distance_matrix[previous_stop_id][*candidate_stop_id];
                if current_distance < best_distance {
                    best_distance = current_distance;
                    best_candidate = *candidate_stop_id;
                }
            }
            greed_stop_ids.push(best_candidate);
            remaining_stops.remove(&best_candidate);
        }

        let greed_stop_ids: Vec<Option<i64>> = greed_stop_ids.iter().map(|id| Some(*id as i64)).collect();
        return greed_stop_ids;
    }
}

impl<'a> CotwinBuilderTrait<TravelSchedule, EntityVariants<'a>, UtilityObjectVariants, HardSoftScore> for CotwinBuilder {

    fn build_cotwin(&self, domain: TravelSchedule, is_already_initialized: bool) -> Cotwin<EntityVariants<'a>, UtilityObjectVariants, HardSoftScore> {
        
        let mut cotwin = Cotwin::new();
        cotwin.add_planning_entities("path_stops".to_string(), self.build_planning_stops(&domain, is_already_initialized));

        if self.use_incremental_score_calculation {
            let mut score_calculator = TSPIncrementalScoreCalculator::new();
            score_calculator.add_utility_object("distance_matrix".to_string(), UtilityObjectVariants::DistanceMatrix((domain.distance_matrix)));
            cotwin.add_score_calculator(ScoreCalculatorVariants::ISC(score_calculator));
        } else {
            let mut score_calculator = TSPPlainScoreCalculator::new();
            score_calculator.add_utility_object("distance_matrix".to_string(), UtilityObjectVariants::DistanceMatrix((domain.distance_matrix)));
            cotwin.add_score_calculator(ScoreCalculatorVariants::PSC(score_calculator));
        }

        return cotwin;
    }

}