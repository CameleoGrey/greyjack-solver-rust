
pub trait ScoreTrait {
    fn get_sum_abs(&self) -> f64;

    fn get_priority_score(&self) -> f64;

    fn get_fitness_value(&self) -> f64;

    fn get_null_score() -> Self;

    fn mul(&self, scalar: f64) -> Self;
}