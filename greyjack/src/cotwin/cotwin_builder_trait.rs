

use crate::cotwin::Cotwin;
use crate::score_calculation::scores::ScoreTrait;
use std::ops::AddAssign;

pub trait CotwinBuilderTrait<DomainType, EntityVariants, UtilityObjectVariants, ScoreType>
where 
    ScoreType: ScoreTrait + Clone + AddAssign + Send {
    fn new(use_incremental_score_calculation: bool) -> Self;

    fn build_cotwin(&self, domain: DomainType) -> Cotwin<EntityVariants, UtilityObjectVariants, ScoreType>;
}