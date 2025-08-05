use crate::cotwin::cot_queen::CotQueen;
use crate::cotwin::greynet_queen::GreynetQueen;
use crate::domain::ChessBoard;
use crate::score::{
    NQueensGreynetScoreCalculator, NQueensIncrementalScoreCalculator, NQueensPlainScoreCalculator,
};
use greyjack::cotwin::{Cotwin, CotwinBuilderTrait, CotwinEntityTrait, CotwinValueTypes};
use greyjack::score_calculation::greynet::greynet_traits::{InitializableFact, ModifiableFact};
use greyjack::score_calculation::score_calculators::score_calculator_variants::ScoreCalculatorVariants;
use greyjack::score_calculation::scores::SimpleScore;
use greyjack::variables::GJInteger;
use polars::datatypes::AnyValue;
use std::rc::Rc;

#[derive(Clone)]
pub enum EntityVariants<'a> {
    CotQueen(CotQueen<'a>),
}

impl<'a> CotwinEntityTrait for EntityVariants<'a> {
    fn to_vec(&self) -> Vec<(String, CotwinValueTypes)> {
        match self {
            EntityVariants::CotQueen(x) => x.to_vec(),
        }
    }
}

impl<'a> InitializableFact for EntityVariants<'a> {
    fn to_initialized_fact(&self) -> Rc<dyn ModifiableFact + Send> {
        match self {
            EntityVariants::CotQueen(cq) => {
                let queen_id = match cq.queen_id {
                    CotwinValueTypes::PAV(AnyValue::UInt64(v)) => v as i64,
                    _ => panic!("Invalid queen_id type"),
                };
                let row_id = match &cq.row_id {
                    CotwinValueTypes::GJI(gji) => gji.initial_value.unwrap() as i64,
                    _ => panic!("Invalid row_id type"),
                };
                let column_id = match cq.column_id {
                    CotwinValueTypes::PAV(AnyValue::UInt64(v)) => v as i64,
                    _ => panic!("Invalid column_id type"),
                };

                Rc::new(GreynetQueen {
                    queen_id,
                    row_id,
                    column_id,
                })
            }
        }
    }
}

pub enum UtilityObjectVariants {}

#[derive(Clone, Copy)]
pub enum CalculatorType {
    Plain,
    Incremental,
    Greynet,
}

#[derive(Clone)]
pub struct CotwinBuilder {
    calculator_type: CalculatorType,
}

impl CotwinBuilder {
    pub fn new(calculator_type: CalculatorType) -> Self {
        Self { calculator_type }
    }
}

impl<'a> CotwinBuilderTrait<ChessBoard, EntityVariants<'a>, UtilityObjectVariants, SimpleScore>
    for CotwinBuilder
{
    fn build_cotwin(
        &self,
        domain_model: ChessBoard,
        is_already_initialized: bool,
    ) -> Cotwin<EntityVariants<'a>, UtilityObjectVariants, SimpleScore> {
        if is_already_initialized {
            panic!("Building cotwin for existing domain is not implemented for NQueens");
        }

        let n = domain_model.n;
        let queens = &domain_model.queens;
        let mut cot_queens: Vec<EntityVariants> = Vec::with_capacity(n as usize);

        for i in 0..n {
            let cot_queen = CotQueen {
                queen_id: CotwinValueTypes::PAV(AnyValue::UInt64(i)),
                column_id: CotwinValueTypes::PAV(AnyValue::UInt64(i)),
                row_id: CotwinValueTypes::GJI(GJInteger::new(
                    Some(queens[i as usize].row.row_id as i64),
                    0,
                    (n - 1) as i64,
                    false,
                    None,
                )),
            };
            cot_queens.push(EntityVariants::CotQueen(cot_queen));
        }

        let mut nqueens_cotwin = Cotwin::new();
        nqueens_cotwin.add_planning_entities("queens".to_string(), cot_queens);

        let calculator = match self.calculator_type {
            CalculatorType::Incremental => ScoreCalculatorVariants::ISC(NQueensIncrementalScoreCalculator::new()),
            CalculatorType::Plain => ScoreCalculatorVariants::PSC(NQueensPlainScoreCalculator::new()),
            CalculatorType::Greynet => ScoreCalculatorVariants::Greynet(NQueensGreynetScoreCalculator::new()),
        };
        nqueens_cotwin.add_score_calculator(calculator);

        nqueens_cotwin
    }
}

unsafe impl Send for CotwinBuilder {}