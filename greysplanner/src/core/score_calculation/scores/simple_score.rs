

use crate::core::score_calculation::scores::score_trait::ScoreTrait;
use std::cmp::Ordering;
use std::ops::{Add, AddAssign};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct SimpleScore {
    score_value: f64
}

impl SimpleScore {
    pub fn new(score_value: f64) -> Self{
        SimpleScore{
            score_value: score_value
        }
    }
}

impl ScoreTrait for SimpleScore {

    fn get_sum_abs(&self) -> f64 {
        self.score_value.abs()
    }

    fn get_priority_score(&self) -> f64 {
        self.score_value
    }

    fn get_fitness_value(&self) -> f64 {
        1.0 - (1.0 / (self.score_value + 1.0))
    }

    fn get_null_score() -> Self {
        SimpleScore {
            score_value: 0.0
        }
    }

    fn get_stub_score() -> Self {
        SimpleScore {
            score_value: f64::MAX - 1.0
        }
    }

    fn mul(&self, scalar: f64) -> Self {
        SimpleScore {
            score_value: scalar * self.score_value,
        }
    }
}

impl Eq for SimpleScore {}

impl Ord for SimpleScore {

    fn cmp(&self, other: &Self) -> Ordering {
        self.score_value.total_cmp(&other.score_value)
    }
    
}

impl Add for SimpleScore {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        SimpleScore {
            score_value: self.score_value + rhs.score_value,
        }
    }
}

impl AddAssign for SimpleScore {
    fn add_assign(&mut self, rhs: Self) {
        self.score_value += rhs.score_value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_score_impl() {
        let score = SimpleScore::new(-1.0);
        assert_eq!(score.get_sum_abs(), 1.0);

        let score = SimpleScore::new(9.0);
        assert_eq!(score.get_priority_score(), 9.0);
        assert_eq!(score.get_fitness_value(), 0.9);
    }

    #[test]
    fn test_simple_score_comparison() {

        let small_score = SimpleScore::new(-10.0);
        let null_score = SimpleScore::new(0.0);
        let large_score = SimpleScore::new(10.0);

        assert_eq!(small_score < large_score, true);
        assert_eq!(small_score <= large_score, true);
        assert_eq!(small_score != large_score, true);
        assert_eq!(null_score == null_score, true);
        assert_eq!(large_score > null_score, true);
        assert_eq!(large_score >= large_score, true);
        
        let mut scores_vec_1: Vec<SimpleScore> = Vec::new();
        for i in 0..10 {
            scores_vec_1.push(SimpleScore::new(i as f64));
        }
        let scores_vec_2 = scores_vec_1.clone();
        scores_vec_1.reverse();
        scores_vec_1.sort();
        assert_eq!(scores_vec_1, scores_vec_2);
        
    }

    #[test]
    fn test_simple_score_add() {
        let mut score_1 = SimpleScore::new(-1.0);
        let score_2 = SimpleScore::new(1.0);
        let score_3 = SimpleScore::new(0.0);
        assert_eq!(score_1.clone() + score_2.clone(), score_3);

        score_1 += score_2.clone();
        assert_eq!(score_1, score_3);
    }
}