use std::fmt::Debug;
use std::ops::{Add, AddAssign};
use crate::utils::math_utils::round;
use serde::{Serialize, Deserialize};
use std::cmp::Ordering;
use std::cmp::Ordering::*;
use std::fmt::{Display, Formatter};
use super::score_trait::*;

/// SimpleScore with primitive accumulator
#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct SimpleScore {
    pub simple_value: f64,
}

impl SimpleScore {
    pub fn new(simple_value: f64) -> Self {
        Self { simple_value }
    }
}

impl Eq for SimpleScore {}

impl Ord for SimpleScore {

    fn cmp(&self, other: &Self) -> Ordering {
        self.simple_value.total_cmp(&other.simple_value)
    }
    
}

impl Add for SimpleScore {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        SimpleScore {
            simple_value: self.simple_value + other.simple_value,
        }
    }
}

impl AddAssign for SimpleScore {
    fn add_assign(&mut self, rhs: Self) {
        self.simple_value += rhs.simple_value;
    }
}

impl Display for SimpleScore {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.simple_value)
    }
    
}

unsafe impl Send for SimpleScore {}

impl ScoreTrait for SimpleScore {
    type Accumulator = f64;

    fn get_null_score() -> Self {
        SimpleScore { simple_value: 0.0 }
    }

    fn get_fields() -> &'static [&'static str] {
        &["simple_value"]
    }

    fn as_vec(&self) -> Vec<f64> {
        vec![self.simple_value]
    }

    fn from_vec(mut values: Vec<f64>) -> Self {
        SimpleScore {
            simple_value: values.pop().unwrap_or(0.0),
        }
    }

    fn get_sum_abs(&self) -> f64 {
        self.simple_value.abs()
    }

    fn get_priority_score(&self) -> f64 {
        self.simple_value
    }

    fn mul(&self, scalar: f64) -> Self {
        SimpleScore {
            simple_value: self.simple_value * scalar,
        }
    }

    fn get_fitness_value(&self) -> f64 {
        1.0 - (1.0 / (self.simple_value + 1.0))
    }

    fn get_stub_score() -> Self {
        SimpleScore { simple_value: f64::INFINITY }
    }

    fn precision_len() -> usize {
        return 1;
    }

    fn round(&mut self, precision: &Vec<u64>) {
        self.simple_value = round(self.simple_value, precision[0]);
    }

    // === ACCUMULATION METHODS ===
    
    #[inline]
    fn accumulate_into(accumulator: &mut Self::Accumulator, score: &Self) {
        *accumulator += score.simple_value;
    }
    
    #[inline]
    fn from_accumulator(accumulator: &Self::Accumulator) -> Self {
        SimpleScore { simple_value: *accumulator }
    }
    
    #[inline]
    fn reset_accumulator(accumulator: &mut Self::Accumulator) {
        *accumulator = 0.0;
    }
}

impl FromSimple for SimpleScore {
    fn simple(value: f64) -> Self {
        Self::new(value)
    }
}

impl From<f64> for SimpleScore {
    fn from(value: f64) -> Self {
        SimpleScore::new(value)
    }
}