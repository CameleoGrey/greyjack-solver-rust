// src/score_calculation/score_calculators/greynet_score_calculator.rs

//! An incremental score calculator that uses the Greynet rule engine.

use crate::score_calculation::greynet::{ConstraintBuilder, Session};
use crate::score_calculation::scores::ScoreTrait;
use std::collections::HashMap;
use crate::score_calculation::greynet::fact::GreynetFact;
use crate::score_calculation::greynet::greynet_traits::ModifiableFact;
use rustc_hash::FxHashMap;
use std::rc::Rc;

/// An incremental score calculator that holds the Greynet session and provides
/// methods for the ScoreRequester to interact with it efficiently.
pub struct GreynetScoreCalculator<ScoreType>
where
    ScoreType: ScoreTrait + Clone + Send + 'static,
{
    pub(crate) session: Session<ScoreType>,
    /// Maps the solver's variable index to the fact instance and attribute name it controls.
    /// This map is populated by the `GreynetScoreRequester`.
    pub var_idx_to_entity_map: FxHashMap<usize, (Rc<dyn ModifiableFact + Send>, String)>,
    /// Reverse index: maps entity fact_id to all variable indices that point to it.
    /// This enables O(k) updates where k = number of changed entities.
    entity_id_to_var_indices: FxHashMap<i64, Vec<usize>>,
}

