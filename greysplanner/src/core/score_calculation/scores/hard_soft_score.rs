

use crate::core::score_calculation::scores::score_trait::ScoreTrait;
use std::cmp::Ordering;
use std::cmp::Ordering::*;
use std::ops::{Add, AddAssign};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct HardSoftScore {
    hard_score: f64,
    soft_score: f64
}

impl HardSoftScore {
    pub fn new(hard_score: f64, soft_score: f64) -> Self{
        HardSoftScore{
            hard_score: hard_score,
            soft_score: soft_score
        }
    }
}

impl ScoreTrait for HardSoftScore {
    fn get_sum_abs(&self) -> f64 {
        self.hard_score.abs() + self.soft_score.abs()
    }

    fn get_priority_score(&self) -> f64 {
        if self.hard_score > 0.0 {
            return self.hard_score;
        } else {
            return self.soft_score;
        }
    }

    fn get_fitness_value(&self) -> f64 {
        let hard_fitness = 1.0 - (1.0 / (self.hard_score + 1.0));
        let soft_fitness = 1.0 - (1.0 / (self.soft_score + 1.0));
        let fitness_value = 0.5 * hard_fitness + 0.5 * soft_fitness;
        
        return fitness_value;
    }

    fn get_null_score() -> Self {
        HardSoftScore {
            hard_score: 0.0,
            soft_score: 0.0
        }
    }

    fn mul(&self, scalar: f64) -> Self {
        HardSoftScore {
            hard_score: scalar * self.hard_score,
            soft_score: scalar * self.soft_score
        }
    }
}

impl Eq for HardSoftScore {}

impl Ord for HardSoftScore {

    fn cmp(&self, other: &Self) -> Ordering {
        let hard_score_ordering = self.hard_score.total_cmp(&other.hard_score);

        match hard_score_ordering {
            Less => return hard_score_ordering,
            Greater => return hard_score_ordering,
            Equal => return self.soft_score.total_cmp(&other.soft_score)
        }
    }
    
}

impl Add for HardSoftScore {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        HardSoftScore {
            hard_score: self.hard_score + rhs.hard_score,
            soft_score: self.soft_score + rhs.soft_score,
        }
    }
}

impl AddAssign for HardSoftScore {
    fn add_assign(&mut self, rhs: Self) {
        self.hard_score += rhs.hard_score;
        self.soft_score += rhs.soft_score;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hard_soft_score_impl() {
        let score = HardSoftScore::new(-1.0, -1.0);
        assert_eq!(score.get_sum_abs(), 2.0);

        let score = HardSoftScore::new(0.0, 9.0);
        assert_eq!(score.get_priority_score(), 9.0);
        assert_eq!(score.get_fitness_value(), 0.45);
    }

    #[test]
    fn test_hard_soft_score_comparison() {

        let small_score = HardSoftScore::new(-1.0, -1.0);
        let null_score = HardSoftScore::new(0.0, 0.0);
        let large_score = HardSoftScore::new(0.0, 0.1);

        assert_eq!(small_score < large_score, true);
        assert_eq!(small_score <= large_score, true);
        assert_eq!(small_score != large_score, true);
        assert_eq!(null_score == null_score, true);
        assert_eq!(large_score > null_score, true);
        assert_eq!(large_score >= large_score, true);
        
        let mut scores_vec_1: Vec<HardSoftScore> = Vec::new();
        for i in 0..10 {
            scores_vec_1.push(HardSoftScore::new(i as f64, (2 * i) as f64));
        }
        let scores_vec_2 = scores_vec_1.clone();
        scores_vec_1.reverse();
        scores_vec_1.sort();
        assert_eq!(scores_vec_1, scores_vec_2);

        let mut scores_vec_1: Vec<HardSoftScore> = Vec::new();
        for i in 0..10 {
            scores_vec_1.push(HardSoftScore::new(0 as f64, i as f64));
        }
        let scores_vec_2 = scores_vec_1.clone();
        scores_vec_1.reverse();
        scores_vec_1.sort();
        assert_eq!(scores_vec_1, scores_vec_2);
        
    }

    #[test]
    fn test_simple_score_add() {
        let mut score_1 = HardSoftScore::new(-1.0, -1.0);
        let score_2 = HardSoftScore::new(1.0, 1.0);
        let score_3 = HardSoftScore::new(0.0, 0.0);
        assert_eq!(score_1.clone() + score_2.clone(), score_3);

        score_1 += score_2.clone();
        assert_eq!(score_1, score_3);
    }
}