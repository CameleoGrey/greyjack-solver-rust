

use greyjack::score_calculation::score_calculators::IncrementalScoreCalculator;
use greyjack::score_calculation::scores::HardMediumSoftScore;
use crate::persistence::cotwin_builder::UtilityObjectVariants;
use std::{collections::HashMap, collections::HashSet};
use polars::prelude::*;


pub struct VRPIncrementalScoreCalculator {

}

impl VRPIncrementalScoreCalculator {

    pub fn new() -> IncrementalScoreCalculator<UtilityObjectVariants, HardMediumSoftScore> {

        let mut score_calculator: IncrementalScoreCalculator<UtilityObjectVariants, HardMediumSoftScore> = IncrementalScoreCalculator::new();

        // interim variant, 5-15x faster (depends on settings) than plain score calculation, 
        //score_calculator.add_constraint("no_duplicating_stops_constraint".to_string(), Box::new(Self::no_duplicating_stops_constraint));
        //score_calculator.add_constraint("capacity_constraint".to_string(), Box::new(Self::capacity_constraint));
        //score_calculator.add_constraint("minimize_distance".to_string(), Box::new(Self::minimize_distance));
        //score_calculator.add_constraint("late_arrival_penalty".to_string(), Box::new(Self::late_arrival_penalty));

        // latest and fastest constraints formulation variant (~40% improvement comparable to the above variant, ~20x faster than plain)
        score_calculator.add_constraint("all_in_one_constraint".to_string(), Box::new(Self::all_in_one_constraint));        

        return score_calculator;
    }

