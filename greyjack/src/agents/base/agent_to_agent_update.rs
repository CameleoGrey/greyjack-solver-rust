
use super::{AgentStatuses, Individual};
use crate::score_calculation::scores::ScoreTrait;
use std::collections::HashMap;
use std::ops::{AddAssign, Sub};
use std::fmt::Debug;




pub struct AgentToAgentUpdate<ScoreType>
where
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send {

    pub agent_id: usize,
    pub migrants: Vec<Individual<ScoreType>>,
    pub round_robin_status_vec: Vec<AgentStatuses>

}

impl<ScoreType> AgentToAgentUpdate<ScoreType>
where
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send {
    
    pub fn new(
        agent_id: usize, 
        migrants: Vec<Individual<ScoreType>>, 
        round_robin_status_vec: Vec<AgentStatuses>)
        -> Self {

            Self {
                agent_id: agent_id,
                migrants: migrants,
                round_robin_status_vec,
            }

        }
}