impl<ScoreType> GreynetScoreCalculator<ScoreType>
where
    ScoreType: ScoreTrait + Clone + Send + 'static,
{
    /// Initializes the calculator by building the Greynet session from the
    /// provided constraint definitions.
    pub fn new(constraint_builder: ConstraintBuilder<ScoreType>) -> Self {
        let session = constraint_builder.build().expect("Failed to build Greynet session");
        Self {
            session,
            var_idx_to_entity_map: FxHashMap::default(),
            entity_id_to_var_indices: FxHashMap::default(),
        }
    }

    /// Rebuilds the reverse index from the current var_idx_to_entity_map.
    /// This should be called by the GreynetScoreRequester after populating var_idx_to_entity_map.
    pub fn rebuild_reverse_index(&mut self) {
        self.entity_id_to_var_indices.clear();
        for (&var_idx, (entity, _)) in &self.var_idx_to_entity_map {
            self.entity_id_to_var_indices
                .entry(entity.fact_id())
                .or_insert_with(Vec::new)
                .push(var_idx);
        }
    }

    /// Performs the initial population of the Greynet session with all facts
    /// from the problem domain. This should only be called once by the requester.
    pub fn initial_load(
        &mut self,
        planning_entities: &HashMap<String, Vec<Rc<dyn ModifiableFact + Send>>>,
        problem_facts: &HashMap<String, Vec<Rc<dyn GreynetFact + Send>>>,
    ) {
        self.session.clear().expect("Failed to clear session");

        // FIX: Use the new `insert_dyn_fact` method on the session. This correctly
        // recovers the concrete TypeId from the trait object and routes the fact
        // to the `FromNode` defined in the user's constraints (e.g., for `GreynetQueen`).
        // This connects the data to the constraint network and solves the "score is 0" bug.

        // Insert problem facts
        for group in problem_facts.values() {
            for fact in group {
                self.session.insert_dyn_fact(fact.clone()).expect("Failed to insert problem fact");
            }
        }

        // Insert planning entities
        for group in planning_entities.values() {
            for fact in group {
                self.session.insert_dyn_fact(fact.clone()).expect("Failed to insert planning entity");
            }
        }
        
        self.session.flush().expect("Failed to flush after initial load");
    }

    /// Retrieves the current total score from the Greynet session.
    pub fn get_score(&mut self) -> ScoreType {
        self.session.get_score().expect("Failed to get score from session")
    }

    /// Applies a batch of deltas, gets the score for each, and reverts the state
    /// between each delta application. This is the primary method for incremental scoring.
    pub fn apply_and_get_score_for_batch(&mut self, deltas: &[Vec<(usize, f64)>]) -> Vec<ScoreType> {
        let mut scores = Vec::with_capacity(deltas.len());
        for delta_set in deltas {
            if delta_set.is_empty() {
                scores.push(self.get_score());
                continue;
            }

            let (originals, changed) = self.apply_deltas_internal(delta_set);
            scores.push(self.get_score());
            self.revert_deltas_internal(originals, changed);
        }
        scores
    }

    /// Commits a set of deltas to the session state permanently and updates the internal
    /// entity map to point to the new fact instances.
    /// OPTIMIZED: Uses reverse index for O(k) updates where k = number of changed entities.
    pub fn commit_deltas(&mut self, deltas: &[(usize, f64)]) {
        if deltas.is_empty() {
            return;
        }

        let (originals, changed_rcs, original_id_to_new_entity_map) = self.prepare_commit(deltas);

        if !originals.is_empty() {
            for fact in &originals {
                self.session.retract_by_id(fact.fact_id()).unwrap();
            }
            for fact in &changed_rcs {
                self.session.insert_dyn_fact(fact.clone()).unwrap();
            }
            self.session.flush().unwrap();
        }
        
        // OPTIMIZATION: Use reverse index to directly find and update affected variables
        // This is O(k * avg_vars_per_entity) instead of O(total_variables)
        for (old_entity_id, new_entity) in &original_id_to_new_entity_map {
            if let Some(var_indices) = self.entity_id_to_var_indices.get(old_entity_id) {
                // Update all variables that point to this entity
                for &var_idx in var_indices {
                    if let Some((entity, _attr)) = self.var_idx_to_entity_map.get_mut(&var_idx) {
                        *entity = new_entity.clone();
                    }
                }
                
                // Update the reverse index to point to the new entity
                if let Some(var_indices_owned) = self.entity_id_to_var_indices.remove(old_entity_id) {
                    self.entity_id_to_var_indices.insert(new_entity.fact_id(), var_indices_owned);
                }
            }
        }
    }

    /// Internal helper to apply changes to the session state by creating modified
    /// copies of facts, retracting the originals, and inserting the copies.
    fn apply_deltas_internal(&mut self, delta_set: &[(usize, f64)]) -> (Vec<Rc<dyn ModifiableFact + Send>>, Vec<Rc<dyn ModifiableFact + Send>>) {
        let mut original_to_changed_map: FxHashMap<i64, (Rc<dyn ModifiableFact + Send>, Box<dyn ModifiableFact + Send>)> = FxHashMap::default();

        for &(var_idx, new_value) in delta_set {
            if let Some((original_fact, attr_name)) = self.var_idx_to_entity_map.get(&var_idx) {
                let fact_id = original_fact.fact_id();
                let (_, changed_fact) = original_to_changed_map
                    .entry(fact_id)
                    .or_insert_with(|| (original_fact.clone(), original_fact.clone_modifiable()));
                
                changed_fact.set_field(attr_name, new_value);
            }
        }

        let mut originals = Vec::new();
        let mut changed = Vec::new();

        for (_, (orig, modified)) in original_to_changed_map {
            originals.push(orig);
            changed.push(Rc::from(modified) as Rc<dyn ModifiableFact + Send>);
        }

        if !originals.is_empty() {
            for fact in &originals { 
                self.session.retract_by_id(fact.fact_id()).unwrap(); 
            }
            for fact in &changed { 
                self.session.insert_dyn_fact(fact.clone()).unwrap(); 
            }
            self.session.flush().unwrap();
        }

        (originals, changed)
    }

    /// Internal helper to revert the changes made by `apply_deltas_internal`.
    fn revert_deltas_internal(&mut self, originals: Vec<Rc<dyn ModifiableFact + Send>>, changed: Vec<Rc<dyn ModifiableFact + Send>>) {
        if !changed.is_empty() {
            for fact in &changed { 
                self.session.retract_by_id(fact.fact_id()).unwrap(); 
            }
            for fact in &originals { 
                self.session.insert_dyn_fact(fact.clone()).unwrap(); 
            }
            self.session.flush().unwrap();
        }
    }

    /// Helper to prepare data for commit, separating logic from `commit_deltas`.
    fn prepare_commit(&self, deltas: &[(usize, f64)]) -> (Vec<Rc<dyn ModifiableFact + Send>>, Vec<Rc<dyn ModifiableFact + Send>>, FxHashMap<i64, Rc<dyn ModifiableFact + Send>>) {
        let mut original_to_changed_map: FxHashMap<i64, (Rc<dyn ModifiableFact + Send>, Box<dyn ModifiableFact + Send>)> = FxHashMap::default();

        for &(var_idx, new_value) in deltas {
             if let Some((original_fact, attr_name)) = self.var_idx_to_entity_map.get(&var_idx) {
                let fact_id = original_fact.fact_id();
                let (_, changed_fact) = original_to_changed_map
                    .entry(fact_id)
                    .or_insert_with(|| (original_fact.clone(), original_fact.clone_modifiable()));
                
                changed_fact.set_field(attr_name, new_value);
            }
        }

        let mut originals = Vec::new();
        let mut changed_rcs = Vec::new();
        let mut original_id_to_new_entity_map = FxHashMap::default();

        for (_, (orig, modified)) in original_to_changed_map {
            let changed_rc: Rc<dyn ModifiableFact + Send> = Rc::from(modified);
            originals.push(orig.clone());
            changed_rcs.push(changed_rc.clone());
            original_id_to_new_entity_map.insert(orig.fact_id(), changed_rc);
        }
        (originals, changed_rcs, original_id_to_new_entity_map)
    }
}