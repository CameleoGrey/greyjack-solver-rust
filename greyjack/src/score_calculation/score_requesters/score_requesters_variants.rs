
use super::GreynetScoreRequester;
use super::DataframeScoreRequester;
use crate::score_calculation::greynet::greynet_traits::InitializableFact;
use crate::score_calculation::scores::ScoreTrait;
use crate::cotwin::CotwinEntityTrait;
use std::ops::AddAssign;

pub enum ScoreRequestersVariants<EntityVariants, UtilityObjectVariants, ScoreType>
where
    ScoreType: ScoreTrait + Clone + Send + AddAssign + 'static,
    EntityVariants: InitializableFact + CotwinEntityTrait + Clone + Send + 'static,
{
    Greynet(GreynetScoreRequester<EntityVariants, UtilityObjectVariants, ScoreType>),
    Dataframe(DataframeScoreRequester<EntityVariants, UtilityObjectVariants, ScoreType>)
}