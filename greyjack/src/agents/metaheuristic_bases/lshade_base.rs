

use std::collections::HashMap;
use crate::score_calculation::score_requesters::VariablesManager;

use super::MetaheuristicBaseTrait;
use crate::score_calculation::scores::ScoreTrait;
use crate::agents::base::Individual;
use std::ops::AddAssign;
use std::fmt::Debug;

use rand::SeedableRng;
use rand::rngs::StdRng;
use rand_distr::{Distribution, Normal};

use super::Mover;
use super::metaheuristic_kinds_and_names::{MetaheuristicKind, MetaheuristicNames};
use std::collections::VecDeque;
use std::collections::HashSet;
use std::cmp::max;

use rand::Rng;
use rand::seq::SliceRandom;
use std::f64::consts::PI;
use rand::prelude::IteratorRandom;

/*
Classic Tanabe-Fukunaga version of LSHADE (https://metahack.org/CEC2014-Tanabe-Fukunaga.pdf) 
with my own modifications to make it better work in common (not only for continuous tasks, but also for MIP).
For pure integer tasks works much badder than Tabu, GenAlg, LateAcc. Don't use LSHADE for purely integer tasks.
Later will be modified further. From 2014 there were a lot of modifications invented by researchers.

WARNING! Don't use this metaheuristic with already initialized values (it will stuck due to sampling from history mechanism)! 
*/

pub struct LSHADEBase<ScoreType>
where ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send {

    pub population_size: usize,
    pub history_archive_size: usize,
    pub initial_f: f64,
    pub initial_cr: f64,
    pub initial_mutation_proba: f64,
    pub mutation_rate_multiplier: f64,
    pub p_best_rate: f64,
    pub memory_pruning_rate: f64,
    pub guarantee_of_change_size: usize,
    
    pub history_archive: Vec<Individual<ScoreType>>,
    pub adaptive_f: Vec<f64>,
    pub adaptive_cr: Vec<f64>,
    pub adaptive_mutation_proba: Vec<f64>,
    pub current_history_archive_size: usize,
    pub k: usize,
    pub minimal_history_size: usize,
    
    pub history_f: Vec<f64>,
    pub history_cr: Vec<f64>,
    pub history_cr_ids: Vec<usize>,
    pub generated_f_list: Vec<f64>,
    pub generated_cr_list: Vec<f64>,
    pub previous_population_scores: Vec<ScoreType>,

    pub tabu_entity_rate: f64,

    pub metaheuristic_kind: MetaheuristicKind,
    pub metaheuristic_name: MetaheuristicNames,

    pub group_mutation_rates_map: HashMap<String, f64>,
    pub discrete_ids: Option<Vec<usize>>,
    pub mover: Mover,

    pub random_generator: StdRng,
}

