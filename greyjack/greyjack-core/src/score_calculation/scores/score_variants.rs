
use super::score_trait::ScoreTrait;
use super::{SimpleScore, HardSoftScore, HardMediumSoftScore};

pub enum ScoreVariants {
    SS(SimpleScore),
    HS(HardSoftScore),
    HMS(HardMediumSoftScore),
}
