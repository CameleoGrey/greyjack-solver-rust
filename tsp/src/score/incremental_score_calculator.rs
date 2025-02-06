

use greyjack::score_calculation::{score_calculators::IncrementalScoreCalculator, scores::ScoreTrait};
use greyjack::score_calculation::scores::HardSoftScore;
use crate::persistence::cotwin_builder::UtilityObjectVariants;
use std::collections::HashMap;
use polars::prelude::*;
use ndarray::{Array, Array2};
use std::collections::HashSet;


pub struct TSPIncrementalScoreCalculator {

}

impl TSPIncrementalScoreCalculator {

    pub fn new() -> IncrementalScoreCalculator<UtilityObjectVariants, HardSoftScore> {

        let mut score_calculator: IncrementalScoreCalculator<UtilityObjectVariants, HardSoftScore> = IncrementalScoreCalculator::new();

        //score_calculator.add_prescoring_function("build_deltas_hashmap".to_string(), Box::new(Self::build_deltas_map));
        score_calculator.add_constraint("no_duplicating_stops_constraint".to_string(), Box::new(Self::no_duplicating_stops_constraint));
        score_calculator.add_constraint("minimize_distance".to_string(), Box::new(Self::minimize_distance));

        return score_calculator;
    }

    /*fn build_deltas_map(
        planning_entity_dfs: &HashMap<String, DataFrame>, 
        problem_fact_dfs: &HashMap<String, DataFrame>,
        delta_dfs: &HashMap<String, DataFrame>,
        utility_objects: &mut HashMap<String, UtilityObjectVariants>,
    ) {
        let path_stops_deltas_df = delta_dfs["path_stops"].clone();

        let mut sample_ids: Vec<usize> = 
        path_stops_deltas_df["sample_id"]
        .u64().unwrap().to_vec()
        .iter().map(|sample_id| sample_id.unwrap() as usize)
        .collect();

        let row_ids: Vec<usize> = 
        path_stops_deltas_df["row_id"]
        .u64().unwrap().to_vec()
        .iter().map(|row_id| row_id.unwrap() as usize)
        .collect();
    
        let location_ids: Vec<usize> = 
        path_stops_deltas_df["location_vec_id"]
        .i64().unwrap().to_vec()
        .iter().map(|loc_id| loc_id.unwrap() as usize)
        .collect();

        let mut srl_deltas_map: Vec<HashMap<usize, usize>> = Vec::new();
        srl_deltas_map = vec![HashMap::new(); sample_ids.len()];
        sample_ids.iter()
        .zip(row_ids.iter())
        .zip(location_ids.iter())
        .for_each(|((sample_id, row_id), loc_id)| {
            srl_deltas_map[*sample_id].insert(*row_id, *loc_id);
        });
        utility_objects.insert("srl_deltas_map".to_string(), UtilityObjectVariants::DeltasMap(srl_deltas_map));
    }

    fn no_duplicating_stops_constraint(
        planning_entity_dfs: &HashMap<String, DataFrame>, 
        problem_fact_dfs: &HashMap<String, DataFrame>,
        delta_dfs: &HashMap<String, DataFrame>,
        utility_objects: &mut HashMap<String, UtilityObjectVariants>,
    ) -> Vec<HardSoftScore> {

        /*
        Works, but slower than "non clean" version on big deltas.
        */

        let path_stops_df = planning_entity_dfs["path_stops"].clone();
        let srl_deltas_map;
        match &utility_objects["srl_deltas_map"] {
            UtilityObjectVariants::DeltasMap(dm) => srl_deltas_map = dm,
            _ => panic!("dragons")
        }

        let location_ids: Vec<usize> = 
        path_stops_df["location_vec_id"]
        .i64().unwrap().to_vec()
        .iter().map(|loc_id| loc_id.unwrap() as usize)
        .collect();
        let unique_native_loc_ids: HashSet<usize> = location_ids.iter().map(|x| *x).collect();

        let path_stops_deltas_df = delta_dfs["path_stops"].clone();
        let n_samples = path_stops_deltas_df["sample_id"].n_unique().unwrap();
        let mut samples_uniques_loc_ids = vec![unique_native_loc_ids; n_samples];

