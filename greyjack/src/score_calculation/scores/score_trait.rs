
pub trait ScoreTrait {
    fn get_sum_abs(&self) -> f64;

    fn get_priority_score(&self) -> f64;

    fn get_fitness_value(&self) -> f64;

    fn get_null_score() -> Self;

    fn get_stub_score() -> Self;

    fn as_vec(&self) -> Vec<f64>;

    fn mul(&self, scalar: f64) -> Self;

    #[inline]
    fn precision_len() -> usize;

    fn round(&mut self, precision: &Vec<u64>);
}