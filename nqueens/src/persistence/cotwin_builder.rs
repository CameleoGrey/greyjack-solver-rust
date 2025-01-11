

use greysplanner::api::oop::cotwin::Cotwin;
use greysplanner::api::oop::cotwin_value_types::CotwinValueTypes;
use greysplanner::api::oop::cotwin_entity_trait::CotwinEntityTrait;
use greysplanner::core::score_calculation::scores::simple_score::SimpleScore;
use greysplanner::core::variables::gp_integer_var::GPIntegerVar;
use crate::cotwin::cot_queen::CotQueen;
use crate::score::nqueens_score_calculator::NQueensScoreCalculator;
use crate::domain::chess_board::ChessBoard;
use polars::datatypes::AnyValue;
use std::collections::HashMap;

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

pub struct CotwinBuilder {

}

impl CotwinBuilder {

    pub fn build_cotwin<'a>(domain_model: &ChessBoard) -> Cotwin<EntityVariants<'a>, UtilityObjectVariants, SimpleScore> {

        let n = domain_model.n;
        let queens = &domain_model.queens;
        let mut cot_queens: Vec<EntityVariants> = Vec::new();

        for i in 0..n {
            let queen_id  = CotwinValueTypes::PolarsAnyValue(AnyValue::UInt64(i));
            let column_id = CotwinValueTypes::PolarsAnyValue(AnyValue::UInt64(i));

            let planning_row_id = CotwinValueTypes::GPIntegerVar(
                GPIntegerVar::new(&format!("queen_{}_row_id", i), 
                Some(i as i64), 
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