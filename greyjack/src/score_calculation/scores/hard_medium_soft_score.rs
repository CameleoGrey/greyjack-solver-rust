

use crate::score_calculation::scores::ScoreTrait;
use crate::utils::math_utils::round;
use std::cmp::Ordering;
use std::cmp::Ordering::*;
use std::ops::{Add, AddAssign};
use std::fmt::{Display, Formatter};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct HardMediumSoftScore {
    pub hard_score: f64,
    pub medium_score: f64,
    pub soft_score: f64
}

impl HardMediumSoftScore {
    pub fn new(hard_score: f64, medium_score: f64, soft_score: f64) -> Self{
        HardMediumSoftScore{
            hard_score: hard_score,
            medium_score: medium_score,
            soft_score: soft_score
        }
    }
}

impl ScoreTrait for HardMediumSoftScore {
    fn get_sum_abs(&self) -> f64 {
        self.hard_score.abs() + self.medium_score.abs() + self.soft_score.abs()
    }

    fn get_priority_score(&self) -> f64 {
        if self.hard_score > 0.0 {
            return self.hard_score;
        } else if self.medium_score > 0.0{
            return self.medium_score;
        } else {
            return self.soft_score;
        }
    }

    fn get_fitness_value(&self) -> f64 {
        let hard_fitness = 1.0 - (1.0 / (self.hard_score + 1.0));
        let medium_fitness = 1.0 - (1.0 / (self.medium_score + 1.0));
        let soft_fitness = 1.0 - (1.0 / (self.soft_score + 1.0));
        let fitness_value = 0.66 * hard_fitness + 0.25 * medium_fitness + 0.9 * soft_fitness;
        
        return fitness_value;
    }

    fn get_null_score() -> Self {
        HardMediumSoftScore {
            hard_score: 0.0,
            medium_score: 0.0,
            soft_score: 0.0
        }
    }

    fn get_stub_score() -> Self {
        HardMediumSoftScore {
            hard_score: f64::MAX - 1.0,
            medium_score: f64::MAX - 1.0,
            soft_score: f64::MAX - 1.0
        }
    }

    fn as_vec(&self) -> Vec<f64> {
        vec![self.hard_score, self.medium_score, self.soft_score]
    }

    fn mul(&self, scalar: f64) -> Self {
        HardMediumSoftScore {
            hard_score: scalar * self.hard_score,
            medium_score: scalar * self.medium_score,
            soft_score: scalar * self.soft_score
        }
    }

    fn precision_len() -> usize {
        3
    }

    fn round(&mut self, precision: &Vec<u64>) {
        self.hard_score = round(self.hard_score, precision[0]);
        self.medium_score = round(self.medium_score, precision[1]);
        self.soft_score = round(self.soft_score, precision[2]);
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

    fn add(self, rhs: Self) -> Self {
        HardMediumSoftScore {
            hard_score: self.hard_score + rhs.hard_score,
            medium_score: self.medium_score + rhs.medium_score,
            soft_score: self.soft_score + rhs.soft_score,
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