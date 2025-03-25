

use crate::agents::base::{Agent, AgentStatuses, AgentToAgentUpdate, Individual};
use crate::agents::AgentBuildersVariants;
use crate::domain::DomainBuilderTrait;
use crate::cotwin::{CotwinBuilderTrait, CotwinEntityTrait};
use crate::score_calculation::scores::ScoreTrait;
use super::ObserverTrait;
use super::SolverLoggingLevels;
use super::InitialSolutionVariants;

use std::ops::{AddAssign, Sub};
use std::fmt::{Debug, Display};
use std::sync::{Arc, Mutex};
use crossbeam_channel::*;
use rayon::prelude::*;
use std::env;
use serde::Serialize;
use serde_json::Value;

pub struct Solver {}

impl Solver {

    pub fn solve<DomainType, DomainBuilder, CotwinBuilder, EntityVariants, UtilityObjectVariants, ScoreType> (
        domain_builder: DomainBuilder,
        cotwin_builder: CotwinBuilder,
        agent_builder: AgentBuildersVariants<ScoreType>,
        n_jobs: usize,
        score_precision: Option<Vec<u64>>,
        logging_level: SolverLoggingLevels,
        observers: Option<Vec<Box<dyn ObserverTrait + Send>>>,
        initial_solution: Option<InitialSolutionVariants<DomainType>>
    ) -> Value
    where
    DomainType: Clone + Send,
    DomainBuilder: DomainBuilderTrait<DomainType> + Clone + Send + Sync,
    CotwinBuilder: CotwinBuilderTrait<DomainType, EntityVariants, UtilityObjectVariants, ScoreType> + Clone + Send,
    EntityVariants: CotwinEntityTrait + Send,
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Display + Send + Serialize {

        /* 
            Returns serde_json::Value to use it as json string (solution.to_string()) to send via http or
            parse it directly by serde_json::from_value(solution).unwrap() and use it in DomainUpdater.
            Note: serde_json::from_value needs type annotation. For solution it's (Vec<(String, AnyValue)>, ScoreType),
            where ScoreType is current score type used for task.
        */

        match &score_precision {
            Some(precision) => {
                if precision.len() != ScoreType::precision_len() {
                    panic!("Invalid score_precision. Suggest: vec![a] for SimpleScore, vec![a, b] for HardSoft, vec![a, b, c] for HardMediumSoft.");
                }
            }
            None => ()
        }

        let agent_ids:Vec<usize> = (0..n_jobs).collect();
        let domain_builders: Vec<DomainBuilder> = vec![domain_builder.clone(); n_jobs];
        let cotwin_builders: Vec<CotwinBuilder> = vec![cotwin_builder.clone(); n_jobs];
        let agent_builders: Vec<AgentBuildersVariants<ScoreType>> = vec![agent_builder.clone(); n_jobs];
        let score_precisions = vec![score_precision; n_jobs];
        let logging_levels = vec![logging_level; n_jobs];
        let initial_solutions: Vec<Option<InitialSolutionVariants<DomainType>>> = vec![initial_solution.clone(); n_jobs];
        let mut round_robin_status_vec: Vec<AgentStatuses> = Vec::new();
        let mut agents_updates_senders: Vec<Sender<AgentToAgentUpdate<ScoreType>>> = Vec::new();
        let mut agents_updates_receivers: Vec<Receiver<AgentToAgentUpdate<ScoreType>>> = Vec::new();
        let global_top_individual: Individual<ScoreType> = Individual::new(vec![1.0], ScoreType::get_stub_score());
        let global_top_individual = Arc::new(Mutex::new(global_top_individual));
        let global_top_json = Arc::new(Mutex::new(Value::Null));
        
        let observers_counts: Vec<usize>;
        let observers_arc:Arc<Mutex<Option<Vec<Box<dyn ObserverTrait + Send>>>>>;
        match observers {
            None => {
                observers_counts = vec![0; n_jobs];
                observers_arc = Arc::new(Mutex::new(None));
            }
            Some(observers) => {
                observers_counts = vec![observers.len(); n_jobs];
                observers_arc = Arc::new(Mutex::new(Some(observers)));
            }
        }

        for i in 0..n_jobs {
            round_robin_status_vec.insert(i, AgentStatuses::Alive);
            let (agent_i_updates_sender, agent_i_updates_receiver): (Sender<AgentToAgentUpdate<ScoreType>>, Receiver<AgentToAgentUpdate<ScoreType>>) = bounded(1);
            agents_updates_senders.push(agent_i_updates_sender);
            agents_updates_receivers.push(agent_i_updates_receiver);
        }
        let agents_round_robin_status_clones = vec![round_robin_status_vec.clone(); n_jobs];
        agents_updates_receivers.rotate_right(1);

        domain_builders.into_par_iter()
        .zip(cotwin_builders.into_par_iter())
        .zip(agent_builders.into_par_iter())
        .zip(agent_ids.into_par_iter())
        .zip(agents_round_robin_status_clones.into_par_iter())
        .zip(agents_updates_senders.into_par_iter())
        .zip(agents_updates_receivers.into_par_iter())
        .zip(score_precisions.into_par_iter())
        .zip(logging_levels.into_par_iter())
        .zip(observers_counts.into_par_iter())
        .zip(initial_solutions.into_par_iter())
        .for_each(|((((((((((db_i, cb_i), ab_i), ai_i), rrs_i), us_i), rc_i), sp_i), ll_i), oc_i), is_i)| {
            let domain_i;
            let mut is_already_initialized = true;
            match is_i {
                None => {
                    is_already_initialized = false;
                    domain_i = db_i.build_domain_from_scratch();
                },
                Some(isv) => {
                    match isv {
                        InitialSolutionVariants::CotwinValuesVector(raw_solution) => domain_i = db_i.build_from_solution(&raw_solution, None),
                        InitialSolutionVariants::DomainObject(existing_domain) => domain_i = db_i.build_from_domain(&existing_domain),
                    }
                }
            }
            let cotwin_i = cb_i.build_cotwin(domain_i, is_already_initialized);
            let mut agent_i;
            match ab_i {
                AgentBuildersVariants::GA(ga_builder) => agent_i = ga_builder.build_agent(cotwin_i),
                AgentBuildersVariants::LA(la_builder) => agent_i = la_builder.build_agent(cotwin_i),
                AgentBuildersVariants::TS(ts_builder) => agent_i = ts_builder.build_agent(cotwin_i),
                AgentBuildersVariants::SA(sa_builder) => agent_i = sa_builder.build_agent(cotwin_i),
                AgentBuildersVariants::LSH(lsd_builder) => agent_i = lsd_builder.build_agent(cotwin_i),
            }
            agent_i.agent_id = ai_i;
            agent_i.score_precision = sp_i;
            agent_i.round_robin_status_vec = rrs_i;
            agent_i.alive_agents_count = n_jobs;
            agent_i.updates_to_agent_sender = Some(us_i);
            agent_i.updates_for_agent_receiver = Some(rc_i);
            agent_i.global_top_individual = Arc::clone(&global_top_individual);
            agent_i.global_top_json = Arc::clone(&global_top_json);
            agent_i.logging_level = ll_i;
            agent_i.observers = observers_arc.clone();
            agent_i.observers_count = oc_i;
            
            //env::set_var("POLARS_MAX_THREADS",  (24 * n_jobs).to_string());
            agent_i.solve();
        });


        let solution_json =  global_top_json.lock().unwrap().clone();
        return solution_json;

    }
}