    fn all_in_one_constraint(
        planning_entity_dfs: &HashMap<String, DataFrame>, 
        problem_fact_dfs: &HashMap<String, DataFrame>,
        delta_dfs: &HashMap<String, DataFrame>,
        utility_objects: &mut HashMap<String, UtilityObjectVariants>,
    ) -> Vec<HardMediumSoftScore> {

        let planning_stops_df = planning_entity_dfs["planning_stops"].clone();
        let path_stops_deltas_df = delta_dfs["planning_stops"].clone();
        let vehicles_info; match &utility_objects["vehicles_info"] {UtilityObjectVariants::VehiclesInfo(x) => vehicles_info = x, _ => panic!("dragons")}
        let customers_info; match &utility_objects["customers_info"] {UtilityObjectVariants::CustomersInfo(x) => customers_info = x, _ => panic!("dragons")}
        let distance_matrix: &Vec<Vec<f64>>; match &utility_objects["distance_matrix"] {UtilityObjectVariants::DistanceMatrix(x) => distance_matrix = &x, _ => panic!("dragons")}
        let is_time_windowed: bool; match &utility_objects["time_windowed"] {UtilityObjectVariants::IsTimeWindowed(x) => is_time_windowed = *x, _ => panic!("dragons")}

        let k_vehicles = vehicles_info.len();
        let vehicle_depot_ids: Vec<usize> = vehicles_info.iter().map(|vi| vi.depot_vec_id).collect();
        let vehicle_null_penalties: Vec<f64> = vec![0.0; k_vehicles];
        let null_trip_demands: Vec<u64> = vec![0; k_vehicles];
        let customers_vec_len = planning_stops_df["customer_id"].len();

        let candidate_vehicle_ids: Vec<usize> = planning_stops_df["vehicle_id"].i64().unwrap().to_vec().iter().map(|id| id.unwrap() as usize).collect();
        let candidate_customer_ids: Vec<usize> = planning_stops_df["customer_id"].i64().unwrap().to_vec().iter().map(|id| id.unwrap() as usize).collect();

        let scores: Vec<HardMediumSoftScore> = 
        path_stops_deltas_df
        .partition_by(["sample_id"], false).unwrap()
        .iter().map(|sample_df| {

            let mut sample_vehicle_ids = candidate_vehicle_ids.clone();
            let mut sample_customer_ids = candidate_customer_ids.clone();
            let mut sum_trip_demands = null_trip_demands.clone();
            let delta_row_ids: Vec<usize> = sample_df["candidate_df_row_id"].u64().unwrap().to_vec().iter().map(|id| id.unwrap() as usize).collect();
            let vehicle_delta_ids: Vec<usize> = sample_df["vehicle_id"].i64().unwrap().to_vec().iter().map(|id| id.unwrap() as usize).collect();
            let customer_delta_ids: Vec<usize> = sample_df["customer_id"].i64().unwrap().to_vec().iter().map(|id| id.unwrap() as usize).collect();
            
            delta_row_ids.iter()
            .zip(vehicle_delta_ids.iter())
            .zip(customer_delta_ids.iter())
            .for_each(|((row_id, dv_id), cd_id )| {
                sample_vehicle_ids[*row_id] = *dv_id;
                sample_customer_ids[*row_id] = *cd_id;
            });

            // no_duplicating_stops_constraint
            let sample_unique_customer_ids: HashSet<usize> = sample_customer_ids.iter().map(|customer_id| *customer_id).collect();
            let unique_stops_penalty = 1000.0 * (customers_vec_len - sample_unique_customer_ids.len()) as f64;

            //capacity_constraint
            sample_vehicle_ids.iter()
            .zip(sample_customer_ids.iter())
            .for_each(|(v_id, c_id)| {
                sum_trip_demands[*v_id] += customers_info[*c_id].demand;
            });
            let capacity_differences: Vec<i64> = (0..k_vehicles).into_iter().map(|v_id| vehicles_info[v_id].capacity as i64 - sum_trip_demands[v_id] as i64).collect();
            let capacity_penalty: i64 = capacity_differences.iter().filter(|x| **x < 0).map(|x| x.abs()).sum();


            let mut vehicle_stops: Vec<Vec<usize>> = vec![Vec::new(); k_vehicles];
            sample_vehicle_ids.iter()
            .zip(sample_customer_ids.iter())
            .for_each(|(v_id, c_id)| {
                vehicle_stops[*v_id].push(*c_id);
            });

            let mut vehicle_distances = vehicle_null_penalties.clone();
            let mut vehicle_time_penalties = vehicle_null_penalties.clone();
            vehicle_distances.iter_mut()
            .zip(vehicle_time_penalties.iter_mut())
            .enumerate().for_each(|(v_id, (current_distance, current_time_penalty))| {
                if vehicle_stops[v_id].len() != 0 {

                    //minimize_distance
                    let last_id = vehicle_stops[v_id].len()-1;
                    *current_distance += distance_matrix[vehicle_depot_ids[v_id]][vehicle_stops[v_id][0]];
                    *current_distance += distance_matrix[vehicle_stops[v_id][last_id]][vehicle_depot_ids[v_id]];
                    *current_distance += (1..=last_id).into_iter().fold(0.0, |interim_distance, i| interim_distance + distance_matrix[vehicle_stops[v_id][i-1]][vehicle_stops[v_id][i]]);

                    if is_time_windowed {
                        //late_arrival_penalty
                        let n_stops = vehicle_stops[v_id].len();
                        let work_day_start: u64 = vehicles_info[v_id].work_day_start;
                        let work_day_end: u64 = vehicles_info[v_id].work_day_end;
                        let mut current_arrival_time = work_day_start;
                        for i in 0..n_stops {

                            let customer_i_start = customers_info[vehicle_stops[v_id][i]].time_window_start;
                            let customer_i_end = customers_info[vehicle_stops[v_id][i]].time_window_end;
                            let customer_i_service_time = customers_info[vehicle_stops[v_id][i]].service_time;
                            current_arrival_time = std::cmp::max(current_arrival_time, customer_i_start);
                            if current_arrival_time > customer_i_end + customer_i_service_time {
                                *current_time_penalty += (current_arrival_time - (customer_i_end + customer_i_service_time)) as f64;
                            }
                            current_arrival_time += customer_i_service_time;
                        }
                        if current_arrival_time > work_day_end {
                            *current_time_penalty += (current_arrival_time - work_day_end) as f64;
                        }
                    }
                }
            });

            let sum_distance: f64 = vehicle_distances.iter().sum();
            let sum_time_penalty: f64 = vehicle_time_penalties.iter().sum();

            HardMediumSoftScore::new(unique_stops_penalty + capacity_penalty as f64, sum_time_penalty, sum_distance)

        }).collect();

        return scores;


    }


