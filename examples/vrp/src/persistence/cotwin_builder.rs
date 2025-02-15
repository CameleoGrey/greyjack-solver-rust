

use crate::domain::{Customer, Vehicle, VehicleRoutingPlan};
use crate::cotwin::{CotStop, CotCustomer, CotVehicle};
use crate::score::{VRPIncrementalScoreCalculator, VRPPlainScoreCalculator};
use greyjack::cotwin::{Cotwin, CotwinEntityTrait, CotwinValueTypes, CotwinBuilderTrait};
use greyjack::score_calculation::score_calculators::score_calculator_variants::ScoreCalculatorVariants;
use greyjack::score_calculation::scores::HardMediumSoftScore;
use greyjack::variables::GJInteger;
use polars::frame::DataFrame;
use std::collections::HashMap;
use polars::datatypes::AnyValue;


pub enum EntityVariants<'a> {
    CotCustomer(CotCustomer<'a>),
    CotVehicle(CotVehicle<'a>),
    CotStop(CotStop<'a>),
}

impl<'a> CotwinEntityTrait for EntityVariants<'a> {
    fn to_vec(&self) -> Vec<(String, CotwinValueTypes)> {
        match self {
            EntityVariants::CotCustomer(x) => return x.to_vec(),
            EntityVariants::CotVehicle(x) =>  return x.to_vec(),
            EntityVariants::CotStop(x) =>  return x.to_vec(),
        }
    }
}

pub enum UtilityObjectVariants {
    DistanceMatrix(Vec<Vec<f64>>),
    DataFrame(DataFrame),
    
    // for incremental score calculator only to avoid joins
    VehiclesInfo(Vec<Vehicle>),
    CustomersInfo(Vec<Customer>),
    IsTimeWindowed(bool), // for latest constraints formulation
}


#[derive(Clone)]
pub struct CotwinBuilder {
    use_incremental_score_calculation: bool,
}

impl CotwinBuilder {

    fn build_problem_fact_customers<'a>(domain: &VehicleRoutingPlan) -> Vec<EntityVariants<'a>> {

        let customers_vec = &domain.customers_vec;
        let n_customers = customers_vec.len();
        let n_depots = domain.depot_vec.len();
        let mut demand_utility: Vec<u64> = vec![0; n_customers];

        let mut problem_fact_customers: Vec<EntityVariants<'a>> = Vec::new();
        for i in n_depots..n_customers {
            let current_customer = &customers_vec[i];
            let customer_id = CotwinValueTypes::PAV(AnyValue::Int64(i as i64));
            let demand = CotwinValueTypes::PAV(AnyValue::UInt64(current_customer.demand));
            let time_window_start = CotwinValueTypes::PAV(AnyValue::UInt64(current_customer.time_window_start));
            let time_window_end = CotwinValueTypes::PAV(AnyValue::UInt64(current_customer.time_window_end));
            let service_time = CotwinValueTypes::PAV(AnyValue::UInt64(current_customer.service_time));
            let cot_customer = EntityVariants::CotCustomer(CotCustomer::new( customer_id, demand, time_window_start, time_window_end, service_time));
            problem_fact_customers.push( cot_customer );

            demand_utility[i] = current_customer.demand;
        }

        return problem_fact_customers;
    }

    fn build_problem_fact_vehicles<'a>(domain: &VehicleRoutingPlan) -> Vec<EntityVariants<'a>> {

        let mut problem_fact_vehicles: Vec<EntityVariants<'a>> = Vec::new();
        for (i, domain_vehicle) in domain.vehicles.iter().enumerate() {
            let vehicle_id = CotwinValueTypes::PAV(AnyValue::Int64(i as i64));
            let depot_vec_id = CotwinValueTypes::PAV(AnyValue::UInt64(domain_vehicle.depot_vec_id as u64));
            let capacity = CotwinValueTypes::PAV(AnyValue::UInt64(domain_vehicle.capacity));
            let work_day_start = CotwinValueTypes::PAV(AnyValue::UInt64(domain_vehicle.work_day_start));
            let work_day_end = CotwinValueTypes::PAV(AnyValue::UInt64(domain_vehicle.work_day_end));
            let cot_vehicle = EntityVariants::CotVehicle(CotVehicle::new( vehicle_id, capacity, depot_vec_id, work_day_start, work_day_end));
            problem_fact_vehicles.push( cot_vehicle );
        }

        return problem_fact_vehicles;
    }
    
    fn build_planning_stops<'a>(domain: &VehicleRoutingPlan, is_already_initialized: bool) -> Vec<EntityVariants<'a>> {

        let n_depots = domain.depot_vec.len();
        let n_locations = domain.customers_vec.len();
        let k_vehicles = domain.vehicles.len();
        let mut planning_stops_vec: Vec<EntityVariants<'a>> = Vec::new();

        let mut initial_vehicle_ids: Vec<Option<i64>> = vec![None; n_locations - n_depots];
        let mut initial_customer_ids: Vec<Option<i64>> = vec![None; n_locations - n_depots];

        if is_already_initialized {
            let mut i = 0;
            for k in (0..k_vehicles) {
                let vehicle_k = &domain.vehicles[k];
                for customer in &vehicle_k.customers {
                    initial_vehicle_ids[i] = Some(k as i64);
                    initial_customer_ids[i] = Some(customer.vec_id as i64);
                    i += 1;
                }
            }
        }

        for i in n_depots..n_locations {
            //let current_location = &domain.locations_vec[i];
            let current_stop = EntityVariants::CotStop(CotStop {
                // init: Some((i % k_vehicles) as i64)
                vehicle_id: CotwinValueTypes::GJI(GJInteger::new(
                    &format!("planning_vehicle_id_{}", i), 
                    initial_vehicle_ids[i - n_depots], 0, (k_vehicles-1) as i64, false, 
                    Some(vec!["vehicle_assignment".to_string()]))),
                // init: Some(i as i64)
                customer_id: CotwinValueTypes::GJI(GJInteger::new(
                    &format!("planning_customer_id_{}", i), 
                    initial_customer_ids[i - n_depots], n_depots as i64, (n_locations-1) as i64, false, 
                    Some(vec!["customer_assignment".to_string()])))
            });

            planning_stops_vec.push(current_stop);
        }

        return planning_stops_vec;
    }

    fn build_utility_customers_info(domain: &VehicleRoutingPlan) -> Vec<Customer> {

        let utility_customers = domain.customers_vec.clone();

        return utility_customers;
    }

    fn build_utility_vehicles_info(domain: &VehicleRoutingPlan) -> Vec<Vehicle> {

        let mut utility_vehicles = domain.vehicles.clone();

        // we need only common vehicle infos for score calculation
        for i in 0..utility_vehicles.len() {
            utility_vehicles[i].customers = Vec::new();
        }

        return utility_vehicles;
    }
}

impl<'a> CotwinBuilderTrait<VehicleRoutingPlan, EntityVariants<'a>, UtilityObjectVariants, HardMediumSoftScore> for CotwinBuilder {

    fn new(use_incremental_score_calculation: bool) -> Self {
        Self {
            use_incremental_score_calculation: use_incremental_score_calculation
        }
    }

    fn build_cotwin(&self, domain: VehicleRoutingPlan, is_already_initialized: bool) -> Cotwin<EntityVariants<'a>, UtilityObjectVariants, HardMediumSoftScore> {
        
        let mut cotwin = Cotwin::new();
        cotwin.add_problem_facts("vehicles".to_string(), Self::build_problem_fact_vehicles(&domain));
        cotwin.add_problem_facts("customers".to_string(), Self::build_problem_fact_customers(&domain));
        cotwin.add_planning_entities("planning_stops".to_string(), Self::build_planning_stops(&domain, is_already_initialized));

        if self.use_incremental_score_calculation {
            let mut score_calculator = VRPIncrementalScoreCalculator::new();
            score_calculator.add_utility_object("customers_info".to_string(), UtilityObjectVariants::CustomersInfo(Self::build_utility_customers_info(&domain)));
            score_calculator.add_utility_object("vehicles_info".to_string(), UtilityObjectVariants::VehiclesInfo(Self::build_utility_vehicles_info(&domain)));
            score_calculator.add_utility_object("time_windowed".to_string(), UtilityObjectVariants::IsTimeWindowed(domain.time_windowed));
            score_calculator.add_utility_object("distance_matrix".to_string(), UtilityObjectVariants::DistanceMatrix(domain.distance_matrix));
           if domain.time_windowed == false {
                score_calculator.remove_constraint("late_arrival_penalty".to_string());
            }
            cotwin.add_score_calculator(ScoreCalculatorVariants::ISC(score_calculator));   

        } else {
            let mut score_calculator = VRPPlainScoreCalculator::new();
            score_calculator.add_utility_object("distance_matrix".to_string(), UtilityObjectVariants::DistanceMatrix(domain.distance_matrix));
            if domain.time_windowed == false {
                score_calculator.remove_constraint("late_arrival_penalty".to_string());
            }
            cotwin.add_score_calculator(ScoreCalculatorVariants::PSC(score_calculator));   
        }

        return cotwin;
    }

}