
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use crate::score_calculation::score_requesters::VariablesManager;
use ndarray::Array1;
use ndarray_rand::RandomExt;
use rand::{seq::SliceRandom, SeedableRng};
use rand::rngs::StdRng;
use rand_distr::{Distribution, Uniform};
use crate::utils::math_utils;

pub struct TabuMoves {

    pub tabu_entity_rate: f64,
    pub tabu_entity_size_map: HashMap<String, usize>,
    pub tabu_ids_sets_map: HashMap<String, HashSet<usize>>,
    pub tabu_ids_vecdeque_map: HashMap<String, VecDeque<usize>>,

}

impl TabuMoves {

    pub fn new(
        tabu_entity_rate: f64,
        tabu_entity_size_map: HashMap<String, usize>,
        tabu_ids_sets_map: HashMap<String, HashSet<usize>>,
        tabu_ids_vecdeque_map: HashMap<String, VecDeque<usize>>,
    ) -> Self {

        Self {
            tabu_entity_rate: tabu_entity_rate,
            tabu_entity_size_map: tabu_entity_size_map,
            tabu_ids_sets_map: tabu_ids_sets_map,
            tabu_ids_vecdeque_map: tabu_ids_vecdeque_map,
        }
    }

    pub fn select_non_tabu_ids(&mut self, group_name: &String, selection_size: usize, right_end: usize) -> Vec<usize> {

        let mut random_ids: Vec<usize> = Vec::new();
        while random_ids.len() != selection_size {
            let random_id = math_utils::get_random_id(0, right_end);

            if self.tabu_ids_sets_map[group_name].contains(&random_id) == false {
                self.tabu_ids_sets_map.get_mut(group_name).unwrap().insert(random_id);
                self.tabu_ids_vecdeque_map.get_mut(group_name).unwrap().push_front(random_id);
                random_ids.push(random_id);

                if self.tabu_ids_vecdeque_map[group_name].len() > self.tabu_entity_size_map[group_name] {
                    self.tabu_ids_sets_map.get_mut(group_name).unwrap().remove( 
                        &self.tabu_ids_vecdeque_map.get_mut(group_name).unwrap().pop_back().unwrap()
                    );
                }
            }

        }

        return random_ids;
    }

    #[inline]
    pub fn change_move_base(
        &mut self, 
        candidate: &Array1<f64>, 
        variables_manager: &VariablesManager,
        mut current_change_count: usize, 
        group_ids: &Vec<usize>,
        group_name: &String,
        incremental: bool,
    ) -> (Option<Array1<f64>>, Option<Vec<usize>>, Option<Vec<f64>>) {
        
        if current_change_count < 1 {
            current_change_count = 1;
        }
        if group_ids.len() < current_change_count {
            return (None, None, None);
        }

        let changed_columns: Vec<usize>;
        if self.tabu_entity_rate == 0.0 {
            changed_columns = math_utils::choice(group_ids, current_change_count, false);
        } else {
            changed_columns = self.select_non_tabu_ids(group_name, current_change_count, group_ids.len());
        }

        if incremental {
            let deltas: Vec<f64> = changed_columns.iter().map(|i| variables_manager.get_column_random_value(*i)).collect();
            return (None, Some(changed_columns), Some(deltas));
        } else {
            let mut changed_candidate = candidate.clone();
            changed_columns.iter().for_each(|i| changed_candidate[*i] = variables_manager.get_column_random_value(*i));
            return (Some(changed_candidate), Some(changed_columns), None);
        }
    }

    #[inline]
    pub fn swap_move_base(
        &mut self, candidate: 
        &Array1<f64>, 
        variables_manager: &VariablesManager, 
        mut current_change_count: usize, 
        group_ids: &Vec<usize>,
        group_name: &String,
        incremental: bool,
    ) -> (Option<Array1<f64>>, Option<Vec<usize>>, Option<Vec<f64>>) {

        if current_change_count < 2 {
            current_change_count = 2;
        }
        if group_ids.len() < current_change_count {
            return (None, None, None);
        }

        let changed_columns: Vec<usize>;
        if self.tabu_entity_rate == 0.0 {
            changed_columns = math_utils::choice(group_ids, current_change_count, false);
        } else {
            changed_columns = self.select_non_tabu_ids(group_name, current_change_count, group_ids.len());
        }

        if incremental {
            let mut deltas: Vec<f64> = Vec::new();
            for i in 1..current_change_count {
                deltas.push(candidate[changed_columns[i]]);
                deltas.push(candidate[changed_columns[i-1]]);
            }
            return (None, Some(changed_columns), Some(deltas));
        } else {
            let mut changed_candidate = candidate.clone();
            for i in 1..current_change_count {
                changed_candidate.swap(changed_columns[i-1], changed_columns[i]);
            }
            return (Some(changed_candidate), Some(changed_columns), None);
        }
    }

