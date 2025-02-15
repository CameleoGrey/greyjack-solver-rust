
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
        candidate: &Array1<f64>, 
        variables_manager: &VariablesManager,
        incremental: bool,
    ) -> (Option<Array1<f64>>, Option<Vec<usize>>, Option<Vec<f64>>);

    fn swap_move(
        &mut self, 
        candidate: &Array1<f64>, 
        variables_manager: &VariablesManager, 
        incremental: bool,
    ) -> (Option<Array1<f64>>, Option<Vec<usize>>, Option<Vec<f64>>);

    fn swap_edges_move(
        &mut self, 
        candidate: &Array1<f64>, 
        variables_manager: &VariablesManager, 
        incremental: bool,
    ) -> (Option<Array1<f64>>, Option<Vec<usize>>, Option<Vec<f64>>);

    fn scramble_move(
        &mut self, 
        candidate: &Array1<f64>, 
        variables_manager: &VariablesManager, 
        incremental: bool,
    ) -> (Option<Array1<f64>>, Option<Vec<usize>>, Option<Vec<f64>>);

    fn insertion_move(
        &mut self, 
        candidate: &Array1<f64>, 
        variables_manager: &VariablesManager, 
        incremental: bool,
    ) -> (Option<Array1<f64>>, Option<Vec<usize>>, Option<Vec<f64>>);

    fn inverse_move(
        &mut self, 
        candidate: &Array1<f64>, 
        variables_manager: &VariablesManager,
        incremental: bool,
    ) -> (Option<Array1<f64>>, Option<Vec<usize>>, Option<Vec<f64>>);

}
