

use crate::domain::VehicleRoutingPlan;
use crate::cotwin::{CotStop, CotCustomer, CotVehicle};
use crate::score::ScoreCalculator;
use greysplanner::cotwin::{Cotwin, CotwinEntityTrait, CotwinValueTypes, CotwinBuilderTrait};
use greysplanner::score_calculation::scores::HardSoftScore;
use greysplanner::variables::GPIntegerVar;
use polars::frame::DataFrame;
use std::collections::HashMap;
use polars::datatypes::AnyValue;


pub enum EntityVariants<'a> {
    CotCustomer(CotCustomer<'a>),
    CotVehicle(CotVehicle<'a>),
    CotStop(CotStop<'a>),
}

impl<'a> CotwinEntityTrait for EntityVariants<'a> {
    fn to_hash_map(&self) -> HashMap<String, CotwinValueTypes> {
        match self {
            EntityVariants::CotCustomer(x) => return x.to_hash_map(),
            EntityVariants::CotVehicle(x) =>  return x.to_hash_map(),
            EntityVariants::CotStop(x) =>  return x.to_hash_map(),
        }
    }
}

pub enum UtilityObjectVariants {
    DistanceMatrix(Vec<Vec<f64>>),
    DataFrame(DataFrame)
}


#[derive(Clone)]
pub struct CotwinBuilder {
    
}

impl CotwinBuilder {

    fn build_problem_fact_customers<'a>(domain: &VehicleRoutingPlan) -> Vec<EntityVariants<'a>> {

        let customers_vec = &domain.customers_vec;
        let n_customers = customers_vec.len();
        let n_depots = domain.depot_vec.len();

        let mut problem_fact_customers: Vec<EntityVariants<'a>> = Vec::new();
        for i in n_depots..n_customers {
            let current_customer = &customers_vec[i];
            let customer_id = CotwinValueTypes::PolarsAnyValue(AnyValue::Int64(i as i64));
            let demand = CotwinValueTypes::PolarsAnyValue(AnyValue::UInt64(current_customer.demand));
            let time_window_start = CotwinValueTypes::PolarsAnyValue(AnyValue::UInt64(current_customer.time_window_start));
            let time_window_end = CotwinValueTypes::PolarsAnyValue(AnyValue::UInt64(current_customer.time_window_end));
            let service_time = CotwinValueTypes::PolarsAnyValue(AnyValue::UInt64(current_customer.service_time));
            let cot_customer = EntityVariants::CotCustomer(CotCustomer::new( customer_id, demand, time_window_start, time_window_end, service_time));
            problem_fact_customers.push( cot_customer );
        }

        return problem_fact_customers;
    }

    fn build_problem_fact_vehicles<'a>(domain: &VehicleRoutingPlan) -> Vec<EntityVariants<'a>> {

        let mut problem_fact_vehicles: Vec<EntityVariants<'a>> = Vec::new();
        for (i, domain_vehicle) in domain.vehicles.iter().enumerate() {
            let vehicle_id = CotwinValueTypes::PolarsAnyValue(AnyValue::Int64(i as i64));
            let depot_vec_id = CotwinValueTypes::PolarsAnyValue(AnyValue::UInt64(domain_vehicle.depot_vec_id as u64));
            let capacity = CotwinValueTypes::PolarsAnyValue(AnyValue::UInt64(domain_vehicle.capacity));
            let work_day_start = CotwinValueTypes::PolarsAnyValue(AnyValue::UInt64(domain_vehicle.work_day_start));
            let work_day_end = CotwinValueTypes::PolarsAnyValue(AnyValue::UInt64(domain_vehicle.work_day_end));
            let cot_vehicle = EntityVariants::CotVehicle(CotVehicle::new( vehicle_id, capacity, depot_vec_id, work_day_start, work_day_end));
            problem_fact_vehicles.push( cot_vehicle );
        }

        return problem_fact_vehicles;
    }
    
    fn build_planning_stops<'a>(domain: &VehicleRoutingPlan) -> Vec<EntityVariants<'a>> {

        let n_depots = domain.depot_vec.len();
        let n_customers = domain.customers_vec.len();
        let k_vehicles = domain.vehicles.len();
        let mut planning_stops_vec: Vec<EntityVariants<'a>> = Vec::new();
        for i in n_depots..n_customers {
            //let current_location = &domain.locations_vec[i];
            let current_stop = EntityVariants::CotStop(CotStop {
                vehicle_id: CotwinValueTypes::GPIntegerVar(GPIntegerVar::new(&format!("planning_vehicle_id_{}", i), None, 0, (k_vehicles-1) as i64, false, Some(vec!["vehicle_assignment".to_string(), "common".to_string()]))),
                customer_id: CotwinValueTypes::GPIntegerVar(GPIntegerVar::new(&format!("planning_customer_id_{}", i), None, n_depots as i64, (n_customers-1) as i64, false, Some(vec!["customer_assignment".to_string(), "common".to_string()])))
            });

            planning_stops_vec.push(current_stop);
        }

        return planning_stops_vec;
    }
}

impl<'a> CotwinBuilderTrait<VehicleRoutingPlan, EntityVariants<'a>, UtilityObjectVariants, HardSoftScore> for CotwinBuilder {

    fn new() -> Self {
        Self{}
    }

    fn build_cotwin(&self, domain: VehicleRoutingPlan) -> Cotwin<EntityVariants<'a>, UtilityObjectVariants, HardSoftScore> {
        
        let mut cotwin = Cotwin::new();
        cotwin.add_problem_facts("vehicles".to_string(), Self::build_problem_fact_vehicles(&domain));
        cotwin.add_problem_facts("customers".to_string(), Self::build_problem_fact_customers(&domain));
        cotwin.add_planning_entities("planning_stops".to_string(), Self::build_planning_stops(&domain));

        let mut score_calculator = ScoreCalculator::new();
        score_calculator.add_utility_object("distance_matrix".to_string(), UtilityObjectVariants::DistanceMatrix(domain.distance_matrix));
        if domain.time_windowed == false {
            score_calculator.remove_constraint("late_arrival_penalty".to_string());
        }

        cotwin.add_score_calculator(score_calculator);

        return cotwin;
    }

}