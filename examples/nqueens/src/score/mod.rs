

pub mod plain_score_calculator;
pub mod incremental_score_calculator;
pub mod greynet_score_calculator;

pub use plain_score_calculator::NQueensPlainScoreCalculator;
pub use incremental_score_calculator::NQueensIncrementalScoreCalculator;
pub use greynet_score_calculator::NQueensGreynetScoreCalculator;