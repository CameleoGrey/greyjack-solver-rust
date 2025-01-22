

use greysplanner::cotwin::Cotwin;
use greysplanner::cotwin::CotwinBuilderTrait;
use greysplanner::cotwin::CotwinValueTypes;
use greysplanner::cotwin::CotwinEntityTrait;
use greysplanner::score_calculation::scores::SimpleScore;
use greysplanner::variables::GPIntegerVar;
use crate::cotwin::CotQueen;
use crate::score::NQueensScoreCalculator;
use crate::domain::ChessBoard;
use polars::datatypes::AnyValue;
use std::collections::HashMap;
use greysplanner::score_calculation::scores::ScoreTrait;
use std::ops::AddAssign;


pub enum DomainVariants {
    CB(ChessBoard)
}

pub enum EntityVariants<'a> {
    CotQueen(CotQueen<'a>)
}

impl<'a> CotwinEntityTrait for EntityVariants<'a> {
    fn to_hash_map(&self) -> HashMap<String, CotwinValueTypes> {
        match self {
            EntityVariants::CotQueen(x) => return x.to_hash_map()
        }
    }
}

pub enum UtilityObjectVariants {}

#[derive(Clone)]
pub struct NQueensCotwinBuilder {

}

impl<'a> CotwinBuilderTrait<ChessBoard, EntityVariants<'a>, UtilityObjectVariants, SimpleScore> for NQueensCotwinBuilder
 {
    fn new() -> Self {
        Self{}
    }

    fn build_cotwin(&self, domain_model: ChessBoard) -> Cotwin<EntityVariants<'a>, UtilityObjectVariants, SimpleScore> {

        let n = domain_model.n;
        let queens = &domain_model.queens;
        let mut cot_queens: Vec<EntityVariants> = Vec::new();

        for i in 0..n {
            let queen_id  = CotwinValueTypes::PolarsAnyValue(AnyValue::UInt64(i));
            let column_id = CotwinValueTypes::PolarsAnyValue(AnyValue::UInt64(i));

            let planning_row_id = CotwinValueTypes::GPIntegerVar(
                GPIntegerVar::new(&format!("queen_{}_row_id", i), 
                Some(queens[i as usize].row.row_id as i64), 
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

        let score_calculator = NQueensScoreCalculator::new();
        nqueens_cotwin.add_score_calculator(score_calculator);

        return nqueens_cotwin;
    }
}

impl NQueensCotwinBuilder {
}

unsafe impl Send for NQueensCotwinBuilder {}