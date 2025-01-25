
use std::collections::HashMap;
use crate::score_calculation::score_requesters::VariablesManager;
use ndarray::Array1;

pub trait MoveTrait {

    fn get_necessary_info_for_move<'d>(
        &self, 
        variables_manager: &'d VariablesManager, 
    ) -> (&'d Vec<usize>, &'d String, usize);

    fn change_move(
        &mut self, 
        candidate: &mut Array1<f64>, 
        variables_manager: &VariablesManager, 
    ) -> Option<Vec<usize>>;

    fn swap_move(
        &mut self, 
        candidate: &mut Array1<f64>, 
        variables_manager: &VariablesManager, 
    ) -> Option<Vec<usize>>;

    fn swap_edges_move(
        &mut self, 
        candidate: &mut Array1<f64>, 
        variables_manager: &VariablesManager, 
    ) -> Option<Vec<usize>>;

    fn insertion_move(
        &mut self, 
        candidate: &mut Array1<f64>, 
        variables_manager: &VariablesManager, 
    ) -> Option<Vec<usize>>;

    fn scramble_move(
        &mut self, 
        candidate: &mut Array1<f64>, 
        variables_manager: &VariablesManager, 
    ) -> Option<Vec<usize>>;

}
