



use serde_json::{json, Value};
use greyjack::{domain::DomainBuilderTrait, solver::ObserverTrait};
use crate::persistence::DomainBuilder;
use lapin::{Channel, options::*, BasicProperties,};
use tokio::runtime::Handle;

#[derive(Clone)]
pub struct RabbitMQObserver {

    domain_builder: DomainBuilder,
    solution_sender_channel: Channel,
    tokio_runtime: Handle,
}

impl RabbitMQObserver {
    
    pub fn new(domain_builder: DomainBuilder, solution_sender_channel: Channel, tokio_runtime: Handle) -> Self {

        Self {
            domain_builder: domain_builder,
            solution_sender_channel: solution_sender_channel,
            tokio_runtime: tokio_runtime,
        }
    }
}

impl ObserverTrait for RabbitMQObserver {

    fn update(&mut self, solution: Value) {

        let domain = self.domain_builder.build_from_solution(&solution, None);
        let domain_json = json!(&domain).clone();

        let channel = self.solution_sender_channel.clone();
        let guard = self.tokio_runtime.enter();
        self.tokio_runtime.block_on(async move {
            channel
            .basic_publish(
                "vrp_solutions_exchange",
                "vrp_out",
                BasicPublishOptions::default(),
                domain_json.to_string().as_bytes(),
                BasicProperties::default(),
            )
            .await.unwrap();
        });
        drop(guard);
    }
    
}

unsafe impl Send for RabbitMQObserver {
    
}