    fn no_duplicating_stops_constraint(
        planning_entity_dfs: &HashMap<String, DataFrame>, 
        problem_fact_dfs: &HashMap<String, DataFrame>,
        delta_dfs: &HashMap<String, DataFrame>,
        utility_objects: &mut HashMap<String, UtilityObjectVariants>,
    ) -> Vec<HardMediumSoftScore> {

        let planning_stops_df = planning_entity_dfs["planning_stops"].clone();
        let path_stops_deltas_df = delta_dfs["planning_stops"].clone();

        let customers_vec_len = planning_stops_df["customer_id"].len();
        let customer_ids: Vec<usize> = 
        planning_stops_df["customer_id"]
        .i64().unwrap().to_vec()
        .iter().map(|costumer_id| costumer_id.unwrap() as usize)
        .collect();
        
        let mut scores: Vec<HardMediumSoftScore> = path_stops_deltas_df
        .partition_by(["sample_id"], false).unwrap()
        .iter().enumerate().map(|(i, sample_df)| {
            let mut sample_customer_ids: Vec<usize> = 
            sample_df["customer_id"]
            .i64().unwrap().to_vec()
            .iter().map(|customer_id| customer_id.unwrap() as usize)
            .collect();

            let mut sample_row_ids: Vec<usize> = 
            sample_df["candidate_df_row_id"]
            .u64().unwrap().to_vec()
            .iter().map(|row_id| row_id.unwrap() as usize)
            .collect();
            
            let mut changed_customer_ids = customer_ids.clone();
            sample_row_ids.iter().zip(sample_customer_ids.iter()).into_iter().for_each(|(row_id, customer_id)| {
                changed_customer_ids[*row_id] = *customer_id;
            });

            let sample_unique_customer_ids: HashSet<usize> = changed_customer_ids.iter().map(|customer_id| *customer_id).collect();

            HardMediumSoftScore::new(1000.0 * (customers_vec_len - sample_unique_customer_ids.len()) as f64, 0.0, 0.0)
        }).collect();

        return scores;

    }

    fn capacity_constraint(
        planning_entity_dfs: &HashMap<String, DataFrame>, 
        problem_fact_dfs: &HashMap<String, DataFrame>,
        delta_dfs: &HashMap<String, DataFrame>,
        utility_objects: &mut HashMap<String, UtilityObjectVariants>,
    ) -> Vec<HardMediumSoftScore> {

        let planning_stops_df = planning_entity_dfs["planning_stops"].clone();
        let path_stops_deltas_df = delta_dfs["planning_stops"].clone();
        let vehicles_info; match &utility_objects["vehicles_info"] {UtilityObjectVariants::VehiclesInfo(x) => vehicles_info = x, _ => panic!("dragons")}
        let customers_info; match &utility_objects["customers_info"] {UtilityObjectVariants::CustomersInfo(x) => customers_info = x, _ => panic!("dragons")}

        let k_vehicles = vehicles_info.len();
        let null_trip_demands: Vec<u64> = vec![0; k_vehicles];

        let candidate_vehicle_ids: Vec<usize> = planning_stops_df["vehicle_id"].i64().unwrap().to_vec().iter().map(|id| id.unwrap() as usize).collect();
        let candidate_customer_ids: Vec<usize> = planning_stops_df["customer_id"].i64().unwrap().to_vec().iter().map(|id| id.unwrap() as usize).collect();

        let scores: Vec<HardMediumSoftScore> = 
        path_stops_deltas_df
        .partition_by(["sample_id"], false).unwrap()
        .iter().map(|sample_df| {

            let mut sample_vehicle_ids = candidate_vehicle_ids.clone();
            let mut sample_customer_ids = candidate_customer_ids.clone();
            let mut sum_trip_demands = null_trip_demands.clone();
            let delta_row_ids: Vec<usize> = sample_df["candidate_df_row_id"].u64().unwrap().to_vec().iter().map(|id| id.unwrap() as usize).collect();
            let vehicle_delta_ids: Vec<usize> = sample_df["vehicle_id"].i64().unwrap().to_vec().iter().map(|id| id.unwrap() as usize).collect();
            let customer_delta_ids: Vec<usize> = sample_df["customer_id"].i64().unwrap().to_vec().iter().map(|id| id.unwrap() as usize).collect();
            
            delta_row_ids.iter()
            .zip(vehicle_delta_ids.iter())
            .zip(customer_delta_ids.iter())
            .for_each(|((row_id, dv_id), cd_id )| {
                sample_vehicle_ids[*row_id] = *dv_id;
                sample_customer_ids[*row_id] = *cd_id;
            });
            
            sample_vehicle_ids.iter()
            .zip(sample_customer_ids.iter())
            .for_each(|(v_id, c_id)| {
                sum_trip_demands[*v_id] += customers_info[*c_id].demand;
            });

            let capacity_differences: Vec<i64> = (0..k_vehicles).into_iter().map(|v_id| vehicles_info[v_id].capacity as i64 - sum_trip_demands[v_id] as i64).collect();
            let capacity_penalty: i64 = capacity_differences.iter().filter(|x| **x < 0).map(|x| x.abs()).sum();

            HardMediumSoftScore::new(capacity_penalty as f64, 0.0, 0.0)

        }).collect();

        return scores;

    }

