// src/agents/metaheuristic_bases/mover.rs

use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use crate::score_calculation::score_requesters::VariablesManager;
use crate::utils;
use rand::{seq::SliceRandom, SeedableRng};
use rand::rngs::StdRng;
use rand_distr::{Distribution, Uniform};
use crate::utils::math_utils;

pub struct Mover {
    pub tabu_entity_rate: f64,
    pub tabu_entity_size_map: HashMap<String, usize>,
    pub tabu_ids_sets_map: HashMap<String, HashSet<usize>>,
    pub tabu_ids_vecdeque_map: HashMap<String, VecDeque<usize>>,
    pub group_mutation_rates_map: HashMap<String, f64>,
    pub moves_count: u64,
    pub move_probas_tresholds: Vec<f64>,
    // Add an RNG to the Mover struct
    rng: StdRng,
}

impl Mover {
    pub fn new(
        tabu_entity_rate: f64,
        tabu_entity_size_map: HashMap<String, usize>,
        tabu_ids_sets_map: HashMap<String, HashSet<usize>>,
        tabu_ids_vecdeque_map: HashMap<String, VecDeque<usize>>,
        group_mutation_rates_map: HashMap<String, f64>,
        move_probas: Option<Vec<f64>>,
    ) -> Self {
        let moves_count = 6;
        let mut rng = math_utils::create_rng();
        let move_probas_vec: Vec<f64>;

        match move_probas {
            None => {
                let mut increments: Vec<f64> = vec![math_utils::round(1.0 / (moves_count as f64), 3); moves_count];
                increments[0] += 1.0 - increments.iter().sum::<f64>();
                let mut proba_tresholds = vec![0.0; moves_count];
                let mut accumulator: f64 = 0.0;
                increments.iter().enumerate().for_each(|(i, proba)| {
                    accumulator += proba;
                    proba_tresholds[i] = accumulator;
                });
                move_probas_vec = proba_tresholds;
            }
            Some(probas) => {
                assert_eq!(probas.len(), moves_count, "Optional move probas vector length is not equal to available moves count");
                assert_eq!(utils::math_utils::round(probas.iter().sum(), 1), 1.0, "Optional move probas sum must be equal to 1.0");

                let mut proba_tresholds = vec![0.0; moves_count];
                let mut accumulator: f64 = 0.0;
                probas.iter().enumerate().for_each(|(i, proba)| {
                    accumulator += proba;
                    proba_tresholds[i] = accumulator;
                });
                move_probas_vec = proba_tresholds;
            }
        }

        Self {
            tabu_entity_rate,
            tabu_entity_size_map,
            tabu_ids_sets_map,
            tabu_ids_vecdeque_map,
            group_mutation_rates_map,
            moves_count: moves_count as u64,
            move_probas_tresholds: move_probas_vec,
            rng,
        }
    }

    pub fn select_non_tabu_ids(&mut self, group_name: &String, selection_size: usize, right_end: usize) -> Vec<usize> {
        let mut random_ids: Vec<usize> = Vec::new();
        while random_ids.len() != selection_size {
            let random_id = math_utils::get_random_id(0, right_end, &mut self.rng);

            if !self.tabu_ids_sets_map[group_name].contains(&random_id) {
                self.tabu_ids_sets_map.get_mut(group_name).unwrap().insert(random_id);
                self.tabu_ids_vecdeque_map.get_mut(group_name).unwrap().push_front(random_id);
                random_ids.push(random_id);

                if self.tabu_ids_vecdeque_map[group_name].len() > self.tabu_entity_size_map[group_name] {
                    if let Some(popped) = self.tabu_ids_vecdeque_map.get_mut(group_name).unwrap().pop_back() {
                        self.tabu_ids_sets_map.get_mut(group_name).unwrap().remove(&popped);
                    }
                }
            }
        }
        random_ids
    }

