

use std::collections::HashMap;
use std::collections::VecDeque;
use crate::score_calculation::score_requesters::VariablesManager;

use super::MetaheuristicBaseTrait;
use crate::score_calculation::scores::ScoreTrait;
use crate::agents::base::Individual;
use ndarray::Array1;
use ndarray_rand::RandomExt;
use rand_distr::num_traits::ToPrimitive;
use std::ops::AddAssign;
use std::fmt::Debug;

use rand::SeedableRng;
use rand::rngs::StdRng;
use rand_distr::{Distribution, Uniform};

use super::moves::BaseMoves;
use super::moves::MoveTrait;
use super::metaheuristic_kinds_and_names::{MetaheuristicKind, MetaheuristicNames};
use crate::utils::math_utils;
use std::collections::HashSet;
use std::cmp::max;



pub struct TabuSearchBase {

    pub population_size: usize,
    pub neighbours_count: usize,

    pub tabu_size: usize,
    pub tabus_vec_deque: VecDeque<String>,
    pub tabus_set: HashSet<String>,

    pub tabu_entity_rate: f64,
    pub tabu_entity_size_map: HashMap<String, usize>,
    pub tabu_ids_sets_map: HashMap<String, HashSet<usize>>,
    pub tabu_ids_vec_map: HashMap<String, Vec<usize>>,

    pub metaheuristic_kind: MetaheuristicKind,
    pub metaheuristic_name: MetaheuristicNames,

    pub group_mutation_rates_map: HashMap<String, f64>,
    pub discrete_ids: Option<Vec<usize>>,
    pub moves_count: usize,
}

impl TabuSearchBase {

    pub fn new(
        population_size: usize,
        neighbours_count: usize,
        tabu_size: usize,
        tabu_entity_rate: f64,
        mutation_rate_multiplier: Option<f64>, 
        semantic_groups_map: HashMap<String, Vec<usize>>,
        discrete_ids: Option<Vec<usize>>,
    ) -> Self {

        let current_mutation_rate_multiplier;
        match mutation_rate_multiplier {
            Some(x) => current_mutation_rate_multiplier = mutation_rate_multiplier.unwrap(),
            None => current_mutation_rate_multiplier = 1.0,
        }
        let mut group_mutation_rates_map: HashMap<String, f64> = HashMap::new();
        for group_name in semantic_groups_map.keys() {
            let group_size = semantic_groups_map[group_name].len();
            let current_group_mutation_rate = current_mutation_rate_multiplier * (1.0 / (group_size as f64));
            group_mutation_rates_map.insert(group_name.clone(), current_group_mutation_rate);
        }

        Self {
            population_size: population_size,
            neighbours_count: neighbours_count,

            tabu_size: tabu_size,
            tabus_vec_deque: VecDeque::new(),
            tabus_set: HashSet::new(),

            tabu_entity_rate: tabu_entity_rate,
            tabu_entity_size_map: HashMap::new(),
            tabu_ids_sets_map: HashMap::new(),
            tabu_ids_vec_map: HashMap::new(),

            metaheuristic_kind: MetaheuristicKind::LocalSearch,
            metaheuristic_name: MetaheuristicNames::TabuSearch,

            group_mutation_rates_map: group_mutation_rates_map,
            discrete_ids: discrete_ids.clone(),
            moves_count: 5,
        }
    }

    fn mutate(&mut self, candidate: &mut Array1<f64>, variables_manager: &VariablesManager) -> Option<Vec<usize>>{

        let rand_method_id = Uniform::new(0, self.moves_count).sample(&mut StdRng::from_entropy());
        let changed_columns: Option<Vec<usize>>;
        match rand_method_id {
            0 => changed_columns = Self::change_move(candidate, variables_manager, &self.group_mutation_rates_map, variables_manager.variables_count),
            1 => changed_columns = Self::swap_move(candidate, variables_manager, &self.group_mutation_rates_map, variables_manager.variables_count),
            2 => changed_columns = Self::swap_edges_move(candidate, variables_manager, &self.group_mutation_rates_map, variables_manager.variables_count),
            3 => changed_columns = Self::insertion_move(candidate, variables_manager, &self.group_mutation_rates_map, variables_manager.variables_count),
            4 => changed_columns = Self::scramble_move(candidate, variables_manager, &self.group_mutation_rates_map, variables_manager.variables_count),
            _ => panic!("Invalid rand_method_id, no move with such id"),
        }

        return changed_columns;
    }

}