impl<ScoreType> LSHADEBase<ScoreType>
where ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send {

    pub fn new(
        population_size: usize, // Very task specific. Typically I set 32, 64, 128, 256, etc
        history_archive_size: usize, // recommendation from article: 100
        p_best_rate: f64, // 0.05 - faster convergence, 0.2 - better exploration
        memory_pruning_rate: f64, // My idea to use prunning instead of changing archive size. If != 0, gives faster convergence, but stucks more often
        guarantee_of_change_size: usize, // in article there are always changing at least 1 component of vector, but I found during prototyping, that 0 can be useful too
        initial_f: f64, // recommendation from article: 0.5
        initial_cr: f64, // recommendation from article: 0.5, I use 0.02
        initial_mutation_proba: f64,
        tabu_entity_rate: f64,
        mutation_rate_multiplier: Option<f64>,
        move_probas: Option<Vec<f64>>,
        semantic_groups_dict: HashMap<String, Vec<usize>>,
        discrete_ids: Option<Vec<usize>>,
    ) -> Self {

        let current_mutation_rate_multiplier;
        match mutation_rate_multiplier {
            Some(x) => current_mutation_rate_multiplier = mutation_rate_multiplier.unwrap(),
            None => current_mutation_rate_multiplier = 0.0 // 0.0 - always use minimal possible move size, 1.0 - is more intuitive,
        }
        let mut group_mutation_rates_map: HashMap<String, f64> = HashMap::new();
        for group_name in semantic_groups_dict.keys() {
            let group_size = semantic_groups_dict[group_name].len();
            let current_group_mutation_rate = current_mutation_rate_multiplier * (1.0 / (group_size as f64));
            group_mutation_rates_map.insert(group_name.clone(), current_group_mutation_rate);
        }

        Self {
            population_size: population_size,
            history_archive_size: history_archive_size,
            initial_f: initial_f,
            initial_cr: initial_cr,
            initial_mutation_proba: initial_mutation_proba,
            mutation_rate_multiplier: current_mutation_rate_multiplier,
            p_best_rate: p_best_rate,
            memory_pruning_rate: memory_pruning_rate,
            guarantee_of_change_size: guarantee_of_change_size,

            history_archive: Vec::new(),
            adaptive_f: vec![initial_f; history_archive_size],
            adaptive_cr: vec![initial_cr; history_archive_size],
            adaptive_mutation_proba: vec![initial_mutation_proba; history_archive_size],
            current_history_archive_size: history_archive_size,
            k: 0,
            minimal_history_size: 16, // from experiments. Lesser value will cause stucks due to possible situation of "no different vectors"

            history_f: Vec::new(),
            history_cr: Vec::new(),
            history_cr_ids: Vec::new(),

            generated_f_list: Vec::new(),
            generated_cr_list: Vec::new(),
            previous_population_scores: Vec::new(),

            tabu_entity_rate: tabu_entity_rate,

            metaheuristic_kind: MetaheuristicKind::Population,
            metaheuristic_name: MetaheuristicNames::LSHADE,

            group_mutation_rates_map: group_mutation_rates_map.clone(),
            discrete_ids: discrete_ids.clone(),
            mover: Mover::new(tabu_entity_rate, HashMap::new(), HashMap::new(), HashMap::new(), group_mutation_rates_map.clone(), move_probas),
            random_generator: StdRng::from_entropy(),
        }
    }

    fn adapt_parameters(&mut self, population: &mut Vec<Individual<ScoreType>>) {

        // Algorithm 1: Memory update algorithm in SHADE 1.1; Eq. (7), (8), (9)
        if !self.history_f.is_empty() && !self.history_cr.is_empty() {
            let archive_size = self.history_cr.len();
            let score_deltas: Vec<f64> = self.history_cr_ids.iter()
                .map(|&i| {
                    let previous_score = self.previous_population_scores[i].get_priority_score();
                    let current_score = population[i].score.get_priority_score();
                    (current_score - previous_score).abs()
                })
                .collect();
            
            let sum_delta: f64 = score_deltas.iter().sum();
            let weights: Vec<f64> = if sum_delta == 0.0 {
                vec![0.0; archive_size]
            } else {
                score_deltas.iter().map(|&d| d / sum_delta).collect()
            };
            
            let new_cr_k = self.calculate_weighted_lehmer_mean(&self.history_cr, &weights);
            self.adaptive_cr[self.k] = if new_cr_k > 0.0 { new_cr_k } else { self.initial_cr };
            
            self.adaptive_mutation_proba[self.k] = 1.0 - new_cr_k;
            
            let new_f_k = self.calculate_weighted_lehmer_mean(&self.history_f, &weights);
            self.adaptive_f[self.k] = if new_f_k > 0.0 { new_f_k } else { self.initial_f };
            
            self.k += 1;
            if self.k >= self.current_history_archive_size {
                self.k = 0;
            }
        }
        
        // TODO: set accomplish rate like it was for Simulated Annealing.
        //self.current_history_archive_size = (self.history_archive_size as f64 + self.termination_strategy.get_accomplish_rate() * 
        //    (self.minimal_history_size as f64 - self.history_archive_size as f64)).round() as usize;
    }


    // https://ru.wikipedia.org/wiki/%D0%A0%D0%B0%D1%81%D0%BF%D1%80%D0%B5%D0%B4%D0%B5%D0%BB%D0%B5%D0%BD%D0%B8%D0%B5_%D0%9A%D0%BE%D1%88%D0%B8
    fn get_cauchy(&mut self, loc: f64, scale: f64) -> f64 {
        let uniform_value = self.random_generator.gen::<f64>();
        loc + scale * (PI * (uniform_value - 0.5)).tan()
    }
    
    fn calculate_weighted_lehmer_mean(&self, values: &Vec<f64>, weights: &Vec<f64>) -> f64 {
        let numerator: f64 = values.iter().zip(weights.iter()).map(|(v, w)| w * v * v).sum();
        let divider: f64 = values.iter().zip(weights.iter()).map(|(v, w)| w * v).sum();
        
        if divider == 0.0 {
            0.0
        } else {
            numerator / divider
        }
    }

}

