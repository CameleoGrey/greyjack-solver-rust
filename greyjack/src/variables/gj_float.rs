
use std::cmp::Ordering::*;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand_distr::{Normal, Distribution, Uniform};

#[derive(Debug, Clone)]
pub struct GJFloat {
    pub name: String,
    pub initial_value: Option<f64>,
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub frozen: bool,
    pub random_generator: StdRng,
    pub uniform_distribution: Uniform<f64>,
    pub normal_distribution: Option<Normal<f64>>,
    pub semantic_groups: Vec<String>
}

impl GJFloat {
    pub fn new(name: &str, initial_value: Option<f64>, 
        lower_bound: f64, upper_bound: f64, frozen: bool, semantic_groups: Option<Vec<String>>) -> Self {
            
            let normal_distribution;
            match initial_value {
                None => normal_distribution = None,
                Some(x) => normal_distribution = Some(Normal::new(x, 0.1).unwrap())
            };

            let mut current_semantic_groups: Vec<String> = Vec::new();
            match semantic_groups {
                None => current_semantic_groups.push("common".to_string()),
                Some(groups) => {
                    for group in groups {
                        current_semantic_groups.push(group);
                    }
                },
            }

            GJFloat {
                name: name.to_string(),
                initial_value: initial_value,
                lower_bound: lower_bound,
                upper_bound: upper_bound,
                frozen: frozen,
                random_generator: StdRng::from_entropy(),
                uniform_distribution: Uniform::new_inclusive(lower_bound, upper_bound),
                normal_distribution: normal_distribution,
                semantic_groups: current_semantic_groups
            }
        }
}

impl GJFloat {

    pub fn set_name(&mut self, new_name: String) {
        self.name = new_name;
    }

    pub fn inverse_transform(&self, value: f64) -> f64 {
        return self.fix(value);
    }

    pub fn fix(&self, value: f64) -> f64 {

        if self.frozen {
            match self.initial_value {
                Some(x) => return x,
                None => panic!("Frozen value must be initialized")
            }
        }
        
        let fixed_value = Self::min(Self::max(value, self.lower_bound), self.upper_bound);

        return fixed_value;
    }

    pub fn sample(&mut self) -> f64 {

        if self.frozen {
            match self.initial_value {
                Some(x) => return x,
                None => panic!("Frozen value must be initialized")
            }
        }

        let sampled_value: f64 = self.uniform_distribution.sample( &mut self.random_generator);
        return sampled_value;
    }

    pub fn get_initial_value(&mut self) -> f64 {

        match self.initial_value {
            None => return self.sample(),
            Some(x) => {
                let mut initial_value = x;
                if self.frozen {
                    return initial_value;
                }
                
                // needful for LSHADE in case of initialized variables for the whole population
                // (it needs to choose vectors from history archive / population, that are different by at least one component).
                // LSHADE will be added in later versions for tasks, containing many floats
                match self.normal_distribution {
                   Some(gauss) => {
                    initial_value = Normal::new(initial_value, 0.1).unwrap().sample(&mut self.random_generator);
                    initial_value = self.fix(initial_value);
                   },
                   None => ()
                }

                return initial_value;
            }

        }



    }

    pub fn min(a: f64, b: f64) -> f64 {

        let min_value;
        match a.total_cmp(&b) {
            Less => min_value = a,
            Greater => min_value = b,
            Equal => min_value = a
        }
        min_value
    }

    pub fn max(a: f64, b: f64) -> f64 {

        let max_value;
        match a.total_cmp(&b) {
            Less => max_value = b,
            Greater => max_value = a,
            Equal => max_value = b
        }
        max_value
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    
    /*use polars::frame::row::Row;
    use polars::prelude::*;
    use polars::datatypes::{AnyValue, DataType};

    #[test]
    fn test_polars_init() {
        /*let mut schema = Schema::default();
        schema.insert(PlSmallStr::from_static("float_col"), DataType::Float64);
        schema.insert(PlSmallStr::from_static("int_col"), DataType::Int64);
        let frame_1 = DataFrame::empty_with_schema(&schema);

        let rows: Vec<Row> = Vec::new();
        let row_values: Vec<AnyValue> = Vec::new();
        row_values.push(AnyValue::Float64(1.0));
        row_values.push(AnyValue::Int64(2));
        rows.push(Row::new(row_values));

        let frame_1 = DataFrame::from_iter(rows.into());*/

        let mut frame_data: Vec<Column> = Vec::new();
        let mut float_values: Vec<AnyValue> = Vec::new();
        float_values.push(AnyValue::Float64(0.0));
        float_values.push(AnyValue::Null);
        frame_data.push(Column::new("floats".into(), float_values));
        
        let frame_1 = DataFrame::new(frame_data).expect("Broken column data");
        println!("{}", frame_1);

    }*/

    #[test]
    fn test_gp_float_var_frozen() {
        let mut x = GJFloat::new("x", Some(1.0), -1.0, 1.0, true, None);
        
        let initial_value = x.get_initial_value();
        assert_eq!(initial_value, 1.0);
    }

    #[test]
    fn test_gp_float_var_unfrozen() {
        let mut x = GJFloat::new("x", Some(1000.0), -10000.0, 10000.0, false, None);
        
        let initial_value = x.get_initial_value();
        assert_ne!(initial_value, 1000.0);
    }

    #[test]
    fn test_gp_float_var_fix_value() {
        let mut x = GJFloat::new("x", Some(1.0), -1.0, 1.0, false, None);
        
        let too_little_value: f64 = -100.0;
        let fixed_value = x.fix(too_little_value);
        assert_eq!(fixed_value, -1.0);

        let too_big_value: f64 = 100.0;
        let fixed_value = x.fix(too_big_value);
        assert_eq!(fixed_value, 1.0);
    }
}