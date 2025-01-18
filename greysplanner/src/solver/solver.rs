

use std::collections::HashMap;
use std::ops::AddAssign;
use std::fmt::Debug;

use crate::agents::base::{Agent, AgentStatuses, AgentToSolverUpdate, AgentToAgentUpdate};
use crate::agents::base::AgentStatuses::*;
use crate::cotwin::CotwinEntityTrait;
use crate::score_calculation::scores::ScoreTrait;
use super::{ObservableTrait, ObserverTrait};
use crossbeam_channel::*;
use threadpool::*;
use std::sync::{RwLock, Arc, Mutex};

pub struct Solver {}

impl Solver {

    /*pub fn new(
        agents: Vec<Agent<'b, EntityVariants, UtilityObjectVariants, ScoreType>>,
        score_precision: Vec<u64>
    ) -> Self {

        if score_precision.len() != ScoreType::precision_len() {
            panic!("Invalid score_precision. Suggest: vec![a] for SimpleScore, vec![a, b] for HardSoft, vec![a, b, c] for HardMediumSoft.");
        }

        Self {
            agents: agents,
            score_precision: score_precision,
            round_robin_status_vec: Vec::new(),
            observers: Vec::new()
        }
    }*/

    pub fn solve<EntityVariants, UtilityObjectVariants, ScoreType> (
        mut agents: Vec<Agent<EntityVariants, UtilityObjectVariants, ScoreType>>,
        score_precision: Vec<u64>
    )
    where
    EntityVariants: CotwinEntityTrait + 'static,
    UtilityObjectVariants: 'static,
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send + 'static{

        if score_precision.len() != ScoreType::precision_len() {
            panic!("Invalid score_precision. Suggest: vec![a] for SimpleScore, vec![a, b] for HardSoft, vec![a, b, c] for HardMediumSoft.");
        }
        //self.setup_agents();

        let n_agents = agents.len();
        let mut round_robin_status_vec: Vec<AgentStatuses> = Vec::new();
        let mut agents_updates_senders: Vec<Sender<AgentToAgentUpdate>> = Vec::new();
        let mut agents_updates_receivers: Vec<Receiver<AgentToAgentUpdate>> = Vec::new();
        for i in 0..n_agents {
            agents[i].agent_id = i;
            round_robin_status_vec.insert(i, AgentStatuses::Alive);
            let (agent_i_updates_sender, agent_i_updates_receiver): (Sender<AgentToAgentUpdate>, Receiver<AgentToAgentUpdate>) = bounded(1);
            agents_updates_senders.push(agent_i_updates_sender);
            agents_updates_receivers.push(agent_i_updates_receiver);
        }
        for i in 0..n_agents {
            let next_agent_id = (i+1) % n_agents;
            agents[i].round_robin_status_vec = round_robin_status_vec.clone();
            agents[i].updates_to_agent_sender = Some(agents_updates_senders[i].clone());
            agents[i].updates_for_agent_receiver = Some(agents_updates_receivers[next_agent_id].clone());
        }

        let agents_pool = ThreadPool::new(n_agents);
        for mut agent in agents {
            agents_pool.execute( move || agent.solve());
        }

        agents_pool.join();

    }

    /*fn setup_agents(&mut self) {

        let mut agents_updates_senders: Vec<Sender<AgentToAgentUpdate>> = Vec::new();
        let mut agents_updates_receivers: Vec<Receiver<AgentToAgentUpdate>> = Vec::new();
        for i in 0..agents.len() {
            agents[i].agent_id = i;
            self.round_robin_status_vec.insert(i, AgentStatuses::Alive);
            let (agent_i_updates_sender, agent_i_updates_receiver): (Sender<AgentToAgentUpdate>, Receiver<AgentToAgentUpdate>) = bounded(1);
            agents_updates_senders.push(agent_i_updates_sender);
            agents_updates_receivers.push(agent_i_updates_receiver);
        }
        for i in 0..agents.len() {
            let next_agent_id = (i+1) % agents.len();
            agents[i].round_robin_status_vec = self.round_robin_status_vec.clone();
            agents[i].updates_to_agent_sender = Some(agents_updates_senders[i].clone());
            agents[i].updates_for_agent_receiver = Some(agents_updates_receivers[next_agent_id].clone());
        }

    }*/

}