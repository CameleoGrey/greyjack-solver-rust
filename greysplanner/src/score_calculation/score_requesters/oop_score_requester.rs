

use crate::cotwin::Cotwin;
use crate::cotwin::CotwinEntityTrait;
use crate::cotwin::CotwinValueTypes;
use crate::cotwin::CotwinValueTypes::*;
use crate::variables::PlanningVariablesVariants;
use crate::score_calculation::scores::ScoreTrait;
use crate::score_calculation::score_requesters::VariablesManager;

use std::ops::AddAssign;
use std:: collections::HashMap;
use std::string::String;
use ndarray::Array1;
use polars::prelude::*;


pub struct OOPScoreRequester<EntityVariants, UtilityObjectVariants, ScoreType>
where
    ScoreType: ScoreTrait + Clone + AddAssign + Send{
        pub cotwin: Cotwin<EntityVariants, UtilityObjectVariants, ScoreType>,
        pub variables_manager: VariablesManager,

        pub var_name_to_df_col_names: HashMap<String, (String, String)>,
        pub var_name_to_vec_id_map: HashMap<String, usize>,
        pub vec_id_to_var_name_map: HashMap<usize, String>,
        pub df_column_var_ids: HashMap<(String, String), Vec<usize>>,
        pub var_id_to_df_name: Vec<String>,
        pub var_id_to_col_name: Vec<String>,
        
        pub cached_sample_id_vectors: HashMap<String, Vec<u64>>,
        pub cached_sample_size: usize,

        pub planning_entities_column_map: HashMap<String, Vec<String>>,
        pub problem_facts_column_map: HashMap<String, Vec<String>>,
        pub planning_entity_dfs: HashMap<String, DataFrame>,
        pub problem_fact_dfs: HashMap<String, DataFrame>,
        pub raw_dfs: HashMap<String, DataFrame>,
}