        srl_deltas_map.iter().enumerate().for_each(|(sample_id, sample_deltas)| {

            let mut previous_loc_ids: HashSet<usize> = HashSet::new();
            let mut new_loc_ids: HashSet<usize> = HashSet::new();
            sample_deltas.iter().for_each(|(row_id, new_loc_id)| {
                previous_loc_ids.insert(location_ids[*row_id]);
                new_loc_ids.insert(*new_loc_id);
            });

            previous_loc_ids.iter().for_each(|prev_loc_id| {samples_uniques_loc_ids[sample_id].remove(prev_loc_id);});
            new_loc_ids.iter().for_each(|new_loc_id| {samples_uniques_loc_ids[sample_id].insert(*new_loc_id);});

            
        });

        let locations_vec_len = location_ids.len();
        let scores: Vec<HardSoftScore> = samples_uniques_loc_ids.iter().map(|sample_uniques| {
            HardSoftScore::new((locations_vec_len - sample_uniques.len()) as f64, 0.0)
        }).collect();

        //println!("{:?}", scores);

        return scores;

    }*/

    fn no_duplicating_stops_constraint(
        planning_entity_dfs: &HashMap<String, DataFrame>, 
        problem_fact_dfs: &HashMap<String, DataFrame>,
        delta_dfs: &HashMap<String, DataFrame>,
        utility_objects: &mut HashMap<String, UtilityObjectVariants>,
    ) -> Vec<HardSoftScore> {

        /*
        Not incremental in "clean" sense (just using deltas to save resources on dataframes creation)
        Prescoring functions are redudant in this approach
        */

        let path_stops_df = planning_entity_dfs["path_stops"].clone();
        let path_stops_deltas_df = delta_dfs["path_stops"].clone();

        let locations_vec_len = path_stops_df["location_vec_id"].len();
        let location_ids: Vec<usize> = 
        path_stops_df["location_vec_id"]
        .i64().unwrap().to_vec()
        .iter().map(|loc_id| loc_id.unwrap() as usize)
        .collect();
        
        let mut scores: Vec<HardSoftScore> = path_stops_deltas_df
        .partition_by(["sample_id"], false).unwrap()
        .iter().enumerate().map(|(i, sample_df)| {
            let mut sample_location_ids: Vec<usize> = 
            sample_df["location_vec_id"]
            .i64().unwrap().to_vec()
            .iter().map(|loc_id| loc_id.unwrap() as usize)
            .collect();

            let mut sample_row_ids: Vec<usize> = 
            sample_df["row_id"]
            .u64().unwrap().to_vec()
            .iter().map(|row_id| row_id.unwrap() as usize)
            .collect();
            
            let mut changed_location_ids = location_ids.clone();
            sample_row_ids.iter().zip(sample_location_ids.iter()).into_iter().for_each(|(row_id, loc_id)| {
                changed_location_ids[*row_id] = *loc_id;
            });

            let sample_unique_location_ids: HashSet<usize> = changed_location_ids.iter().map(|loc_id| *loc_id).collect();

            HardSoftScore::new((locations_vec_len - sample_unique_location_ids.len()) as f64, 0.0)
        }).collect();

        return scores;

    }

    /*fn minimize_distance(
        planning_entity_dfs: &HashMap<String, DataFrame>, 
        problem_fact_dfs: &HashMap<String, DataFrame>,
        delta_dfs: &HashMap<String, DataFrame>,
        utility_objects: &mut HashMap<String, UtilityObjectVariants>,
    ) -> Vec<HardSoftScore> {

        /*
        Broken, cam't understand, what is wrong.
        */

        let path_stops_df = planning_entity_dfs["path_stops"].clone();
        let path_stops_deltas_df = delta_dfs["path_stops"].clone();
        let distance_matrix: &Vec<Vec<f64>>;
        match &utility_objects["distance_matrix"] {
            UtilityObjectVariants::DistanceMatrix(dm) => distance_matrix = &dm,
            _ => panic!("dragons")
        }
        let srl_deltas_map;
        match &utility_objects["srl_deltas_map"] {
            UtilityObjectVariants::DeltasMap(dm) => srl_deltas_map = dm,
            _ => panic!("dragons")
        }

        let planning_stop_ids: Vec<usize> = path_stops_df["location_vec_id"].i64().unwrap().to_vec().iter().map(|x| x.unwrap() as usize).collect();
        let last_row_id = planning_stop_ids.len() - 1;

        let mut sample_distance = 0.0;
        let last_id = planning_stop_ids.len() - 1; 
        sample_distance += distance_matrix[0][planning_stop_ids[0]];
        sample_distance += distance_matrix[planning_stop_ids[last_id]][0];
        sample_distance += (1..planning_stop_ids.len()).into_iter().fold(0.0, |interim_distance, i| interim_distance + distance_matrix[planning_stop_ids[i-1]][planning_stop_ids[i]]);
        let n_samples = path_stops_deltas_df["sample_id"].n_unique().unwrap();
        let mut sample_distances: Vec<f64> = vec![sample_distance; n_samples];

        for i in 0..n_samples {
            if srl_deltas_map[i].contains_key(&0) {
                sample_distances[i] -= distance_matrix[0][planning_stop_ids[0]];
                sample_distances[i] += distance_matrix[0][*srl_deltas_map[i].get(&0).unwrap()];
            }

            if srl_deltas_map[i].contains_key(&last_row_id) {
                sample_distances[i] -= distance_matrix[planning_stop_ids[last_row_id]][0];
                sample_distances[i] += distance_matrix[*srl_deltas_map[i].get(&last_row_id).unwrap()][0];
            }

            for (row_id, new_loc_id) in &srl_deltas_map[i] {
                if *row_id == 0 {
                    continue;
                }
                
                let previous_loc_id;
                if srl_deltas_map[i].contains_key(&(*row_id-1)) {
                    previous_loc_id = *srl_deltas_map[i].get(&(*row_id-1)).unwrap();
                } else {
                    previous_loc_id = planning_stop_ids[*row_id-1];
                }

                sample_distances[i] -= distance_matrix[previous_loc_id][planning_stop_ids[*row_id]];
                sample_distances[i] += distance_matrix[previous_loc_id][*new_loc_id];

            }
        }

        let scores: Vec<HardSoftScore> = sample_distances.iter().map(|sample_distance| HardSoftScore::new(0.0, *sample_distance)).collect();

        return scores;
    }*/

    fn minimize_distance(
        planning_entity_dfs: &HashMap<String, DataFrame>, 
        problem_fact_dfs: &HashMap<String, DataFrame>,
        delta_dfs: &HashMap<String, DataFrame>,
        utility_objects: &mut HashMap<String, UtilityObjectVariants>,
    ) -> Vec<HardSoftScore> {

        /*
        Not incremental in "clean" sense (just using deltas to save resources on dataframes creation)
        Prescoring functions are redudant in this approach
        */

        let path_stops_df = planning_entity_dfs["path_stops"].clone();
        let path_stops_deltas_df = delta_dfs["path_stops"].clone();
        let distance_matrix: &Vec<Vec<f64>>;
        match &utility_objects["distance_matrix"] {
            UtilityObjectVariants::DistanceMatrix(dm) => distance_matrix = &dm,
            _ => panic!("dragons")
        }

        let planning_stop_ids: Vec<usize> = path_stops_df["location_vec_id"].i64().unwrap().to_vec().iter().map(|x| x.unwrap() as usize).collect();

        let mut scores: Vec<HardSoftScore> = path_stops_deltas_df
        .partition_by(["sample_id"], false).unwrap()
        .iter().enumerate().map(|(i, sample_df)| {
            let current_loc_ids: Vec<usize> = sample_df["location_vec_id"]
            .i64().unwrap().to_vec()
            .iter().map(|loc_id| loc_id.unwrap() as usize)
            .collect();

            let current_row_ids: Vec<usize> = sample_df["row_id"]
            .u64().unwrap().to_vec()
            .iter().map(|row_id| row_id.unwrap() as usize)
            .collect();

            let mut changed_stops = planning_stop_ids.clone();
            current_row_ids.iter()
            .zip(current_loc_ids.iter())
            .for_each(|(row_id, loc_id)| {
                changed_stops[*row_id] = *loc_id;
            });
            
            let mut sample_distance = 0.0;
            let last_id = changed_stops.len() - 1; 
            sample_distance += distance_matrix[0][changed_stops[0]];
            sample_distance += distance_matrix[changed_stops[last_id]][0];
            sample_distance += (1..changed_stops.len()).into_iter().fold(0.0, |interim_distance, i| interim_distance + distance_matrix[changed_stops[i-1]][changed_stops[i]]);

            HardSoftScore::new(0.0, sample_distance)
        }).collect();

        return scores;
    }
}