    pub fn do_move(&mut self, candidate: &Vec<f64>, variables_manager: &VariablesManager, incremental: bool) -> (Option<Vec<f64>>, Option<Vec<usize>>, Option<Vec<f64>>) {
        let random_value = Uniform::new_inclusive(0.0, 1.0).sample(&mut self.rng);
        if random_value <= self.move_probas_tresholds[0] {
            self.change_move(candidate, variables_manager, incremental)
        } else if random_value <= self.move_probas_tresholds[1] {
            self.swap_move(candidate, variables_manager, incremental)
        } else if random_value <= self.move_probas_tresholds[2] {
            self.swap_edges_move(candidate, variables_manager, incremental)
        } else if random_value <= self.move_probas_tresholds[3] {
            self.scramble_move(candidate, variables_manager, incremental)
        } else if random_value <= self.move_probas_tresholds[4] {
            self.insertion_move(candidate, variables_manager, incremental)
        } else if random_value <= self.move_probas_tresholds[5] {
            self.inverse_move(candidate, variables_manager, incremental)
        } else {
            panic!("Something wrong with probabilities");
        }
    }

    fn get_necessary_info_for_move<'d>(&mut self, variables_manager: &'d VariablesManager) -> (&'d Vec<usize>, &'d String, usize) {
        let (group_ids, group_name) = variables_manager.get_random_semantic_group_ids(&mut self.rng);
        let group_mutation_rate = self.group_mutation_rates_map[group_name];
        let random_values: Vec<f64> = (0..variables_manager.variables_count).map(|_| Uniform::new_inclusive(0.0, 1.0).sample(&mut self.rng)).collect();
        let crossover_mask: Vec<bool> = random_values.iter().map(|x| x < &group_mutation_rate).collect();
        let current_change_count = crossover_mask.iter().filter(|x| **x).count();
        (group_ids, group_name, current_change_count)
    }

    pub fn change_move(&mut self, candidate: &Vec<f64>, variables_manager: &VariablesManager, incremental: bool) -> (Option<Vec<f64>>, Option<Vec<usize>>, Option<Vec<f64>>) {
        let (group_ids, group_name, mut current_change_count) = self.get_necessary_info_for_move(variables_manager);
        if current_change_count < 1 {
            current_change_count = 1;
        }
        if group_ids.len() < current_change_count {
            return (None, None, None);
        }

        let mut changed_columns: Vec<usize>;
        if self.tabu_entity_rate == 0.0 {
            changed_columns = math_utils::choice(&(0..group_ids.len()).collect::<Vec<usize>>(), current_change_count, false, &mut self.rng);
        } else {
            changed_columns = self.select_non_tabu_ids(group_name, current_change_count, group_ids.len());
        }
        changed_columns = changed_columns.iter().map(|i| group_ids[*i]).collect();

        if incremental {
            let deltas: Vec<f64> = changed_columns.iter().map(|i| variables_manager.get_column_random_value(*i, &mut self.rng)).collect();
            (None, Some(changed_columns), Some(deltas))
        } else {
            let mut changed_candidate = candidate.clone();
            changed_columns.iter().for_each(|i| changed_candidate[*i] = variables_manager.get_column_random_value(*i, &mut self.rng));
            (Some(changed_candidate), Some(changed_columns), None)
        }
    }

    pub fn swap_move(&mut self, candidate: &Vec<f64>, variables_manager: &VariablesManager, incremental: bool) -> (Option<Vec<f64>>, Option<Vec<usize>>, Option<Vec<f64>>) {
        let (group_ids, group_name, mut current_change_count) = self.get_necessary_info_for_move(variables_manager);

        if current_change_count < 2 {
            current_change_count = 2;
        }
        if group_ids.len() < current_change_count {
            return (None, None, None);
        }

        let mut changed_columns: Vec<usize>;
        if self.tabu_entity_rate == 0.0 {
            changed_columns = math_utils::choice(&(0..group_ids.len()).collect::<Vec<usize>>(), current_change_count, false, &mut self.rng);
        } else {
            changed_columns = self.select_non_tabu_ids(group_name, current_change_count, group_ids.len());
        }
        changed_columns = changed_columns.iter().map(|i| group_ids[*i]).collect();

        if incremental {
            let mut deltas: Vec<f64> = changed_columns.iter().map(|&i| candidate[i]).collect();
            deltas.rotate_left(1);
            (None, Some(changed_columns), Some(deltas))
        } else {
            let mut changed_candidate = candidate.clone();
            let first_val = changed_candidate[changed_columns[0]];
            for i in 0..current_change_count - 1 {
                changed_candidate[changed_columns[i]] = changed_candidate[changed_columns[i + 1]];
            }
            changed_candidate[*changed_columns.last().unwrap()] = first_val;
            (Some(changed_candidate), Some(changed_columns), None)
        }
    }

    pub fn swap_edges_move(&mut self, candidate: &Vec<f64>, variables_manager: &VariablesManager, incremental: bool) -> (Option<Vec<f64>>, Option<Vec<usize>>, Option<Vec<f64>>) {
        let (group_ids, group_name, mut current_change_count) = self.get_necessary_info_for_move(variables_manager);

        if group_ids.is_empty() {
            return (None, None, None);
        }
        if current_change_count < 2 {
            current_change_count = 2;
        }
        if current_change_count > group_ids.len() - 1 {
            current_change_count = group_ids.len() - 1;
        }

        let columns_to_change: Vec<usize>;
        if self.tabu_entity_rate == 0.0 {
            columns_to_change = math_utils::choice(&(0..(group_ids.len() - 1)).collect(), current_change_count, false, &mut self.rng);
        } else {
            columns_to_change = self.select_non_tabu_ids(group_name, current_change_count, group_ids.len() - 1);
        }

        let mut edges: Vec<(usize, usize)> = Vec::new();
        let mut changed_columns: HashSet<usize> = HashSet::new();
        for &i in &columns_to_change {
            let edge = (group_ids[i], group_ids[i + 1]);
            edges.push(edge);
            changed_columns.insert(edge.0);
            changed_columns.insert(edge.1);
        }
        let changed_columns_vec: Vec<usize> = changed_columns.into_iter().collect();

        if incremental {
            let mut deltas: HashMap<usize, f64> = HashMap::new();
            let mut rotated_edges = edges.clone();
            rotated_edges.rotate_left(1);

            for (original_edge, rotated_edge) in edges.iter().zip(rotated_edges.iter()) {
                deltas.insert(original_edge.0, candidate[rotated_edge.0]);
                deltas.insert(original_edge.1, candidate[rotated_edge.1]);
            }
            let final_deltas = changed_columns_vec.iter().map(|&col| deltas[&col]).collect();
            (None, Some(changed_columns_vec), Some(final_deltas))
        } else {
            let mut changed_candidate = candidate.clone();
            let first_edge_values = (changed_candidate[edges[0].0], changed_candidate[edges[0].1]);
            for i in 0..edges.len() - 1 {
                changed_candidate[edges[i].0] = changed_candidate[edges[i + 1].0];
                changed_candidate[edges[i].1] = changed_candidate[edges[i + 1].1];
            }
            let last_edge = edges.last().unwrap();
            changed_candidate[last_edge.0] = first_edge_values.0;
            changed_candidate[last_edge.1] = first_edge_values.1;
            (Some(changed_candidate), Some(changed_columns_vec), None)
        }
    }

    pub fn scramble_move(&mut self, candidate: &Vec<f64>, variables_manager: &VariablesManager, incremental: bool) -> (Option<Vec<f64>>, Option<Vec<usize>>, Option<Vec<f64>>) {
        let current_change_count = Uniform::new_inclusive(3, 6).sample(&mut self.rng);
        let (group_ids, group_name) = variables_manager.get_random_semantic_group_ids(&mut self.rng);

        if group_ids.len() < current_change_count {
            return (None, None, None);
        }

        let current_start_id: usize;
        if self.tabu_entity_rate == 0.0 {
            current_start_id = math_utils::get_random_id(0, group_ids.len() - current_change_count, &mut self.rng);
        } else {
            current_start_id = self.select_non_tabu_ids(group_name, 1, group_ids.len() - current_change_count)[0];
        }

        let native_columns: Vec<usize> = (current_start_id..current_start_id + current_change_count).map(|i| group_ids[i]).collect();
        let mut scrambled_indices = native_columns.clone();
        scrambled_indices.shuffle(&mut self.rng);

        if incremental {
            let deltas: Vec<f64> = scrambled_indices.iter().map(|&i| candidate[i]).collect();
            (None, Some(native_columns), Some(deltas))
        } else {
            let mut changed_candidate = candidate.clone();
            let original_values: Vec<f64> = native_columns.iter().map(|&i| candidate[i]).collect();
            let mut scrambled_values = original_values.clone();
            scrambled_values.shuffle(&mut self.rng);
            for (i, &col_index) in native_columns.iter().enumerate() {
                changed_candidate[col_index] = scrambled_values[i];
            }
            (Some(changed_candidate), Some(native_columns), None)
        }
    }

    pub fn insertion_move(&mut self, candidate: &Vec<f64>, variables_manager: &VariablesManager, incremental: bool) -> (Option<Vec<f64>>, Option<Vec<usize>>, Option<Vec<f64>>) {
        let (group_ids, group_name) = variables_manager.get_random_semantic_group_ids(&mut self.rng);
        if group_ids.len() <= 1 {
            return (None, None, None);
        }

        let columns_to_change: Vec<usize>;
        if self.tabu_entity_rate == 0.0 {
            columns_to_change = math_utils::choice(&(0..group_ids.len()).collect::<Vec<usize>>(), 2, false, &mut self.rng);
        } else {
            columns_to_change = self.select_non_tabu_ids(group_name, 2, group_ids.len());
        }

        let mut get_out_id = columns_to_change[0];
        let mut put_in_id = columns_to_change[1];
        if get_out_id == put_in_id {
            return (None, None, None);
        }
        if get_out_id > put_in_id {
            std::mem::swap(&mut get_out_id, &mut put_in_id);
        }
        
        let old_ids: Vec<usize> = (get_out_id..=put_in_id).map(|i| group_ids[i]).collect();
        
        if incremental {
            let mut deltas: Vec<f64> = old_ids.iter().map(|&id| candidate[id]).collect();
            deltas.rotate_left(1);
            (None, Some(old_ids), Some(deltas))
        } else {
            let mut changed_candidate = candidate.clone();
            let value_to_move = changed_candidate[group_ids[get_out_id]];
            for i in get_out_id..put_in_id {
                changed_candidate[group_ids[i]] = changed_candidate[group_ids[i + 1]];
            }
            changed_candidate[group_ids[put_in_id]] = value_to_move;
            (Some(changed_candidate), Some(old_ids), None)
        }
    }

    pub fn inverse_move(&mut self, candidate: &Vec<f64>, variables_manager: &VariablesManager, incremental: bool) -> (Option<Vec<f64>>, Option<Vec<usize>>, Option<Vec<f64>>) {
        let (group_ids, group_name) = variables_manager.get_random_semantic_group_ids(&mut self.rng);
        if group_ids.len() <= 1 {
            return (None, None, None);
        }

        let columns_to_change: Vec<usize>;
        if self.tabu_entity_rate == 0.0 {
            columns_to_change = math_utils::choice(&(0..group_ids.len()).collect::<Vec<usize>>(), 2, false, &mut self.rng);
        } else {
            columns_to_change = self.select_non_tabu_ids(group_name, 2, group_ids.len());
        }

        let mut start_id = columns_to_change[0];
        let mut end_id = columns_to_change[1];
        if start_id > end_id {
            std::mem::swap(&mut start_id, &mut end_id);
        }

        let old_ids: Vec<usize> = (start_id..=end_id).map(|i| group_ids[i]).collect();

        if incremental {
            let mut deltas: Vec<f64> = old_ids.iter().map(|&id| candidate[id]).collect();
            deltas.reverse();
            (None, Some(old_ids), Some(deltas))
        } else {
            let mut changed_candidate = candidate.clone();
            let mut sub_slice: Vec<f64> = (start_id..=end_id).map(|i| changed_candidate[group_ids[i]]).collect();
            sub_slice.reverse();
            for (i, val) in sub_slice.iter().enumerate() {
                changed_candidate[group_ids[start_id + i]] = *val;
            }
            (Some(changed_candidate), Some(old_ids), None)
        }
    }
}
