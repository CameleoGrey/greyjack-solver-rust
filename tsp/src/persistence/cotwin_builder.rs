

use crate::domain::TravelSchedule;
use crate::cotwin::CotStop;
use crate::score::ScoreCalculator;
use greysplanner::cotwin::{Cotwin, CotwinEntityTrait, CotwinValueTypes, CotwinBuilderTrait};
use greysplanner::score_calculation::scores::HardSoftScore;
use greysplanner::variables::GPIntegerVar;
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
}


#[derive(Clone)]
pub struct CotwinBuilder {
    
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
                stop_id: CotwinValueTypes::PolarsAnyValue(AnyValue::UInt64(i as u64)),
                locations_vec_id: CotwinValueTypes::GPIntegerVar(GPIntegerVar::new(&format!("stop_{}_id", i), Some((i+1) as i64), 1, (n_locations-1) as i64, false, None))
            });

            planning_stops_vec.push(current_stop);
        }

        return planning_stops_vec;
    }
}

impl<'a> CotwinBuilderTrait<TravelSchedule, EntityVariants<'a>, UtilityObjectVariants, HardSoftScore> for CotwinBuilder {

    fn new() -> Self {
        Self{}
    }

    fn build_cotwin(&self, domain: TravelSchedule) -> Cotwin<EntityVariants<'a>, UtilityObjectVariants, HardSoftScore> {
        
        let mut tsp_cotwin = Cotwin::new();
        tsp_cotwin.add_planning_entities("path_stops".to_string(), Self::build_planning_stops(&domain));

        let mut score_calculator = ScoreCalculator::new();
        score_calculator.add_utility_object("distance_matrix".to_string(), UtilityObjectVariants::DistanceMatrix((domain.distance_matrix)));
        tsp_cotwin.add_score_calculator(score_calculator);

        return tsp_cotwin;
    }

}