    #[inline]
    pub fn swap_edges_move_base(
        &mut self, 
        candidate: &Array1<f64>, 
        variables_manager: &VariablesManager, 
        mut current_change_count: usize, 
        group_ids: &Vec<usize>,
        group_name: &String,
        incremental: bool,
    ) -> (Option<Array1<f64>>, Option<Vec<usize>>, Option<Vec<f64>>) {

        if group_ids.len() == 0 {
            return (None, None, None);
        }
        if current_change_count < 2 {
            current_change_count = 2;
        }
        if current_change_count > group_ids.len()-1 {
            current_change_count = group_ids.len()-1;
        }

        let columns_to_change: Vec<usize>;
        if self.tabu_entity_rate == 0.0 {
            columns_to_change = math_utils::choice(&(0..(group_ids.len()-1)).collect(), current_change_count, false);
        } else {
            columns_to_change = self.select_non_tabu_ids(group_name, current_change_count, group_ids.len()-1);
        }

        let mut edges: Vec<(usize, usize)> = Vec::new();
        let mut changed_columns: Vec<usize> = Vec::new();
        for i in 0..current_change_count {
            let edge = (group_ids[columns_to_change[i]], group_ids[columns_to_change[i] + 1]);
            edges.push(edge);
            changed_columns.push(edge.0);
            changed_columns.push(edge.1);
        }
        edges.rotate_left(1);

        if incremental {
            let mut deltas: Vec<f64> = Vec::new();
            for i in 1..current_change_count {
                let left_edge = edges[i-1];
                let right_edge = edges[i];
                deltas.push(candidate[right_edge.0]);
                deltas.push(candidate[left_edge.0]);
                deltas.push(candidate[right_edge.1]);
                deltas.push(candidate[left_edge.1]);
            }
            return (None, Some(changed_columns), Some(deltas));
        } else {
            let mut changed_candidate = candidate.clone();
            for i in 1..current_change_count {
                let left_edge = edges[i-1];
                let right_edge = edges[i];
                changed_candidate.swap(left_edge.0, right_edge.0);
                changed_candidate.swap(left_edge.1, right_edge.1);
            }
            return (Some(changed_candidate), Some(changed_columns), None);
        }
    }

    #[inline]
    pub fn insertion_move_base(
        &mut self, 
        candidate: &Array1<f64>, 
        variables_manager: &VariablesManager, 
        mut current_change_count: usize, 
        group_ids: &Vec<usize>,
        group_name: &String,
        incremental: bool,
    ) -> (Option<Array1<f64>>, Option<Vec<usize>>, Option<Vec<f64>>) {

        if group_ids.len() <= 1 {
            return (None, None, None);
        }

        let columns_to_change: Vec<usize>;
        if self.tabu_entity_rate == 0.0 {
            columns_to_change = math_utils::choice(group_ids, current_change_count, false);
        } else {
            columns_to_change = self.select_non_tabu_ids(group_name, current_change_count, group_ids.len());
        }

        let get_out_id = columns_to_change[0];
        let put_in_id = columns_to_change[1];
        let old_ids: Vec<usize>;
        let mut shifted_ids: Vec<usize>;
        let left_rotate;
        if get_out_id < put_in_id {
            old_ids = (get_out_id..=put_in_id).into_iter().map(|i| group_ids[i]).collect();
            shifted_ids = old_ids.clone();
            shifted_ids.rotate_left(1);
            left_rotate = true;

        } else if get_out_id > put_in_id {
            old_ids = (put_in_id..=get_out_id).into_iter().map(|i| group_ids[i]).collect();
            shifted_ids = old_ids.clone();
            shifted_ids.rotate_right(1);
            left_rotate = false;

        } else {
            return (None, None, None);
        }

        let changed_columns = old_ids.clone();

        if incremental {
            let mut deltas: Vec<f64> = Vec::new();
            for old_id in old_ids {
                deltas.push(candidate[old_id]);
            }
            if left_rotate {
                deltas.rotate_left(1);
            } else {
                deltas.rotate_right(1);
            }
            return (None, Some(changed_columns), Some(deltas));
        } else {
            let mut changed_candidate = candidate.clone();
            old_ids.iter().zip(shifted_ids.iter()).for_each(|(oi, si)| changed_candidate.swap(*oi, *si));
            return (Some(changed_candidate), Some(changed_columns), None);
        }
    }

    #[inline]
    pub fn scramble_move_base(
        &mut self, 
        candidate: &Array1<f64>, 
        variables_manager: &VariablesManager, 
        mut current_change_count: usize, 
        group_ids: &Vec<usize>,
        group_name: &String,
        incremental: bool,
    ) -> (Option<Array1<f64>>, Option<Vec<usize>>, Option<Vec<f64>>) {

        if group_ids.len() < current_change_count - 1 {
            return (None, None, None);
        }

        let current_start_id: usize;
        if self.tabu_entity_rate == 0.0 {
            current_start_id = math_utils::get_random_id(0, group_ids.len() - current_change_count);
        } else {
            current_start_id = self.select_non_tabu_ids(group_name, 1, group_ids.len() - current_change_count)[0];
        }

        let native_columns: Vec<usize> = (0..current_change_count).into_iter().map(|i| group_ids[current_start_id + i]).collect();
        let mut scrambled_columns = native_columns.clone();
        scrambled_columns.shuffle(&mut StdRng::from_entropy());


        if incremental {
            let mut deltas: Vec<f64> = Vec::new();
            for scrambled_column in &scrambled_columns {
                deltas.push(candidate[*scrambled_column]);
            }
            let changed_columns = native_columns.clone();
            return (None, Some(changed_columns), Some(deltas));
        } else {
            let changed_columns = native_columns.clone();
            let mut changed_candidate = candidate.clone();
            native_columns.iter().zip(scrambled_columns.iter()).for_each(|(oi, si)| changed_candidate.swap(*oi, *si));
            return (Some(changed_candidate), Some(changed_columns), None);
        }
    }
}