impl<ScoreType> MetaheuristicBaseTrait<ScoreType> for TabuSearchBase
where ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send {

    fn sample_candidates(
            &mut self, 
            population: &mut Vec<Individual<ScoreType>>, 
            current_top_individual: &Individual<ScoreType>,
            variables_manager: &VariablesManager
        ) -> Vec<Array1<f64>> {

        if self.tabu_entity_size_map.len() == 0 {
            let semantic_groups_map = variables_manager.semantic_groups_map.clone();
            for (group_name, group_ids) in semantic_groups_map {
                self.tabu_ids_sets_map.insert(group_name.clone(), HashSet::new());
                self.tabu_entity_size_map.insert(group_name.clone(), max((self.tabu_entity_rate * (group_ids.len().to_f64().unwrap())).ceil() as usize, 1));
                self.tabu_ids_vec_map.insert(group_name.clone(), Vec::new());
            }
        }

        if population.len() > 1 {
            population.sort();
        }

        let current_best_candidate = population[0].variable_values.clone();
        let mut candidates: Vec<Array1<f64>> = Vec::new();
        while candidates.len() < self.neighbours_count {

            let mut candidate = current_best_candidate.clone();
            let changed_columns = self.mutate(&mut candidate, variables_manager);
            variables_manager.fix_variables(&mut candidate, changed_columns);

            if self.tabu_size > 0 {
                let candidate_string = candidate.to_string();
                if self.tabus_set.contains(&candidate_string) {
                    continue;
                } else {
                    self.tabus_vec_deque.push_front(candidate_string.clone());
                    self.tabus_set.insert(candidate_string);
                }

                if self.tabus_vec_deque.len() > self.tabu_size {
                    self.tabus_set.remove( &self.tabus_vec_deque.pop_back().unwrap() );
                }
            }

            candidates.push(candidate);
        }

        return candidates;
    }

    fn build_updated_population(
        &mut self, 
        current_population: &Vec<Individual<ScoreType>>, 
        candidates: &mut Vec<Individual<ScoreType>>
        ) -> (Vec<Individual<ScoreType>>, bool) {
        
        let mut new_population:Vec<Individual<ScoreType>>;
        // current_population is already sorted in sample_candidates()
        candidates.sort();
        let best_candidate = candidates[0].clone();
        if best_candidate.score <= current_population[0].score {
            new_population = vec![best_candidate; 1];

            if current_population.len() > 1 {
            new_population.append(&mut current_population[1..(current_population.len()-1)].to_vec());
            }
            
            if new_population.len() > self.population_size {
                new_population = new_population[..self.population_size].to_vec();
            }
        } else {
            new_population = current_population.clone();
        }

        return (new_population, true);
    }

    fn get_metaheuristic_kind(&self) -> MetaheuristicKind {
        self.metaheuristic_kind.clone()
    }

    fn get_metaheuristic_name(&self) -> MetaheuristicNames {
        self.metaheuristic_name.clone()
    }
}

impl MoveTrait for TabuSearchBase {

    fn get_needful_info_for_move<'d>(
            variables_manager: &'d VariablesManager, 
            group_mutation_rates_map: &HashMap<String, f64>, 
            variables_count: usize
        ) -> (&'d Vec<usize>, usize) {
        
            let (group_ids, group_name) = variables_manager.get_random_semantic_group_ids();
            let group_mutation_rate = group_mutation_rates_map[group_name];
            let random_values = Array1::random(variables_count, Uniform::new_inclusive(0.0, 1.0));
            let crossover_mask: Array1<bool> = random_values.iter().map(|x| x < &group_mutation_rate).collect();
            let current_change_count = crossover_mask.iter().filter(|x| **x == true).count();

            return (group_ids, current_change_count);
    }

    fn change_move(
            candidate: &mut Array1<f64>, 
            variables_manager: &VariablesManager, 
            group_mutation_rates_map: &HashMap<String, f64>, 
            variables_count: usize
        ) -> Option<Vec<usize>> {

            let (group_ids, current_change_count) = Self::get_needful_info_for_move(variables_manager, group_mutation_rates_map, variables_count);

            BaseMoves::change_move_base(candidate, variables_manager, current_change_count, &group_ids)   
    }

    fn swap_move(
            candidate: &mut Array1<f64>, 
            variables_manager: &VariablesManager, 
            group_mutation_rates_map: &HashMap<String, f64>, 
            variables_count: usize
        ) -> Option<Vec<usize>> {

            let (group_ids, current_change_count) = Self::get_needful_info_for_move(variables_manager, group_mutation_rates_map, variables_count);
        
            BaseMoves::swap_move_base(candidate, variables_manager, current_change_count, &group_ids)
    }
    
    fn swap_edges_move(
            candidate: &mut Array1<f64>, 
            variables_manager: &VariablesManager, 
            group_mutation_rates_map: &HashMap<String, f64>, 
            variables_count: usize
        ) -> Option<Vec<usize>> {

            let (group_ids, current_change_count) = Self::get_needful_info_for_move(variables_manager, group_mutation_rates_map, variables_count);
            
            BaseMoves::swap_edges_move_base(candidate, variables_manager, current_change_count, &group_ids)
    }

    fn insertion_move(
            candidate: &mut Array1<f64>, 
            variables_manager: &VariablesManager, 
            group_mutation_rates_map: &HashMap<String, f64>, 
            variables_count: usize
        ) -> Option<Vec<usize>> {

            let (group_ids, group_name) = variables_manager.get_random_semantic_group_ids();
            let current_change_count = 2;

            BaseMoves::insertion_move_base(candidate, variables_manager, current_change_count, group_ids)
        
    }

    fn scramble_move(
            candidate: &mut Array1<f64>, 
            variables_manager: &VariablesManager, 
            group_mutation_rates_map: 
            &HashMap<String, f64>, variables_count: usize
        ) -> Option<Vec<usize>> {
        
            let mut current_change_count = Uniform::new_inclusive(3, 6).sample(&mut StdRng::from_entropy());
            let (group_ids, group_name) = variables_manager.get_random_semantic_group_ids();

            BaseMoves::scramble_move_base(candidate, variables_manager, current_change_count, group_ids)
    }
}

unsafe impl Send for TabuSearchBase {}