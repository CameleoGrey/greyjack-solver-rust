// src/score_calculation/score_requesters/greynet_score_requester.rs

//! The integration layer between the solver and the Greynet score calculator.

use crate::cotwin::Cotwin;
use crate::score_calculation::score_calculators::greynet_score_calculator::GreynetScoreCalculator;
use crate::score_calculation::greynet::greynet_traits::{InitializableFact, ModifiableFact};
use crate::score_calculation::greynet::fact::GreynetFact;
use crate::score_calculation::score_calculators::ScoreCalculatorVariants;
use crate::score_calculation::scores::ScoreTrait;
use std::ops::AddAssign;
use crate::variables::PlanningVariablesVariants;
use crate::cotwin::{CotwinEntityTrait, CotwinValueTypes};
use super::variables_manager::VariablesManager;
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::rc::Rc;

/// This requester is responsible for initializing the Greynet session and translating
/// the solver's incremental deltas into fact updates for the Greynet engine.
pub struct GreynetScoreRequester<EntityVariants, UtilityObjectVariants, ScoreType>
where
    ScoreType: ScoreTrait + Clone + Send + AddAssign + 'static,
    EntityVariants: InitializableFact + CotwinEntityTrait + Clone + Send + 'static,
{
    pub cotwin: Cotwin<EntityVariants, UtilityObjectVariants, ScoreType>,
    pub variables_manager: VariablesManager,
    pub calculator: GreynetScoreCalculator<ScoreType>,
}

