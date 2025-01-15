
use std::collections::HashMap;
use crate::core::score_calculation::score_requesters::variables_manager::VariablesManager;
use ndarray::Array1;

pub trait MutationsTrait {

    fn get_needful_info_for_move<'d>(
        variables_manager: &'d VariablesManager, 
        group_mutation_rates_dict: &HashMap<String, f64>, 
        variables_count: usize
    ) -> (&'d Vec<usize>, usize);

    fn change_move(
        candidate: &mut Array1<f64>, 
        variables_manager: &VariablesManager, 
        group_mutation_rates_dict: &HashMap<String, f64>, 
        variables_count: usize
    ) -> Option<Vec<usize>>;

    fn swap_move(
        candidate: &mut Array1<f64>, 
        variables_manager: &VariablesManager, 
        group_mutation_rates_dict: &HashMap<String, f64>, 
        variables_count: usize
    ) -> Option<Vec<usize>>;

    fn swap_edges_move(
        candidate: &mut Array1<f64>, 
        variables_manager: &VariablesManager, 
        group_mutation_rates_dict: &HashMap<String, f64>, 
        variables_count: usize
    ) -> Option<Vec<usize>>;

    fn insertion_move(
        candidate: &mut Array1<f64>, 
        variables_manager: &VariablesManager, 
        group_mutation_rates_dict: &HashMap<String, f64>, 
        variables_count: usize
    ) -> Option<Vec<usize>>;

    fn scramble_move(

        candidate: &mut Array1<f64>, 
        variables_manager: &VariablesManager, 
        group_mutation_rates_dict: 
        &HashMap<String, f64>, variables_count: usize
    ) -> Option<Vec<usize>>;

}
