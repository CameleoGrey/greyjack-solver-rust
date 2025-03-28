

use greyjack::cotwin::Cotwin;
use greyjack::cotwin::CotwinBuilderTrait;
use greyjack::cotwin::CotwinValueTypes;
use greyjack::cotwin::CotwinEntityTrait;
use greyjack::score_calculation::scores::SimpleScore;
use greyjack::score_calculation::score_calculators::score_calculator_variants::ScoreCalculatorVariants;
use greyjack::variables::GJInteger;
use crate::cotwin::CotQueen;
use crate::score::NQueensIncrementalScoreCalculator;
use crate::score::NQueensPlainScoreCalculator;
use crate::domain::ChessBoard;
use polars::datatypes::AnyValue;
use std::collections::HashMap;


pub enum EntityVariants<'a> {
    CotQueen(CotQueen<'a>)
}

impl<'a> CotwinEntityTrait for EntityVariants<'a> {
    fn to_vec(&self) -> Vec<(String, CotwinValueTypes)> {
        match self {
            EntityVariants::CotQueen(x) => return x.to_vec()
        }
    }
}

pub enum UtilityObjectVariants {}

#[derive(Clone)]
pub struct CotwinBuilder {
    use_incremental_score_calculation: bool,
}

impl CotwinBuilder {
    pub fn new(use_incremental_score_calculation: bool) -> Self {
        Self {
            use_incremental_score_calculation: use_incremental_score_calculation,
        }
    }
}

impl<'a> CotwinBuilderTrait<ChessBoard, EntityVariants<'a>, UtilityObjectVariants, SimpleScore> for CotwinBuilder
 {

    fn build_cotwin(&self, domain_model: ChessBoard, is_already_initialized: bool) -> Cotwin<EntityVariants<'a>, UtilityObjectVariants, SimpleScore> {

        let n = domain_model.n;
        let queens = &domain_model.queens;
        let mut cot_queens: Vec<EntityVariants> = Vec::new();

        if is_already_initialized {
            panic!("Building cotwin for existing domain isn't already implemented for NQueens problem")
        }

        for i in 0..n {
            let queen_id  = CotwinValueTypes::PAV(AnyValue::UInt64(i));
            let column_id = CotwinValueTypes::PAV(AnyValue::UInt64(i));
            // initial_value: Some(queens[i as usize].row.row_id as i64)
            let planning_row_id = CotwinValueTypes::GJI(
                GJInteger::new( Some(queens[i as usize].row.row_id as i64), 
                0, (n-1) as i64, false, None)
            );

            let cot_queen = CotQueen {
                queen_id: queen_id,
                row_id: planning_row_id,
                column_id: column_id,

            };
            let cot_queen = EntityVariants::CotQueen(cot_queen);
            cot_queens.push(cot_queen);
        }

        let mut nqueens_cotwin = Cotwin::new();
        nqueens_cotwin.add_planning_entities("queens".to_string(), cot_queens);

        if self.use_incremental_score_calculation {
            let score_calculator = NQueensIncrementalScoreCalculator::new();
            nqueens_cotwin.add_score_calculator(ScoreCalculatorVariants::ISC(score_calculator));
        } else {
            let score_calculator = NQueensPlainScoreCalculator::new();
            nqueens_cotwin.add_score_calculator(ScoreCalculatorVariants::PSC(score_calculator));
        }

        return nqueens_cotwin;
    }
}

impl CotwinBuilder {
}

unsafe impl Send for CotwinBuilder {}