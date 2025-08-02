use std::fmt::Debug;
use std::ops::Add;

/// Core trait for all score types with mandatory accumulation support
pub trait ScoreTrait: Clone + Add<Output = Self> + PartialOrd + Debug + 'static {
    /// Accumulator type for efficient score building
    type Accumulator: Default + Clone + Debug;
    
    /// Returns a "null" or zero score (the additive identity)
    fn get_null_score() -> Self;

    /// Returns the names of score fields for this score type
    fn get_fields() -> &'static [&'static str];

    /// Converts the score to a list of numeric values
    fn as_vec(&self) -> Vec<f64>;

    /// Creates a score from a list of numeric values
    fn from_vec(values: Vec<f64>) -> Self;

    /// Returns the sum of absolute values of all score components
    fn get_sum_abs(&self) -> f64;

    /// Returns the priority score (typically the most important component)
    fn get_priority_score(&self) -> f64;

    /// Multiplies the score by a scalar value
    fn mul(&self, scalar: f64) -> Self;
    
    /// Calculates a fitness value, often used in genetic algorithms
    fn get_fitness_value(&self) -> f64;

    /// Returns a "stub" or placeholder score, usually representing a very high penalty
    fn get_stub_score() -> Self;

    fn precision_len() -> usize;
    
    /// Rounds the score's components to a given number of decimal places
    fn round(&mut self, precision: &Vec<u64>);
    
    // === NEW ACCUMULATION METHODS ===
    
    /// Add a score to the accumulator
    fn accumulate_into(accumulator: &mut Self::Accumulator, score: &Self);
    
    /// Build final score from accumulator
    fn from_accumulator(accumulator: &Self::Accumulator) -> Self;
    
    /// Reset the accumulator to zero state
    fn reset_accumulator(accumulator: &mut Self::Accumulator);
    
    /// Create a new accumulator with this score as initial value
    fn into_accumulator(&self) -> Self::Accumulator {
        let mut acc = Self::Accumulator::default();
        Self::accumulate_into(&mut acc, self);
        acc
    }
}

// Keep the trait implementations for ergonomic construction
pub trait FromSimple: Sized {
    fn simple(value: f64) -> Self;
}

pub trait FromHard: Sized {
    fn hard(value: f64) -> Self;
}

pub trait FromMedium: Sized {
    fn medium(value: f64) -> Self;
}

pub trait FromSoft: Sized {
    fn soft(value: f64) -> Self;
}