impl<ScoreType> MetaheuristicBaseTrait<ScoreType> for LSHADEBase<ScoreType>
where ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send {

    fn sample_candidates_plain(
            &mut self, 
            population: &mut Vec<Individual<ScoreType>>, 
            current_top_individual: &Individual<ScoreType>,
            variables_manager: &VariablesManager
        ) -> Vec<Vec<f64>> {

        if self.mover.tabu_entity_size_map.len() == 0 {
            let semantic_groups_map = variables_manager.semantic_groups_map.clone();
            for (group_name, group_ids) in semantic_groups_map {
                self.mover.tabu_ids_sets_map.insert(group_name.clone(), HashSet::new());
                self.mover.tabu_entity_size_map.insert(group_name.clone(), max((self.tabu_entity_rate * (group_ids.len() as f64)).ceil() as usize, 1));
                self.mover.tabu_ids_vecdeque_map.insert(group_name.clone(), VecDeque::new());
            }
        }
        
        population.sort();

        self.generated_f_list = vec![0.0; self.population_size];
        self.generated_cr_list = vec![0.0; self.population_size];
        self.previous_population_scores = population.iter().map(|ind| ind.score.clone()).collect();

        let mut candidates = Vec::with_capacity(self.population_size);
        
        for i in 0..self.population_size {
            let random_id = self.random_generator.gen_range(0..self.adaptive_cr.len());
            let current_cr = (Normal::new(self.adaptive_cr[random_id], 0.1).unwrap().sample(&mut self.random_generator)).clamp(0.0, 1.0);
            let mutation_proba = (Normal::new(self.adaptive_mutation_proba[random_id], 0.1).unwrap().sample(&mut self.random_generator)).clamp(0.0, 1.0);
            
            // LSHADE uses technique from JADE
            let mut current_f = -1.0;
            while current_f <= 0.0 {
                current_f = self.get_cauchy(self.adaptive_f[random_id], 0.1);
                current_f = current_f.min(1.0);
            }
            
            self.generated_cr_list[i] = current_cr;
            self.generated_f_list[i] = current_f;
            
            let p_best_proba = self.random_generator.gen_range(0.00001..self.p_best_rate);
            let last_top_id = (p_best_proba * self.population_size as f64).ceil() as usize;
            let p_best_vector = &population[self.random_generator.gen_range(0..last_top_id)].variable_values;
            let current_vector = &population[i].variable_values;
            
            let united_population: Vec<&Individual<ScoreType>> = population.iter().chain(self.history_archive.iter()).collect();
            
            // chosing both vectors from united_population works better (less stucks in local minimums)
            let random_vector_1 = &united_population[self.random_generator.gen_range(0..united_population.len())].variable_values;
            let random_vector_2 = loop {
                let vec = &united_population[self.random_generator.gen_range(0..united_population.len())].variable_values;

                // (3) diffence of vectors condition
                let diff1 = random_vector_1.iter().zip(vec.iter()).map(|(a, b)| (a - b).abs()).sum::<f64>();
                let diff2 = vec.iter().zip(current_vector.iter()).map(|(a, b)| (a - b).abs()).sum::<f64>();
                if diff1 != 0.0 && diff2 != 0.0 {
                    break vec;
                }
            };
            
            let mut crossover_vector: Vec<f64> = current_vector.iter()
                .zip(p_best_vector.iter())
                .zip(random_vector_1.iter())
                .zip(random_vector_2.iter())
                .map(|(((x, p), r1), r2)| x + current_f * (p - x) + current_f * (r1 - r2))
                .collect();
            
            let (mut candidate_vector, changed_columns) = 
            if self.random_generator.gen::<f64>() < 0.5 {
                let crossover_mask: Vec<bool> = (0..variables_manager.variables_count)
                    .map(|_| self.random_generator.gen::<f64>() < current_cr)
                    .collect();
                
                let candidate: Vec<f64> = crossover_mask.iter()
                    .zip(current_vector.iter())
                    .zip(crossover_vector.iter())
                    .map(|((mask, curr), cross)| if *mask { *cross } else { *curr })
                    .collect();
                
                let changed_columns: Vec<usize> = crossover_mask.iter()
                    .enumerate()
                    .filter(|(_, &mask)| mask)
                    .map(|(i, _)| variables_manager.variable_ids[i])
                    .collect();
                
                let changed_columns =  if changed_columns.len() == 0 {None} else {Some(changed_columns)};

                (candidate, changed_columns)

            } else if self.random_generator.gen::<f64>() <= mutation_proba {
                // my modification to prevent population degeneration and adapt LSHADE to mixed variable types cases
                // take the whole crossover_vec and make mutation (move)
                // p_best crossover changes all columns
                let (candidate, _, _) = self.mover.do_move(&mut crossover_vector, &variables_manager, false);
                let changed_columns: Vec<usize> = (0..crossover_vector.len()).collect();
                (candidate.unwrap(), Some(changed_columns))
            } else {
                let crossover_mask: Vec<bool> = (0..variables_manager.variables_count)
                    .map(|_| self.random_generator.gen::<f64>() < current_cr)
                    .collect();
                
                let candidate: Vec<f64> = crossover_mask.iter()
                    .zip(current_vector.iter())
                    .zip(crossover_vector.iter())
                    .map(|((mask, curr), cross)| if *mask { *cross } else { *curr })
                    .collect();
                
                let changed_columns: Vec<usize> = crossover_mask.iter()
                    .enumerate()
                    .filter(|(_, &mask)| mask)
                    .map(|(i, _)| variables_manager.variable_ids[i])
                    .collect();

                let changed_columns =  if changed_columns.len() == 0 {None} else {Some(changed_columns)};
                (candidate, changed_columns)
            };
            
            let mut candidate_vector = candidate_vector;
            if self.guarantee_of_change_size > 0 {
                let current_change_count = self.random_generator.gen_range(1..=self.guarantee_of_change_size);
                let columns_to_change: Vec<usize> = (0..variables_manager.variables_count).choose_multiple(&mut self.random_generator, current_change_count);
                
                for &col in &columns_to_change {
                    candidate_vector[col] = crossover_vector[col];
                }
                
                variables_manager.fix_variables(&mut candidate_vector, Some(columns_to_change));
            }
            variables_manager.fix_variables(&mut candidate_vector, changed_columns);
            candidates.push(candidate_vector);
        }
        
        return candidates;
    }

    fn sample_candidates_incremental(
        &mut self,
        population: &mut Vec<Individual<ScoreType>>, 
        current_top_individual: &Individual<ScoreType>,
        variables_manager: &VariablesManager
    ) -> (Vec<f64>, Vec<Vec<(usize, f64)>>) {
        panic!("Incremental candidates sampling is available only for local search approaches (TabuSearch, LateAcceptance, etc).")
    }

    fn build_updated_population(
        &mut self, 
        current_population: &Vec<Individual<ScoreType>>, 
        candidates: &mut Vec<Individual<ScoreType>>
        ) -> Vec<Individual<ScoreType>> {
        
        let mut new_population: Vec<Individual<ScoreType>> = Vec::new();

        // Fill history
        for i in 0..self.population_size {
            if candidates[i].score.get_priority_score() < current_population[i].score.get_priority_score() {
                self.history_archive.push(candidates[i].clone());
                self.history_cr.push(self.generated_cr_list[i]);
                self.history_f.push(self.generated_f_list[i]);
                self.history_cr_ids.push(i);
            }
            
            if candidates[i].score.get_priority_score() <= current_population[i].score.get_priority_score() {
                new_population.push(candidates[i].clone());
            } else {
                new_population.push(current_population[i].clone());
            }
        }
        
        // Memory pruning
        let samples_to_remember = ((1.0 - self.memory_pruning_rate) * self.history_archive_size as f64).ceil() as usize;
        if self.history_archive.len() > self.current_history_archive_size {
            let samples_to_forget_count = self.history_archive.len() - samples_to_remember;
            
            if samples_to_forget_count > 0 {
                let indices: Vec<usize> = (0..self.history_archive.len()).collect();
                let chosen = indices.choose_multiple(&mut self.random_generator, samples_to_forget_count);
                let to_remove: HashSet<_> = chosen.collect();
                
                let mut pruned_archive = Vec::new();
                let mut pruned_f = Vec::new();
                let mut pruned_cr = Vec::new();
                let mut pruned_cr_ids = Vec::new();
                
                for i in 0..self.history_archive.len() {
                    if !to_remove.contains(&i) {
                        pruned_archive.push(self.history_archive[i].clone());
                        pruned_cr.push(self.history_cr[i]);
                        pruned_f.push(self.history_f[i]);
                        pruned_cr_ids.push(self.history_cr_ids[i]);
                    }
                }
                
                self.history_archive = pruned_archive;
                self.history_cr = pruned_cr;
                self.history_f = pruned_f;
                self.history_cr_ids = pruned_cr_ids;
            }
        }

        self.adapt_parameters(&mut new_population);

        return new_population;
    }

    fn build_updated_population_incremental(
            &mut self, 
            current_population: &Vec<Individual<ScoreType>>, 
            sample: &mut Vec<f64>,
            deltas: Vec<Vec<(usize, f64)>>,
            scores: Vec<ScoreType>,
        ) -> Vec<Individual<ScoreType>> {
        
        panic!("Incremental candidates sampling is available only for local search approaches (TabuSearch, LateAcceptance, etc).")
    }

    fn get_metaheuristic_kind(&self) -> MetaheuristicKind {
        self.metaheuristic_kind.clone()
    }

    fn get_metaheuristic_name(&self) -> MetaheuristicNames {
        self.metaheuristic_name.clone()
    }
}

unsafe impl<ScoreType> Send for LSHADEBase<ScoreType>
where ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send {}