

use std::collections::HashMap;
use std::ops::AddAssign;
use std::fmt::Debug;

use crate::agents::base::{Agent, AgentStatuses, AgentToSolverUpdate, AgentToAgentUpdate};
use crate::agents::base::AgentStatuses::*;
use crate::agents::AgentBuildersVariants;
use crate::cotwin::{CotwinBuilderTrait, CotwinEntityTrait};
use crate::score_calculation::scores::ScoreTrait;
use super::{ObservableTrait, ObserverTrait};
use crossbeam_channel::*;
use rayon::prelude::*;
use std::env;

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

    pub fn solve<DomainType, CotwinBuilder, EntityVariants, UtilityObjectVariants, ScoreType> (
        domain: DomainType,
        cotwin_builder: CotwinBuilder,
        agent_builders: Vec<AgentBuildersVariants<ScoreType>>,
        score_precision: Vec<u64>
    )
    where
    DomainType: Clone + Send,
    CotwinBuilder: CotwinBuilderTrait<DomainType, EntityVariants, UtilityObjectVariants, ScoreType> + Clone + Send,
    EntityVariants: CotwinEntityTrait + Send,
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send {

        if score_precision.len() != ScoreType::precision_len() {
            panic!("Invalid score_precision. Suggest: vec![a] for SimpleScore, vec![a, b] for HardSoft, vec![a, b, c] for HardMediumSoft.");
        }
        //self.setup_agents();
        
        let n_jobs = agent_builders.len();
        let agent_ids:Vec<usize> = (0..n_jobs).collect();
        let domains: Vec<DomainType> = vec![domain.clone(); n_jobs];
        let cotwin_builders: Vec<CotwinBuilder> = vec![cotwin_builder.clone(); n_jobs];
        let mut round_robin_status_vec: Vec<AgentStatuses> = Vec::new();
        let mut agents_updates_senders: Vec<Sender<AgentToAgentUpdate>> = Vec::new();
        let mut agents_updates_receivers: Vec<Receiver<AgentToAgentUpdate>> = Vec::new();
        for i in 0..n_jobs {
            round_robin_status_vec.insert(i, AgentStatuses::Alive);
            let (agent_i_updates_sender, agent_i_updates_receiver): (Sender<AgentToAgentUpdate>, Receiver<AgentToAgentUpdate>) = bounded(1);
            agents_updates_senders.push(agent_i_updates_sender);
            agents_updates_receivers.push(agent_i_updates_receiver);
        }
        let agents_round_robin_status_clones = vec![round_robin_status_vec.clone(); n_jobs];
        agents_updates_receivers.rotate_right(1);

        /*let agent_ids = Arc::new(Mutex::new(agent_ids));
        let domains = Arc::new(Mutex::new(domains));
        let cotwin_builders = Arc::new(Mutex::new(cotwin_builders));
        let agent_builders = Arc::new(Mutex::new(agent_builders));
        let agents_round_robin_status_clones = Arc::new(Mutex::new(agents_round_robin_status_clones));

        let agents_pool = ThreadPool::new(n_jobs);
        for i in 0..n_jobs {
            let agent_ids = Arc::clone(&agent_ids);
            let domains = Arc::clone(&domains);
            let cotwin_builders = Arc::clone(&cotwin_builders);
            let agent_builders = Arc::clone(&agent_builders);
            let agents_round_robin_status_clones = Arc::clone(&agents_round_robin_status_clones);
            let agent_i_update_sender = agents_updates_senders[i].clone();
            let agent_i_updates_receiver = agents_updates_receivers[i].clone();
            agents_pool.execute( move || {

                let id_i = agent_ids.lock().unwrap()[i].clone();
                let domain_i = domains.lock().unwrap()[i].clone();
                let agent_builder_i = agent_builders.lock().unwrap()[i].clone();
                let cotwin_builder_i = cotwin_builders.lock().unwrap()[i].clone();
                let rrsc_i = agents_round_robin_status_clones.lock().unwrap()[i].clone();

                let cotwin_i = cotwin_builder_i.build_cotwin(domain_i);
                let mut agent_i;
                match agent_builder_i {
                    AgentBuildersVariants::GA(ga_builder) => agent_i = ga_builder.build_agent(cotwin_i)
                }
                agent_i.agent_id = id_i;
                agent_i.round_robin_status_vec = rrsc_i;
                agent_i.updates_to_agent_sender = Some(agent_i_update_sender);
                agent_i.updates_for_agent_receiver = Some(agent_i_updates_receiver);

                agent_i.solve();
            });
        }
        agents_pool.join();*/

        //agents_pool.join();

        //let thread_pool = rayon::ThreadPoolBuilder::new().num_threads(n_jobs).build().unwrap();
        domains.into_par_iter()
        .zip(cotwin_builders.into_par_iter())
        .zip(agent_builders.into_par_iter())
        .zip(agent_ids.into_par_iter())
        .zip(agents_round_robin_status_clones.into_par_iter())
        .zip(agents_updates_senders.into_par_iter())
        .zip(agents_updates_receivers.into_par_iter())
        .for_each(|((((((domain_i, cotwin_builder_i), agent_builder_i), id_i), rrs_i), us_i), rc_i)| {
            let cotwin_i = cotwin_builder_i.build_cotwin(domain_i);
            let mut agent_i;
            match agent_builder_i {
                AgentBuildersVariants::GA(ga_builder) => agent_i = ga_builder.build_agent(cotwin_i)
            }
            agent_i.agent_id = id_i;
            agent_i.round_robin_status_vec = rrs_i;
            agent_i.updates_to_agent_sender = Some(us_i);
            agent_i.updates_for_agent_receiver = Some(rc_i);
            
            //env::set_var("POLARS_MAX_THREADS",  (24 * n_jobs).to_string());
            agent_i.solve();
        });

        //let agents_pool = ThreadPool::new(n_jobs);
        /*for i in 0..n_jobs {
            agents_pool.execute( move || {

                let domain_i = domains[i];
                let cotwin_builder_i = cotwin_builders[i];
                let cotwin_i = cotwin_builder_i.build_cotwin(&domain_i);

                /*agents[i].round_robin_status_vec = round_robin_status_vec.clone();
                agents[i].updates_to_agent_sender = Some(agents_updates_senders[i].clone());
                agents[i].updates_for_agent_receiver = Some(agents_updates_receivers[next_agent_id].clone());

                agent.solve()*/
            });
        }*/

        //agents_pool.join();

    }

}