

use crate::domain::TravelSchedule;
use crate::cotwin::CotStop;
use crate::score::{TSPPlainScoreCalculator, TSPIncrementalScoreCalculator};
use greyjack::cotwin::{Cotwin, CotwinEntityTrait, CotwinValueTypes, CotwinBuilderTrait};
use greyjack::score_calculation::score_calculators::score_calculator_variants::ScoreCalculatorVariants;
use greyjack::score_calculation::scores::HardSoftScore;
use greyjack::variables::GJInteger;
use std::collections::HashMap;
use polars::datatypes::AnyValue;


pub enum EntityVariants<'a> {
    CotStop(CotStop<'a>)
}

impl<'a> CotwinEntityTrait for EntityVariants<'a> {
    fn to_hash_map(&self) -> HashMap<String, CotwinValueTypes> {
        match self {
            EntityVariants::CotStop(x) => return x.to_hash_map()
        }
    }
}

pub enum UtilityObjectVariants {
    DistanceMatrix(Vec<Vec<f64>>),
    DeltasMap(Vec<HashMap<usize, usize>>),
}


#[derive(Clone)]
pub struct CotwinBuilder {

    use_incremental_score_calculation: bool
    
}

impl CotwinBuilder {
    
    fn build_planning_stops<'a>(domain: &TravelSchedule) -> Vec<EntityVariants<'a>> {

        let n_locations = domain.locations_vec.len();
        let n_stops = n_locations - 1;
        let mut planning_stops_vec: Vec<EntityVariants<'a>> = Vec::new();
        for i in 0..n_stops {
            //let current_location = &domain.locations_vec[i];
            let current_stop = EntityVariants::CotStop(
                CotStop {
                stop_id: CotwinValueTypes::PAV(AnyValue::UInt64(i as u64)),
                locations_vec_id: CotwinValueTypes::GJI(GJInteger::new(&format!("stop_{}_id", i), Some((i+1) as i64), 1, (n_locations-1) as i64, false, None))
            });

            planning_stops_vec.push(current_stop);
        }

        return planning_stops_vec;
    }
}

impl<'a> CotwinBuilderTrait<TravelSchedule, EntityVariants<'a>, UtilityObjectVariants, HardSoftScore> for CotwinBuilder {

    fn new(use_incremental_score_calculation: bool) -> Self {
        Self {
            use_incremental_score_calculation: use_incremental_score_calculation,
        }
    }

    fn build_cotwin(&self, domain: TravelSchedule) -> Cotwin<EntityVariants<'a>, UtilityObjectVariants, HardSoftScore> {
        
        let mut cotwin = Cotwin::new();
        cotwin.add_planning_entities("path_stops".to_string(), Self::build_planning_stops(&domain));

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