    fn minimize_distance(
        planning_entity_dfs: &HashMap<String, DataFrame>, 
        problem_fact_dfs: &HashMap<String, DataFrame>,
        delta_dfs: &HashMap<String, DataFrame>,
        utility_objects: &mut HashMap<String, UtilityObjectVariants>,
    ) -> Vec<HardMediumSoftScore> {

        let planning_stops_df = planning_entity_dfs["planning_stops"].clone();
        let path_stops_deltas_df = delta_dfs["planning_stops"].clone();
        let vehicles_info; match &utility_objects["vehicles_info"] {UtilityObjectVariants::VehiclesInfo(x) => vehicles_info = x, _ => panic!("dragons")}
        //let customers_info; match &utility_objects["customers_info"] {UtilityObjectVariants::CustomersInfo(x) => customers_info = x, _ => panic!("dragons")}
        let distance_matrix: &Vec<Vec<f64>>; match &utility_objects["distance_matrix"] {UtilityObjectVariants::DistanceMatrix(x) => distance_matrix = &x, _ => panic!("There are the dragons")}

        let k_vehicles = vehicles_info.len();
        let vehicle_depot_ids: Vec<usize> = vehicles_info.iter().map(|vi| vi.depot_vec_id).collect();
        let null_vehicle_distances: Vec<f64> = vec![0.0; k_vehicles];

        let candidate_vehicle_ids: Vec<usize> = planning_stops_df["vehicle_id"].i64().unwrap().to_vec().iter().map(|id| id.unwrap() as usize).collect();
        let candidate_customer_ids: Vec<usize> = planning_stops_df["customer_id"].i64().unwrap().to_vec().iter().map(|id| id.unwrap() as usize).collect();

        let scores: Vec<HardMediumSoftScore> = 
        path_stops_deltas_df
        .partition_by(["sample_id"], false).unwrap()
        .iter().map(|sample_df| {

            let mut sample_vehicle_ids = candidate_vehicle_ids.clone();
            let mut sample_customer_ids = candidate_customer_ids.clone();
            let delta_row_ids: Vec<usize> = sample_df["candidate_df_row_id"].u64().unwrap().to_vec().iter().map(|id| id.unwrap() as usize).collect();
            let vehicle_delta_ids: Vec<usize> = sample_df["vehicle_id"].i64().unwrap().to_vec().iter().map(|id| id.unwrap() as usize).collect();
            let customer_delta_ids: Vec<usize> = sample_df["customer_id"].i64().unwrap().to_vec().iter().map(|id| id.unwrap() as usize).collect();
            
            delta_row_ids.iter()
            .zip(vehicle_delta_ids.iter())
            .zip(customer_delta_ids.iter())
            .for_each(|((row_id, dv_id), cd_id )| {
                sample_vehicle_ids[*row_id] = *dv_id;
                sample_customer_ids[*row_id] = *cd_id;
            });

            let mut vehicle_stops: Vec<Vec<usize>> = vec![Vec::new(); k_vehicles];
            sample_vehicle_ids.iter()
            .zip(sample_customer_ids.iter())
            .for_each(|(v_id, c_id)| {
                vehicle_stops[*v_id].push(*c_id);
            });

            let mut vehicle_distances = null_vehicle_distances.clone();
            vehicle_distances.iter_mut().enumerate().for_each(|(v_id, current_distance)| {
                if vehicle_stops[v_id].len() != 0 {
                    let last_id = vehicle_stops[v_id].len()-1;
                    *current_distance += distance_matrix[vehicle_depot_ids[v_id]][vehicle_stops[v_id][0]];
                    *current_distance += distance_matrix[vehicle_stops[v_id][last_id]][vehicle_depot_ids[v_id]];
                    *current_distance += (1..=last_id).into_iter().fold(0.0, |interim_distance, i| interim_distance + distance_matrix[vehicle_stops[v_id][i-1]][vehicle_stops[v_id][i]]);
                }
            });
            let sum_distance = vehicle_distances.iter().sum();

            HardMediumSoftScore::new(0.0, 0.0, sum_distance)

        }).collect();

        return scores;

    }


