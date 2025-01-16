
use std::collections::HashMap;
use crate::core::score_calculation::score_requesters::variables_manager::VariablesManager;
use ndarray::Array1;
use ndarray_rand::RandomExt;
use rand::{seq::SliceRandom, SeedableRng};
use rand::rngs::StdRng;
use rand_distr::{Distribution, Uniform};
use crate::core::utils::math_utils;

pub trait MutationsBaseTrait {
    fn change_move_base(candidate: &mut Array1<f64>, variables_manager: &VariablesManager, mut current_change_count: usize, group_ids: &Vec<usize>) -> Option<Vec<usize>> {
        
        if current_change_count < 1 {
            current_change_count = 1;
        }
        if group_ids.len() < current_change_count {
            return None;
        }

        let changed_columns: Vec<usize> = math_utils::choice(group_ids, current_change_count, false);
        changed_columns.iter().for_each(|i| candidate[*i] = variables_manager.get_column_random_value(*i));

        return Some(changed_columns);
    }

    fn swap_move_base(candidate: &mut Array1<f64>, variables_manager: &VariablesManager, mut current_change_count: usize, group_ids: &Vec<usize>) -> Option<Vec<usize>> {

        if current_change_count < 2 {
            current_change_count = 2;
        }
        if group_ids.len() < current_change_count {
            return None;
        }

        let changed_columns: Vec<usize> = math_utils::choice(group_ids, current_change_count, false);
        for i in 1..current_change_count {
            candidate.swap(changed_columns[i-1], changed_columns[i]);
        }

        return Some(changed_columns);
    }

    fn swap_edges_move_base(candidate: &mut Array1<f64>, variables_manager: &VariablesManager, mut current_change_count: usize, group_ids: &Vec<usize>) -> Option<Vec<usize>> {

        if current_change_count < 2 {
            current_change_count = 2;
        }
        if current_change_count > group_ids.len()-1 {
            current_change_count = group_ids.len()-1;
        }

        let columns_to_change: Vec<usize> = math_utils::choice(&(0..(group_ids.len()-1)).collect(), current_change_count, false);

        let mut edges: Vec<(usize, usize)> = Vec::new();
        let mut changed_columns: Vec<usize> = Vec::new();
        for i in 0..current_change_count {
            let edge = (group_ids[columns_to_change[i]], group_ids[columns_to_change[i] + 1]);
            edges.push(edge);
            changed_columns.push(edge.0);
            changed_columns.push(edge.1);
        }
        edges.rotate_left(1);

        for i in 1..current_change_count {
            let left_edge = edges[i-1];
            let right_edge = edges[i];
            candidate.swap(left_edge.0, right_edge.0);
            candidate.swap(left_edge.1, right_edge.1);
        }

        return Some(changed_columns);
    }

    fn insertion_move_base(candidate: &mut Array1<f64>, variables_manager: &VariablesManager, mut current_change_count: usize, group_ids: &Vec<usize>) -> Option<Vec<usize>> {

        if group_ids.len() <= 1 {
            return None;
        }

        let columns_to_change: Vec<usize> = math_utils::choice(group_ids, current_change_count, false);

        let get_out_id = columns_to_change[0];
        let put_in_id = columns_to_change[1];
        let old_ids: Vec<usize>;
        let mut shifted_ids: Vec<usize>;
        if get_out_id < put_in_id {
            old_ids = (get_out_id..=put_in_id).into_iter().map(|i| group_ids[i]).collect();
            shifted_ids = old_ids.clone();
            shifted_ids.rotate_left(1);

        } else if get_out_id > put_in_id {
            old_ids = (put_in_id..=get_out_id).into_iter().map(|i| group_ids[i]).collect();
            shifted_ids = old_ids.clone();
            shifted_ids.rotate_right(1);

        } else {
            return None;
        }

        old_ids.iter().zip(shifted_ids.iter()).for_each(|(oi, si)| candidate.swap(*oi, *si));
        let changed_columns = old_ids;

        return Some(changed_columns);
    }

    fn scramble_move_base(candidate: &mut Array1<f64>, variables_manager: &VariablesManager, mut current_change_count: usize, group_ids: &Vec<usize>) -> Option<Vec<usize>> {

        if group_ids.len() < current_change_count - 1 {
            return None;
        }
        let current_start_id = math_utils::get_random_id(0, group_ids.len() - current_change_count);
        let native_columns: Vec<usize> = (0..current_change_count).into_iter().map(|i| group_ids[current_start_id + i]).collect();
        let mut scrambled_columns = native_columns.clone();
        scrambled_columns.shuffle(&mut StdRng::from_entropy());

        native_columns.iter().zip(scrambled_columns.iter()).for_each(|(oi, si)| candidate.swap(*oi, *si));
        let changed_columns = native_columns;

        return Some(changed_columns);
    }
}