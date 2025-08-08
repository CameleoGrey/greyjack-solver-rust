// src/agents/metaheuristic_bases/mover.rs

use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;
use std::collections::VecDeque;
use crate::score_calculation::score_requesters::VariablesManager;
use crate::utils;
use rand::{seq::SliceRandom, SeedableRng, Rng};
use rand::rngs::{SmallRng};
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
    rng: SmallRng,
    
    // Performance optimization caches
    valid_ids_cache: HashMap<String, Vec<usize>>,
    cache_dirty: HashMap<String, bool>,
    temp_indices: Vec<usize>,  // Reusable vector
    temp_deltas: Vec<f64>,     // Reusable vector
    temp_values: Vec<f64>,     // Additional reusable vector
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
        let rng = math_utils::create_rng();
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

        // Initialize caches
        let mut valid_ids_cache = HashMap::default();
        let mut cache_dirty = HashMap::default();
        
        for group_name in group_mutation_rates_map.keys() {
            valid_ids_cache.insert(group_name.clone(), Vec::new());
            cache_dirty.insert(group_name.clone(), true);
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
            valid_ids_cache,
            cache_dirty,
            temp_indices: Vec::with_capacity(64),
            temp_deltas: Vec::with_capacity(64),
            temp_values: Vec::with_capacity(64),
        }
    }

    // Optimized choice function using partial Fisher-Yates shuffle
    fn choice_without_replacement(&mut self, objects: &[usize], n: usize) -> Vec<usize> {
        if n > objects.len() {
            panic!("Cannot choose {} items from {} objects without replacement", n, objects.len());
        }
        
        if n == 0 {
            return Vec::new();
        }
        
        // For small n relative to objects.len(), use partial Fisher-Yates
        if n <= objects.len() / 2 {
            self.temp_indices.clear();
            self.temp_indices.extend_from_slice(objects);
            
            let mut chosen = Vec::with_capacity(n);
            for i in 0..n {
                let j = self.rng.gen_range(i..self.temp_indices.len());
                self.temp_indices.swap(i, j);
                chosen.push(self.temp_indices[i]);
            }
            chosen
        } else {
            // For large n, shuffle everything and take first n
            self.temp_indices.clear();
            self.temp_indices.extend_from_slice(objects);
            self.temp_indices.shuffle(&mut self.rng);
            self.temp_indices[..n].to_vec()
        }
    }

    // Update valid IDs cache for a group
    fn update_valid_ids_cache(&mut self, group_name: &str, group_size: usize) {
        if !*self.cache_dirty.get(group_name).unwrap_or(&true) {
            return;
        }
        
        let tabu_set = &self.tabu_ids_sets_map[group_name];
        let valid_ids = self.valid_ids_cache.get_mut(group_name).unwrap();
        valid_ids.clear();
        
        for i in 0..group_size {
            if !tabu_set.contains(&i) {
                valid_ids.push(i);
            }
        }
        
        self.cache_dirty.insert(group_name.to_string(), false);
    }

    // Fast non-tabu sampling using cached valid IDs
    fn select_non_tabu_ids(&mut self, group_name: &str, selection_size: usize, group_size: usize) -> Vec<usize> {
        self.update_valid_ids_cache(group_name, group_size);
        
        // Clone the valid IDs to avoid borrowing conflicts
        let valid_ids = self.valid_ids_cache[group_name].clone();
        if valid_ids.len() < selection_size {
            // Fall back to original method if not enough valid IDs
            return self.select_non_tabu_ids(&group_name.to_string(), selection_size, group_size);
        }
        
        let chosen_indices = self.choice_without_replacement(&valid_ids, selection_size);
        
        // Update tabu structures efficiently
        let tabu_set = self.tabu_ids_sets_map.get_mut(group_name).unwrap();
        let tabu_queue = self.tabu_ids_vecdeque_map.get_mut(group_name).unwrap();
        let max_size = self.tabu_entity_size_map[group_name];
        
        for &id in &chosen_indices {
            tabu_set.insert(id);
            tabu_queue.push_front(id);
            
            if tabu_queue.len() > max_size {
                if let Some(popped) = tabu_queue.pop_back() {
                    tabu_set.remove(&popped);
                }
            }
        }
        
        // Mark cache as dirty for next time
        self.cache_dirty.insert(group_name.to_string(), true);
        
        chosen_indices
    }

    pub fn do_move(&mut self, candidate: &Vec<f64>, variables_manager: &VariablesManager, incremental: bool) -> (Option<Vec<f64>>, Option<Vec<usize>>, Option<Vec<f64>>) {
        let random_value = self.rng.gen_range(0.0..=1.0);
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
        
        // Optimize mutation count calculation
        let crossover_count = if group_mutation_rate >= 1.0 {
            variables_manager.variables_count
        } else if group_mutation_rate <= 0.0 {
            0
        } else {
            // Use expected value for performance
            let expected = variables_manager.variables_count as f64 * group_mutation_rate;
            if expected < 5.0 {
                // For small expected values, count individual checks
                (0..variables_manager.variables_count)
                    .map(|_| if self.rng.gen::<f64>() < group_mutation_rate { 1 } else { 0 })
                    .sum()
            } else {
                // For larger expected values, use the expectation with some variance
                let variance = expected * (1.0 - group_mutation_rate);
                let adjustment = (self.rng.gen::<f64>() - 0.5) * variance.sqrt();
                (expected + adjustment).round().max(1.0) as usize
            }
        };
        
        (group_ids, group_name, crossover_count)
    }

    pub fn change_move(&mut self, candidate: &Vec<f64>, variables_manager: &VariablesManager, incremental: bool) -> (Option<Vec<f64>>, Option<Vec<usize>>, Option<Vec<f64>>) {
        let (group_ids, group_name, mut current_change_count) = self.get_necessary_info_for_move(variables_manager);
        if current_change_count < 1 {
            current_change_count = 1;
        }
        if group_ids.len() < current_change_count {
            return (None, None, None);
        }

        let changed_columns: Vec<usize>;
        if self.tabu_entity_rate == 0.0 {
            let indices: Vec<usize> = (0..group_ids.len()).collect();
            let chosen_indices = self.choice_without_replacement(&indices, current_change_count);
            changed_columns = chosen_indices.iter().map(|&i| group_ids[i]).collect();
        } else {
            let chosen_indices = self.select_non_tabu_ids(group_name, current_change_count, group_ids.len());
            changed_columns = chosen_indices.iter().map(|&i| group_ids[i]).collect();
        }

        if incremental {
            self.temp_deltas.clear();
            self.temp_deltas.reserve(changed_columns.len());
            for &i in &changed_columns {
                self.temp_deltas.push(variables_manager.get_column_random_value(i, &mut self.rng));
            }
            (None, Some(changed_columns), Some(self.temp_deltas.clone()))
        } else {
            let mut changed_candidate = candidate.clone();
            for &i in &changed_columns {
                changed_candidate[i] = variables_manager.get_column_random_value(i, &mut self.rng);
            }
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

        let changed_columns: Vec<usize>;
        if self.tabu_entity_rate == 0.0 {
            let indices: Vec<usize> = (0..group_ids.len()).collect();
            let chosen_indices = self.choice_without_replacement(&indices, current_change_count);
            changed_columns = chosen_indices.iter().map(|&i| group_ids[i]).collect();
        } else {
            let chosen_indices = self.select_non_tabu_ids(group_name, current_change_count, group_ids.len());
            changed_columns = chosen_indices.iter().map(|&i| group_ids[i]).collect();
        }

        if incremental {
            self.temp_deltas.clear();
            self.temp_deltas.reserve(changed_columns.len());
            for &i in &changed_columns {
                self.temp_deltas.push(candidate[i]);
            }
            // Rotate left by 1 for cyclic swap
            if !self.temp_deltas.is_empty() {
                self.temp_deltas.rotate_left(1);
            }
            (None, Some(changed_columns), Some(self.temp_deltas.clone()))
        } else {
            let mut changed_candidate = candidate.clone();
            if !changed_columns.is_empty() {
                let first_val = changed_candidate[changed_columns[0]];
                for i in 0..current_change_count - 1 {
                    changed_candidate[changed_columns[i]] = changed_candidate[changed_columns[i + 1]];
                }
                changed_candidate[*changed_columns.last().unwrap()] = first_val;
            }
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
            let indices: Vec<usize> = (0..(group_ids.len() - 1)).collect();
            columns_to_change = self.choice_without_replacement(&indices, current_change_count);
        } else {
            columns_to_change = self.select_non_tabu_ids(group_name, current_change_count, group_ids.len() - 1);
        }

        let mut edges: Vec<(usize, usize)> = Vec::with_capacity(columns_to_change.len());
        let mut changed_columns: HashSet<usize> = HashSet::default();
        for &i in &columns_to_change {
            let edge = (group_ids[i], group_ids[i + 1]);
            edges.push(edge);
            changed_columns.insert(edge.0);
            changed_columns.insert(edge.1);
        }
        let changed_columns_vec: Vec<usize> = changed_columns.into_iter().collect();

        if incremental {
            let mut deltas: HashMap<usize, f64> = HashMap::default();
            let mut rotated_edges = edges.clone();
            rotated_edges.rotate_left(1);

            for (original_edge, rotated_edge) in edges.iter().zip(rotated_edges.iter()) {
                deltas.insert(original_edge.0, candidate[rotated_edge.0]);
                deltas.insert(original_edge.1, candidate[rotated_edge.1]);
            }
            
            self.temp_deltas.clear();
            self.temp_deltas.reserve(changed_columns_vec.len());
            for &col in &changed_columns_vec {
                self.temp_deltas.push(deltas[&col]);
            }
            (None, Some(changed_columns_vec), Some(self.temp_deltas.clone()))
        } else {
            let mut changed_candidate = candidate.clone();
            if !edges.is_empty() {
                let first_edge_values = (changed_candidate[edges[0].0], changed_candidate[edges[0].1]);
                for i in 0..edges.len() - 1 {
                    changed_candidate[edges[i].0] = changed_candidate[edges[i + 1].0];
                    changed_candidate[edges[i].1] = changed_candidate[edges[i + 1].1];
                }
                let last_edge = edges.last().unwrap();
                changed_candidate[last_edge.0] = first_edge_values.0;
                changed_candidate[last_edge.1] = first_edge_values.1;
            }
            (Some(changed_candidate), Some(changed_columns_vec), None)
        }
    }

    pub fn scramble_move(&mut self, candidate: &Vec<f64>, variables_manager: &VariablesManager, incremental: bool) -> (Option<Vec<f64>>, Option<Vec<usize>>, Option<Vec<f64>>) {
        let current_change_count = self.rng.gen_range(3..=6);
        let (group_ids, group_name) = variables_manager.get_random_semantic_group_ids(&mut self.rng);

        if group_ids.len() < current_change_count {
            return (None, None, None);
        }

        let current_start_id: usize;
        if self.tabu_entity_rate == 0.0 {
            current_start_id = self.rng.gen_range(0..=(group_ids.len() - current_change_count));
        } else {
            let selected = self.select_non_tabu_ids(group_name, 1, group_ids.len() - current_change_count + 1);
            if selected.is_empty() {
                return (None, None, None);
            }
            current_start_id = selected[0];
        }

        let native_columns: Vec<usize> = (current_start_id..current_start_id + current_change_count)
            .map(|i| group_ids[i])
            .collect();

        if incremental {
            self.temp_indices.clear();
            self.temp_indices.extend_from_slice(&native_columns);
            self.temp_indices.shuffle(&mut self.rng);
            
            self.temp_deltas.clear();
            self.temp_deltas.reserve(self.temp_indices.len());
            for &i in &self.temp_indices {
                self.temp_deltas.push(candidate[i]);
            }
            (None, Some(native_columns), Some(self.temp_deltas.clone()))
        } else {
            let mut changed_candidate = candidate.clone();
            
            self.temp_values.clear();
            self.temp_values.reserve(native_columns.len());
            for &i in &native_columns {
                self.temp_values.push(candidate[i]);
            }
            self.temp_values.shuffle(&mut self.rng);
            
            for (i, &col_index) in native_columns.iter().enumerate() {
                changed_candidate[col_index] = self.temp_values[i];
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
            let indices: Vec<usize> = (0..group_ids.len()).collect();
            columns_to_change = self.choice_without_replacement(&indices, 2);
        } else {
            columns_to_change = self.select_non_tabu_ids(group_name, 2, group_ids.len());
            if columns_to_change.len() < 2 {
                return (None, None, None);
            }
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
            self.temp_deltas.clear();
            self.temp_deltas.reserve(old_ids.len());
            for &id in &old_ids {
                self.temp_deltas.push(candidate[id]);
            }
            self.temp_deltas.rotate_left(1);
            (None, Some(old_ids), Some(self.temp_deltas.clone()))
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
            let indices: Vec<usize> = (0..group_ids.len()).collect();
            columns_to_change = self.choice_without_replacement(&indices, 2);
        } else {
            columns_to_change = self.select_non_tabu_ids(group_name, 2, group_ids.len());
            if columns_to_change.len() < 2 {
                return (None, None, None);
            }
        }

        let mut start_id = columns_to_change[0];
        let mut end_id = columns_to_change[1];
        if start_id > end_id {
            std::mem::swap(&mut start_id, &mut end_id);
        }

        let old_ids: Vec<usize> = (start_id..=end_id).map(|i| group_ids[i]).collect();

        if incremental {
            self.temp_deltas.clear();
            self.temp_deltas.reserve(old_ids.len());
            for &id in &old_ids {
                self.temp_deltas.push(candidate[id]);
            }
            self.temp_deltas.reverse();
            (None, Some(old_ids), Some(self.temp_deltas.clone()))
        } else {
            let mut changed_candidate = candidate.clone();
            
            self.temp_values.clear();
            self.temp_values.reserve(end_id - start_id + 1);
            for i in start_id..=end_id {
                self.temp_values.push(changed_candidate[group_ids[i]]);
            }
            self.temp_values.reverse();
            
            for (i, &val) in self.temp_values.iter().enumerate() {
                changed_candidate[group_ids[start_id + i]] = val;
            }
            (Some(changed_candidate), Some(old_ids), None)
        }
    }
}