    fn late_arrival_penalty(
        planning_entity_dfs: &HashMap<String, DataFrame>, 
        problem_fact_dfs: &HashMap<String, DataFrame>,
        delta_dfs: &HashMap<String, DataFrame>,
        utility_objects: &mut HashMap<String, UtilityObjectVariants>,
    ) -> Vec<HardMediumSoftScore> {

        let planning_stops_df = planning_entity_dfs["planning_stops"].clone();
        let path_stops_deltas_df = delta_dfs["planning_stops"].clone();
        let vehicles_info; match &utility_objects["vehicles_info"] {UtilityObjectVariants::VehiclesInfo(x) => vehicles_info = x, _ => panic!("dragons")}
        let customers_info; match &utility_objects["customers_info"] {UtilityObjectVariants::CustomersInfo(x) => customers_info = x, _ => panic!("dragons")}
        let distance_matrix: &Vec<Vec<f64>>; match &utility_objects["distance_matrix"] {UtilityObjectVariants::DistanceMatrix(x) => distance_matrix = &x, _ => panic!("There are the dragons")}

        let k_vehicles = vehicles_info.len();
        let vehicle_depot_ids: Vec<usize> = vehicles_info.iter().map(|vi| vi.depot_vec_id).collect();
        let vehicle_null_penalties: Vec<f64> = vec![0.0; k_vehicles];

        let candidate_vehicle_ids: Vec<usize> = planning_stops_df["vehicle_id"].i64().unwrap().to_vec().iter().map(|id| id.unwrap() as usize).collect();
        let candidate_customer_ids: Vec<usize> = planning_stops_df["customer_id"].i64().unwrap().to_vec().iter().map(|id| id.unwrap() as usize).collect();

        let scores: Vec<HardMediumSoftScore> = 
        path_stops_deltas_df
        .partition_by(["sample_id"], false).unwrap()
        .iter().map(|sample_df| {

            let mut sample_vehicle_ids = candidate_vehicle_ids.clone();
            let mut sample_customer_ids = candidate_customer_ids.clone();
            let delta_row_ids: Vec<usize> = sample_df["candidate_df_row_id"].u64().unwrap().to_vec().iter().map(|id| id.unwrap() as usize).collect();
            let vehicle_delta_ids: Vec<usize> = sample_df["vehicle_id"].i64().unwrap().to_vec().iter().map(|id| id.unwrap() as usize).collect();
            let customer_delta_ids: Vec<usize> = sample_df["customer_id"].i64().unwrap().to_vec().iter().map(|id| id.unwrap() as usize).collect();
            
            delta_row_ids.iter()
            .zip(vehicle_delta_ids.iter())
            .zip(customer_delta_ids.iter())
            .for_each(|((row_id, dv_id), cd_id )| {
                sample_vehicle_ids[*row_id] = *dv_id;
                sample_customer_ids[*row_id] = *cd_id;
            });

            let mut vehicle_stops: Vec<Vec<usize>> = vec![Vec::new(); k_vehicles];
            sample_vehicle_ids.iter()
            .zip(sample_customer_ids.iter())
            .for_each(|(v_id, c_id)| {
                vehicle_stops[*v_id].push(*c_id);
            });

            let mut vehicle_time_penalties = vehicle_null_penalties.clone();
            vehicle_time_penalties.iter_mut().enumerate().for_each(|(v_id, current_time_penalty)| {
                if vehicle_stops[v_id].len() != 0 {

                    let n_stops = vehicle_stops[v_id].len();
                    let work_day_start: u64 = vehicles_info[v_id].work_day_start;
                    let work_day_end: u64 = vehicles_info[v_id].work_day_end;
                    let mut current_arrival_time = work_day_start;

                    for i in 0..n_stops {

                        let customer_i_start = customers_info[vehicle_stops[v_id][i]].time_window_start;
                        let customer_i_end = customers_info[vehicle_stops[v_id][i]].time_window_end;
                        let customer_i_service_time = customers_info[vehicle_stops[v_id][i]].service_time;


                        current_arrival_time = std::cmp::max(current_arrival_time, customer_i_start);
                        if current_arrival_time > customer_i_end + customer_i_service_time {
                            *current_time_penalty += (current_arrival_time - (customer_i_end + customer_i_service_time)) as f64;
                        }

                        current_arrival_time += customer_i_service_time;
                    }

                    if current_arrival_time > work_day_end {
                        *current_time_penalty += (current_arrival_time - work_day_end) as f64;
                    }
                }
            });
            let sum_time_penalty = vehicle_time_penalties.iter().sum();

            HardMediumSoftScore::new(0.0, sum_time_penalty, 0.0)

        }).collect();

        return scores;
    }
}