impl<EntityVariants, UtilityObjectVariants, ScoreType> GreynetScoreRequester<EntityVariants, UtilityObjectVariants, ScoreType>
where
    ScoreType: ScoreTrait + Clone + Send + AddAssign + 'static,
    EntityVariants: InitializableFact + CotwinEntityTrait + Clone + Send + 'static,
{
    /// Creates a new requester, which involves a full setup of the Greynet session.
    pub fn new(
        mut cotwin: Cotwin<EntityVariants, UtilityObjectVariants, ScoreType>,
    ) -> Self {
        // Build the variables manager from the cotwin definition
        let (variables_vec, _, _) = Self::build_variables_info(&cotwin);
        let variables_manager = VariablesManager::new(variables_vec);

        // --- Greynet Initialization ---

        // 1. Create concrete fact instances from entity definitions with initial values.
        let mut initialized_planning_entities: HashMap<String, Vec<Rc<dyn ModifiableFact + Send>>> = HashMap::new();
        for (group_name, entities) in &cotwin.planning_entities {
            let initialized_group: Vec<_> = entities.iter().map(|e| e.to_initialized_fact()).collect();
            initialized_planning_entities.insert(group_name.clone(), initialized_group);
        }

        // 2. Build the crucial mapping from the solver's variable index to the corresponding
        //    fact instance and attribute name.
        let mut var_idx_to_entity_map = FxHashMap::default();
        let mut i = 0;
        // The order of iteration here MUST match the order in `build_variables_info`.
        for group_name in cotwin.planning_entities.keys() {
            let native_entities = &cotwin.planning_entities[group_name];
            let initialized_facts = &initialized_planning_entities[group_name];

            for (native_entity, initialized_fact) in native_entities.iter().zip(initialized_facts.iter()) {
                for (attr_name, attr_value) in native_entity.to_vec() {
                    if matches!(attr_value, CotwinValueTypes::GJF(_) | CotwinValueTypes::GJI(_)) {
                        var_idx_to_entity_map.insert(i, (initialized_fact.clone(), attr_name));
                        i += 1;
                    }
                }
            }
        }

        // Take ownership of the score calculator from the cotwin, replacing it with a placeholder.
        let score_calculator_variant = std::mem::replace(
            &mut cotwin.score_calculator,
            ScoreCalculatorVariants::None,
        );

        let mut calculator = match score_calculator_variant {
            ScoreCalculatorVariants::Greynet(sc_gnt) => sc_gnt,
            _ => panic!("Only GreynetScoreCalculator is compatible with GreynetScoreRequester"),
        };

        calculator.var_idx_to_entity_map = var_idx_to_entity_map;
        calculator.rebuild_reverse_index();

        // 3. Perform the initial load of all facts into the Greynet session.
        let mut problem_facts_as_greynet: HashMap<String, Vec<Rc<dyn GreynetFact + Send>>> = HashMap::new();
        for (group_name, entities) in &cotwin.problem_facts {
             let fact_group: Vec<Rc<dyn GreynetFact + Send>> = entities
                .iter()
                // This assumes your problem facts also implement `InitializableFact` and thus `GreynetFact`.
                // If they are different structs, you may need a separate conversion path.
                .map(|e| e.to_initialized_fact() as Rc<dyn GreynetFact + Send>)
                .collect();
            problem_facts_as_greynet.insert(group_name.clone(), fact_group);
        }
        calculator.initial_load(&initialized_planning_entities, &problem_facts_as_greynet);

        Self {
            cotwin,
            variables_manager,
            calculator,
        }
    }

    /// Creates the list of `PlanningVariablesVariants` from the cotwin definition.
    /// The order of creation here is critical and must match the order of `var_idx_to_entity_map` creation.
    fn build_variables_info(cotwin: &Cotwin<EntityVariants, UtilityObjectVariants, ScoreType>)
        -> (Vec<PlanningVariablesVariants>, HashMap<String, usize>, HashMap<usize, String>) {
        
        let mut variables_vec: Vec<PlanningVariablesVariants> = Vec::new();
        let mut var_name_to_vec_id_map: HashMap<String, usize> = HashMap::new();
        let mut vec_id_to_var_name_map: HashMap<usize, String> = HashMap::new();

        let mut i:usize = 0;
        for planning_entities_group_name in cotwin.planning_entities.keys() {
            let current_planning_entities_group = &cotwin.planning_entities[planning_entities_group_name];
            for entity in current_planning_entities_group.iter() {
                let entity_attributes_map = entity.to_vec();

                for (attribute_name, attribute_value) in entity_attributes_map {
                    // Use the global variable index `i` in the name for consistency.
                    let full_variable_name = format!("{}: {}-->{}", planning_entities_group_name, i, attribute_name);
                    let variable = match attribute_value {
                        CotwinValueTypes::GJF(mut float_value) => {
                            float_value.set_name(full_variable_name.clone());
                            Some(PlanningVariablesVariants::GJF(float_value.clone()))
                        },
                        CotwinValueTypes::GJI(mut integer_value) => {
                            integer_value.set_name(full_variable_name.clone());
                            Some(PlanningVariablesVariants::GJI(integer_value.clone()))
                        },
                        CotwinValueTypes::PAV(_) => None,
                    };
                    
                    if let Some(var) = variable {
                        var_name_to_vec_id_map.insert(full_variable_name.clone(), i);
                        vec_id_to_var_name_map.insert(i, full_variable_name);
                        variables_vec.push(var);
                        i += 1;
                    }
                }
            }
        }

        (variables_vec, var_name_to_vec_id_map, vec_id_to_var_name_map)
    }

    /// Forwards an incremental score request to the Greynet calculator.
    pub fn request_score_incremental(&mut self, _sample: &[f64], deltas: &[Vec<(usize, f64)>]) -> Vec<ScoreType> {
        self.calculator.apply_and_get_score_for_batch(deltas)
    }

    /// Forwards a request to commit the final solution to the Greynet calculator.
    pub fn commit_deltas(&mut self, deltas: &[(usize, f64)]) {
        self.calculator.commit_deltas(deltas);
    }
}

unsafe impl<EntityVariants, UtilityObjectVariants, ScoreType> Send for GreynetScoreRequester<EntityVariants, UtilityObjectVariants, ScoreType>
where 
    ScoreType: ScoreTrait + Clone + AddAssign + Send,
    EntityVariants: CotwinEntityTrait + InitializableFact + Clone + Send {}
