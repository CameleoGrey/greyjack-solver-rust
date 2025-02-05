

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

    fn build_deltas_map(
        planning_entity_dfs: &HashMap<String, DataFrame>, 
        problem_fact_dfs: &HashMap<String, DataFrame>,
        delta_dfs: &HashMap<String, DataFrame>,
        utility_objects: &mut HashMap<String, UtilityObjectVariants>,
    ) {

        let path_stops_df = planning_entity_dfs["path_stops"].clone();
        let path_stops_deltas_df = delta_dfs["path_stops"].clone();

        let locations_vec_len = path_stops_df["location_vec_id"].len();

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

        let mut deltas_map: Vec<HashMap<usize, usize>> = Vec::new();
        deltas_map = vec![HashMap::new(); locations_vec_len];
        sample_ids.iter()
        .zip(row_ids.iter())
        .zip(location_ids.iter())
        .for_each(|((sample_id, row_id), loc_id)| {
            deltas_map[*row_id].insert(*sample_id, *loc_id);
        });

        utility_objects.insert("deltas_map".to_string(), UtilityObjectVariants::DeltasMap(deltas_map));
    }

    /*fn no_duplicating_stops_constraint(
        planning_entity_dfs: &HashMap<String, DataFrame>, 
        problem_fact_dfs: &HashMap<String, DataFrame>,
        delta_dfs: &HashMap<String, DataFrame>,
        utility_objects: &mut HashMap<String, UtilityObjectVariants>,
    ) -> Vec<HardSoftScore> {

        let path_stops_df = planning_entity_dfs["path_stops"].clone();
        let path_stops_deltas_df = delta_dfs["path_stops"].clone();
        let deltas_map;
        match &utility_objects["deltas_map"] {
            UtilityObjectVariants::DeltasMap(dm) => deltas_map = dm,
            _ => panic!("dragons")
        }

        let mut sample_ids: Vec<usize> = 
        path_stops_deltas_df["sample_id"]
        .unique_stable().unwrap()
        .u64().unwrap().to_vec()
        .iter().map(|sample_id| sample_id.unwrap() as usize)
        .collect();

        let mut samples_uniques_loc_ids: Vec<HashSet<usize>> = Vec::new();
        samples_uniques_loc_ids = vec![HashSet::new(); sample_ids.len()];


        let location_ids: Vec<usize> = 
        path_stops_df["location_vec_id"]
        .i64().unwrap().to_vec()
        .iter().map(|loc_id| loc_id.unwrap() as usize)
        .collect();

        let stub_collection: () = 
        location_ids.iter()
        .zip(deltas_map.iter())
        .enumerate()
        .map(|(i, (native_loc_id, sample_deltas))| {
            let stub_collection: () = sample_ids.iter().map(|sample_id| {
                if sample_deltas.contains_key(sample_id) {
                    samples_uniques_loc_ids[*sample_id].insert(sample_deltas.get(sample_id).unwrap().clone());
                } else {
                    samples_uniques_loc_ids[*sample_id].insert(*native_loc_id);
                }
            }).collect();
        }).collect();

        let locations_vec_len = location_ids.len();
        let scores: Vec<HardSoftScore> = 
        samples_uniques_loc_ids.iter().map(|sample_uniques| {
            HardSoftScore::new((locations_vec_len - sample_uniques.len()) as f64, 0.0)
        }).collect();

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

        let n_samples = path_stops_deltas_df["sample_id"].n_unique().unwrap();
        let unique_location_ids: HashSet<usize> = location_ids.iter().map(|loc_id| *loc_id).collect();
        let mut scores: Vec<HardSoftScore> = vec![HardSoftScore::new((locations_vec_len - unique_location_ids.len()) as f64, 0.0); n_samples];
        
        path_stops_deltas_df
        .partition_by(["sample_id"], false).unwrap()
        .iter().enumerate().for_each(|(i, sample_df)| {
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
            scores[i].hard_score = (locations_vec_len - sample_unique_location_ids.len()) as f64;
        });

        return scores;

    }

    /*fn minimize_distance(
        planning_entity_dfs: &HashMap<String, DataFrame>, 
        problem_fact_dfs: &HashMap<String, DataFrame>,
        delta_dfs: &HashMap<String, DataFrame>,
        utility_objects: &mut HashMap<String, UtilityObjectVariants>,
    ) -> Vec<HardSoftScore> {

        let path_stops_df = planning_entity_dfs["path_stops"].clone();
        let path_stops_deltas_df = delta_dfs["path_stops"].clone();
        let distance_matrix: &Vec<Vec<f64>>;
        match &utility_objects["distance_matrix"] {
            UtilityObjectVariants::DistanceMatrix(dm) => distance_matrix = &dm,
            _ => panic!("dragons")
        }
        let deltas_map;
        match &utility_objects["deltas_map"] {
            UtilityObjectVariants::DeltasMap(dm) => deltas_map = dm,
            _ => panic!("dragons")
        }

        let planning_stop_ids: Vec<usize> = path_stops_df["location_vec_id"].i64().unwrap().to_vec().iter().map(|x| x.unwrap() as usize).collect();
        let last_row_id = planning_stop_ids.len() - 1;
        let n_samples = path_stops_deltas_df["sample_id"].n_unique().unwrap();
        let mut sample_distances: Vec<f64> = vec![0.0; n_samples];

        for i in 0..n_samples {
            if deltas_map[0].contains_key(&i) {
                sample_distances[i] += distance_matrix[0][*deltas_map[0].get(&i).unwrap()];
            } else {
                sample_distances[i] += distance_matrix[0][planning_stop_ids[0]];
            }

            if deltas_map[last_row_id].contains_key(&i) {
                sample_distances[i] += distance_matrix[*deltas_map[last_row_id].get(&i).unwrap()][0];
            } else {
                sample_distances[i] += distance_matrix[planning_stop_ids[last_row_id]][0];
            }
        }

        for i in 1..planning_stop_ids.len() {

            let mut excluded_sample_ids: HashSet<usize> = HashSet::new();

            for sample_id in deltas_map[i].keys() {
                excluded_sample_ids.insert(*sample_id);

                let current_loc_id = deltas_map[i].get(sample_id).unwrap();
                let previous_loc_id = if deltas_map[i-1].contains_key(sample_id) {*deltas_map[i-1].get(sample_id).unwrap()} else {planning_stop_ids[i-1]};

                sample_distances[*sample_id] += distance_matrix[previous_loc_id][*current_loc_id];
            }

            for sample_id in 0..n_samples {
                if excluded_sample_ids.contains(&sample_id) {
                    continue;
                }

                let current_loc_id = planning_stop_ids[i];
                let previous_loc_id = if deltas_map[i-1].contains_key(&sample_id) {*deltas_map[i-1].get(&sample_id).unwrap()} else {planning_stop_ids[i-1]};

                sample_distances[sample_id] += distance_matrix[previous_loc_id][current_loc_id];
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
        let candidate_distance = HardSoftScore::new(0.0, 0.0);
        let n_samples = path_stops_deltas_df["sample_id"].n_unique().unwrap();
        let mut scores = vec![candidate_distance; n_samples];

        //println!("{:?}", scores.len());

        path_stops_deltas_df
        .partition_by(["sample_id"], false).unwrap()
        .iter().enumerate().for_each(|(i, sample_df)| {
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

            scores[i].soft_score = sample_distance;
        });

        return scores;
    }
}