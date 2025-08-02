
use std::fmt::Debug;
use std::ops::{Add, AddAssign};
use crate::utils::math_utils::round;
use serde::{Serialize, Deserialize};
use std::cmp::Ordering;
use std::cmp::Ordering::*;
use std::fmt::{Display, Formatter};
use super::score_trait::*;


/// HardMediumSoftScore with structured accumulator
#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct HardMediumSoftScore {
    pub hard_score: f64,
    pub medium_score: f64,
    pub soft_score: f64,
}

#[derive(Default, Clone, Debug)]
pub struct HardMediumSoftAccumulator {
    pub hard_total: f64,
    pub medium_total: f64,
    pub soft_total: f64,
}

impl HardMediumSoftScore {
    pub fn new(hard_score: f64, medium_score: f64, soft_score: f64) -> Self {
        Self { hard_score, medium_score, soft_score }
    }

    pub fn hard(hard_score: f64) -> Self {
        Self { hard_score, medium_score: 0.0, soft_score: 0.0 }
    }

    pub fn medium(medium_score: f64) -> Self {
        Self { hard_score: 0.0, medium_score, soft_score: 0.0 }
    }

    pub fn soft(soft_score: f64) -> Self {
        Self { hard_score: 0.0, medium_score: 0.0, soft_score }
    }
}

impl Eq for HardMediumSoftScore {}

impl Ord for HardMediumSoftScore {

    fn cmp(&self, other: &Self) -> Ordering {

        let hard_score_ordering = self.hard_score.total_cmp(&other.hard_score);
        match hard_score_ordering {
            Less => return hard_score_ordering,
            Greater => return hard_score_ordering,
            Equal => {

                let medium_score_ordering = self.medium_score.total_cmp(&other.medium_score);
                match medium_score_ordering {
                    Less => return medium_score_ordering,
                    Greater => return medium_score_ordering,
                    Equal => self.soft_score.total_cmp(&other.soft_score)
                }
            }
        }
    }
    
}

impl Add for HardMediumSoftScore {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        HardMediumSoftScore {
            hard_score: self.hard_score + other.hard_score,
            medium_score: self.medium_score + other.medium_score,
            soft_score: self.soft_score + other.soft_score,
        }
    }
}

impl AddAssign for HardMediumSoftScore {
    fn add_assign(&mut self, rhs: Self) {
        self.hard_score += rhs.hard_score;
        self.medium_score += rhs.medium_score;
        self.soft_score += rhs.soft_score;
    }
}

impl Display for HardMediumSoftScore {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} | {} | {}", self.hard_score, self.medium_score, self.soft_score)
    }
    
}

unsafe impl Send for HardMediumSoftScore {}

impl ScoreTrait for HardMediumSoftScore {
    type Accumulator = HardMediumSoftAccumulator;

    fn get_null_score() -> Self {
        HardMediumSoftScore {
            hard_score: 0.0,
            medium_score: 0.0,
            soft_score: 0.0,
        }
    }

    fn get_fields() -> &'static [&'static str] {
        &["hard_score", "medium_score", "soft_score"]
    }

    fn as_vec(&self) -> Vec<f64> {
        vec![self.hard_score, self.medium_score, self.soft_score]
    }

    fn from_vec(values: Vec<f64>) -> Self {
        HardMediumSoftScore {
            hard_score: values.get(0).copied().unwrap_or(0.0),
            medium_score: values.get(1).copied().unwrap_or(0.0),
            soft_score: values.get(2).copied().unwrap_or(0.0),
        }
    }

    fn get_sum_abs(&self) -> f64 {
        self.hard_score.abs() + self.medium_score.abs() + self.soft_score.abs()
    }

    fn get_priority_score(&self) -> f64 {
        if self.hard_score != 0.0 {
            self.hard_score
        } else if self.medium_score != 0.0 {
            self.medium_score
        } else {
            self.soft_score
        }
    }

    fn mul(&self, scalar: f64) -> Self {
        HardMediumSoftScore {
            hard_score: self.hard_score * scalar,
            medium_score: self.medium_score * scalar,
            soft_score: self.soft_score * scalar,
        }
    }

    fn get_fitness_value(&self) -> f64 {
        let hard_fitness = 1.0 - (1.0 / (self.hard_score + 1.0));
        let medium_fitness = 1.0 - (1.0 / (self.medium_score + 1.0));
        let soft_fitness = 1.0 - (1.0 / (self.soft_score + 1.0));
        const WEIGHT: f64 = 1.0 / 3.0;
        WEIGHT * hard_fitness + WEIGHT * medium_fitness + WEIGHT * soft_fitness
    }

    fn get_stub_score() -> Self {
        HardMediumSoftScore {
            hard_score: f64::INFINITY,
            medium_score: f64::INFINITY,
            soft_score: f64::INFINITY,
        }
    }

    fn precision_len() -> usize {
        return 3;
    }

    fn round(&mut self, precision: &Vec<u64>) {
        self.hard_score = round(self.hard_score, precision[0]);
        self.medium_score = round(self.medium_score, precision[1]);
        self.soft_score = round(self.soft_score, precision[2]);
    }

    // === ACCUMULATION METHODS ===
    
    #[inline]
    fn accumulate_into(accumulator: &mut Self::Accumulator, score: &Self) {
        accumulator.hard_total += score.hard_score;
        accumulator.medium_total += score.medium_score;
        accumulator.soft_total += score.soft_score;
    }
    
    #[inline]
    fn from_accumulator(accumulator: &Self::Accumulator) -> Self {
        HardMediumSoftScore {
            hard_score: accumulator.hard_total,
            medium_score: accumulator.medium_total,
            soft_score: accumulator.soft_total,
        }
    }
    
    #[inline]
    fn reset_accumulator(accumulator: &mut Self::Accumulator) {
        accumulator.hard_total = 0.0;
        accumulator.medium_total = 0.0;
        accumulator.soft_total = 0.0;
    }
}

impl FromHard for HardMediumSoftScore {
    fn hard(value: f64) -> Self {
        Self::hard(value)
    }
}

impl FromMedium for HardMediumSoftScore {
    fn medium(value: f64) -> Self {
        Self::medium(value)
    }
}

impl FromSoft for HardMediumSoftScore {
    fn soft(value: f64) -> Self {
        Self::soft(value)
    }
}