impl<EntityVariants, UtilityObjectVariants, ScoreType> 
OOPScoreRequester<EntityVariants, UtilityObjectVariants, ScoreType>
where
    ScoreType: ScoreTrait + Clone + AddAssign + Send,
    EntityVariants: CotwinEntityTrait {

        pub fn new(cotwin: Cotwin<EntityVariants, UtilityObjectVariants, ScoreType>) -> Self {

            let (variables_vec, var_name_to_vec_id_map, vec_id_to_var_name_map) = Self::build_variables_info(&cotwin);
            let variables_manager = VariablesManager::new(variables_vec);

            let planning_entities_column_map = Self::build_column_map(&cotwin.planning_entities);
            let problem_facts_column_map = Self::build_column_map(&cotwin.problem_facts);
            let planning_entity_dfs = Self::build_group_dfs(&cotwin.planning_entities, &planning_entities_column_map);
            let problem_fact_dfs = Self::build_group_dfs(&cotwin.problem_facts, &problem_facts_column_map);
            let dfs_for_scoring = planning_entity_dfs.clone();

            
            let mut score_requester = Self {
                planning_entities_column_map: planning_entities_column_map,
                problem_facts_column_map: problem_facts_column_map,
                planning_entity_dfs: planning_entity_dfs,
                problem_fact_dfs: problem_fact_dfs,
                raw_dfs: dfs_for_scoring,

                cotwin: cotwin,
                variables_manager: variables_manager,

                var_name_to_df_col_names: HashMap::new(),
                var_name_to_vec_id_map: var_name_to_vec_id_map,
                vec_id_to_var_name_map: vec_id_to_var_name_map,
                df_column_var_ids: HashMap::new(),
                var_id_to_df_name: Vec::new(),
                var_id_to_col_name: Vec::new(),

                cached_sample_id_vectors: HashMap::new(),
                cached_sample_size: 999_999_999

            };

            return score_requester;
        }

        fn build_variables_info(cotwin: &Cotwin<EntityVariants, UtilityObjectVariants, ScoreType>) 
        -> (Vec<PlanningVariablesVariants>, HashMap<String, usize>, HashMap<usize, String>) {
            
            let mut variables_vec: Vec<PlanningVariablesVariants> = Vec::new();
            let mut var_name_to_vec_id_map: HashMap<String, usize> = HashMap::new();
            let mut vec_id_to_var_name_map: HashMap<usize, String> = HashMap::new();


            let mut i:usize = 0;
            for planning_entities_group_name in cotwin.planning_entities.keys() {
                let current_planning_entities_group = &cotwin.planning_entities[planning_entities_group_name];
                for entity in current_planning_entities_group {
                    
                    let mut entity_attributes_map;
                    match entity {
                        x => entity_attributes_map = entity.to_hash_map()
                    }
                    
                    let mut cloned_keys: Vec<String> = Vec::new();
                    for key in entity_attributes_map.keys() {
                        cloned_keys.push(key.clone());
                    }

                    for attribute_name in cloned_keys {
                        let full_variable_name = planning_entities_group_name.to_owned() + ": " + &i.to_string() + "-->" + &attribute_name;
                        let attribute_value = entity_attributes_map.get_mut(&attribute_name).unwrap();
                        let variable;
                        match attribute_value {
                            GPFloatVar(float_value) => {
                                float_value.set_name(full_variable_name.clone());
                                variable = PlanningVariablesVariants::GPFloatVar(float_value.clone());
                            },
                            GPIntegerVar(integer_value) => {
                                integer_value.set_name(full_variable_name.clone());
                                variable = PlanningVariablesVariants::GPIntegerVar(integer_value.clone())
                            },
                            PolarsAnyValue(_) => continue,
                        }
                        
                        var_name_to_vec_id_map.insert(full_variable_name.clone(), i);
                        vec_id_to_var_name_map.insert(i, full_variable_name.clone());
                        variables_vec.push(variable);
                        i += 1;
                    }
                }
            }

            return (variables_vec, var_name_to_vec_id_map, vec_id_to_var_name_map);
        }

        fn build_column_map(entity_groups: &HashMap<String, Vec<EntityVariants>>) -> HashMap<String, Vec<String>> {
            let mut column_map: HashMap<String, Vec<String>> = HashMap::new();

            for group_name in entity_groups.keys() {
                let mut group_columns: Vec<String> = Vec::new();
                let entity_objects = entity_groups.get(group_name).unwrap();
                let sample_object = &entity_objects[0];
                let entity_field_names = sample_object.to_hash_map();
                for field_name in entity_field_names.keys() {
                    group_columns.push(field_name.clone());
                }
                column_map.insert(group_name.clone(), group_columns);
            }

            return column_map;
        }
    
        fn build_group_dfs(entity_groups: &HashMap<String, Vec<EntityVariants>>, column_map: &HashMap<String, Vec<String>>) -> HashMap<String, DataFrame> {

            let mut df_map: HashMap<String, DataFrame> = HashMap::new();

            for df_name in column_map.keys() {
                
                let mut column_names: Vec<String> = Vec::new();
                for column_name in &column_map[df_name] {
                    column_names.push(column_name.clone());
                }

                let entity_group = &entity_groups[df_name];
                let entities_count = entity_group.len();

                let mut entity_fields_data: HashMap<String, Vec<AnyValue>> = HashMap::new();
                for column_name in &column_names {
                    entity_fields_data.insert(column_name.clone(), Vec::new());
                }

                for entity_object in entity_group {
                    let entity_map_representation = entity_object.to_hash_map();
                    for field_name in entity_map_representation.keys() {
                        let field_cotwin_value = &entity_map_representation[field_name];
                        let field_polars_value;
                        match field_cotwin_value {
                            CotwinValueTypes::GPFloatVar(x) => field_polars_value = AnyValue::Null,
                            CotwinValueTypes::GPIntegerVar(x) => field_polars_value = AnyValue::Null,
                            CotwinValueTypes::PolarsAnyValue(x) => field_polars_value = x.clone()
                        }
                        entity_fields_data.get_mut(field_name).unwrap().push( field_polars_value );
                    }
                }
                
                let mut columns_vec: Vec<Column> = Vec::new();
                for column_name in &column_names {
                    let entity_field_vec = &entity_fields_data[column_name];
                    let entity_field_column: Column = Column::new(column_name.into(), entity_field_vec);
                    columns_vec.push(entity_field_column);
                }
                
                column_names.push("sample_id".to_string());
                let sample_column_values= vec![AnyValue::UInt64(0); entities_count];
                let sample_id_column = Column::new("sample_id".into(), sample_column_values);
                columns_vec.push(sample_id_column);

                let df = DataFrame::new(columns_vec).unwrap();
                df_map.insert(df_name.clone(), df);
            }

            return df_map;
        }

        fn update_dfs_for_scoring(&mut self, group_data_map: HashMap<String, HashMap<String, Vec<AnyValue>>>, samples_count: usize) {

            for df_name in group_data_map.keys() {
                let mut current_df = self.planning_entity_dfs[df_name].clone();
                let needful_rows_count  = samples_count * self.raw_dfs[df_name].size();
                if current_df.size() != needful_rows_count {
                    let mut new_df_parts: Vec<LazyFrame> = Vec::new();
                    for i in 0..samples_count {
                        new_df_parts.push(self.raw_dfs[df_name].clone().lazy());
                    }
                    current_df = concat(new_df_parts, UnionArgs::default()).unwrap().collect().unwrap();
                }

                for column_name in group_data_map[df_name].keys() {
                    current_df.drop_in_place(column_name).unwrap();
                    let updated_column_data = &group_data_map[df_name][column_name];
                    let updated_column = Series::new(column_name.into(), updated_column_data);
                    current_df.with_column(updated_column).unwrap();
                }
                current_df.rechunk_mut();

                self.planning_entity_dfs.insert(df_name.clone(), current_df.clone());
            }

        }

        fn get_df_column_name(variable_name: String) -> (String, String) {

            let df_name:Vec<&str> = variable_name.split(": ").collect();
            let df_name = df_name[0].to_string();

            let column_name:Vec<&str> = variable_name.split("-->").collect();
            let column_name = column_name[column_name.len() - 1].to_string();

            return (df_name, column_name);
        }

        fn build_var_mappings(&mut self) -> HashMap<(String, String), Vec<usize>> {
            let variable_names= self.variables_manager.get_variables_names_vec();
            let mut df_column_var_ids: HashMap<(String, String), Vec<usize>> = HashMap::new();
            variable_names.iter().enumerate().for_each(|(i, var_name)| {
                let (df_name, column_name) = &Self::get_df_column_name(var_name.clone());

                self.var_id_to_df_name.push(df_name.clone());
                self.var_id_to_col_name.push(column_name.clone());

                if df_column_var_ids.contains_key(&(df_name.clone(), column_name.clone())) == false {
                    df_column_var_ids.insert((df_name.clone(), column_name.clone()), Vec::new());
                }
                    
                df_column_var_ids.get_mut(&(df_name.clone(), column_name.clone())).unwrap().push(i);

            });

            return df_column_var_ids;
        }

        fn build_group_data_map<'a>(&mut self, samples_vec: &Vec<Vec<AnyValue<'a>>>) -> HashMap<String, HashMap<String, Vec<AnyValue<'a>>>> {

            //let start_time = chrono::Utc::now().timestamp_millis();
            if self.df_column_var_ids.len() == 0 {
                self.df_column_var_ids = self.build_var_mappings();
            }

            let mut group_data_map: HashMap<String, HashMap<String, Vec<AnyValue>>> = HashMap::new();
            let n_variables = self.variables_manager.variables_count;

            for (df_name, col_name) in self.df_column_var_ids.keys() {
                group_data_map.insert(df_name.clone(), HashMap::new());
                group_data_map.get_mut(df_name).unwrap().insert(col_name.clone(), Vec::new());
            }

            /*let samples_count = samples_vec.len();
            for i in 0..samples_count {
                for j in 0..n_variables {
                    group_data_map
                    .get_mut(&self.var_id_to_df_name[j]).unwrap()
                    .get_mut(&self.var_id_to_col_name[j]).unwrap()
                    .push(samples_vec[i][j].clone());
                }
            }*/

            // much faster
            let stub_collection_1: () = self.df_column_var_ids.iter().map(|(df_col_name, var_ids)| {
                let stub_collection_2: () = samples_vec.iter().map(|sample_vec| {
                    let mut current_sample_column: Vec<AnyValue> = var_ids.iter().map(|i| sample_vec[*i].clone()).collect();
                    group_data_map
                    .get_mut(&df_col_name.0).unwrap()
                    .get_mut(&df_col_name.1).unwrap()
                    .append(&mut current_sample_column);
                }).collect();
            }).collect();

            //println!("fill map by data time: {}", chrono::Utc::now().timestamp_millis() - start_time );

            //let start_time = chrono::Utc::now().timestamp_millis();
            // add correct sample ids
            if samples_vec.len() != self.cached_sample_size {
                let df_names: Vec<String> = group_data_map.keys().map(|x| x.clone()).collect();
                for df_name in df_names {
                    let group_keys = group_data_map.get(&df_name).unwrap().keys().into_vec();
                    let first_group_key = group_keys.get(0).unwrap().as_str();
                    let updated_df_column_len = group_data_map.get(&df_name).unwrap().get(first_group_key).unwrap().len();
                    let samples_count = samples_vec.len();
                    let true_df_len = updated_df_column_len / samples_count;
                    let mut correct_sample_ids: Vec<AnyValue> = Vec::new();
                    for i in 0..samples_count {
                        for j in 0..true_df_len {
                            correct_sample_ids.push(AnyValue::UInt64(i as u64));
                        }
                    }

                    self.cached_sample_size = samples_vec.len();
                    self.cached_sample_id_vectors.insert(
                        df_name.clone(), 
                        correct_sample_ids.iter().map(|vec_value| {
                            match vec_value {
                                AnyValue::UInt64(i) => i.clone() as u64,
                                _ => panic!("Broken type"),
                                
                            }
                        }).collect());

                    group_data_map.get_mut(&df_name).unwrap().insert("sample_id".to_string(), correct_sample_ids);
                }
            } else {
                for df_name in self.cached_sample_id_vectors.keys() {
                    group_data_map.get_mut(df_name).unwrap().insert(
                        "sample_id".to_string(), 
                        self.cached_sample_id_vectors.get(df_name).unwrap().iter().map(|x| AnyValue::UInt64(*x)).collect()
                    );
                }
            }
            //println!("correct sample ids time: {}", chrono::Utc::now().timestamp_millis() - start_time );

            return group_data_map;

        }

        pub fn request_score<'a>(&mut self, samples: &Vec<Array1<f64>>) -> Vec<ScoreType>{

            //let start_time = chrono::Utc::now().timestamp_millis();
            let candidates:Vec<Vec<(AnyValue<'a>)>> = samples.iter().map(|x| self.variables_manager.inverse_transform_variables(&x)).collect();
            //println!("inverse transform time: {}", chrono::Utc::now().timestamp_millis() - start_time );
            //let start_time = chrono::Utc::now().timestamp_millis();
            let group_data_map = self.build_group_data_map(&candidates);
            //println!("build group data map time: {}", chrono::Utc::now().timestamp_millis() - start_time );
            let samples_count = candidates.len();
            //let start_time = chrono::Utc::now().timestamp_millis();
            self.update_dfs_for_scoring(group_data_map, samples_count);
            //println!("updatimg dfs time: {}", chrono::Utc::now().timestamp_millis() - start_time );

            //let start_time = chrono::Utc::now().timestamp_millis();
            let score_batch = &self.cotwin.get_score(&self.planning_entity_dfs, &self.problem_fact_dfs);
            let score_batch = score_batch.to_owned();
            //println!("query time: {}", chrono::Utc::now().timestamp_millis() - start_time );

            return score_batch;
        }
    }

unsafe impl<EntityVariants, UtilityObjectVariants, ScoreType> Send for OOPScoreRequester<EntityVariants, UtilityObjectVariants, ScoreType>
where 
    ScoreType: ScoreTrait + Clone + AddAssign + Send,
    EntityVariants: CotwinEntityTrait {}