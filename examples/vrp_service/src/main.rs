mod domain;
mod cotwin;
mod score;
mod persistence;
mod observers;

use std::default;
use std::path::{PathBuf, Path};
use greyjack::domain::DomainBuilderTrait;
use greyjack::cotwin::CotwinBuilderTrait;
use lapin::{Channel, Consumer, ExchangeKind};
use persistence::{CotwinBuilder, DomainBuilder};
use greyjack::solver::{InitialSolutionVariants, ObserverTrait, Solver, SolverLoggingLevels};
use greyjack::agents::{GeneticAlgorithm, LateAcceptance, TabuSearch};
use greyjack::agents::AgentBuildersVariants::*;
use greyjack::agents::termination_strategies::*;
use greyjack::agents::termination_strategies::TerminationStrategiesVariants::*;
use observers::RabbitMQObserver;
use rocket::futures::StreamExt;
use tokio;
use serde_json::*;
use tokio::runtime::Handle;

use lapin::{
    options::*, publisher_confirm::Confirmation, types::FieldTable, BasicProperties, Connection,
    ConnectionProperties, Result, message::Delivery,
};

#[tokio::main]
async fn main() {

    // RabbitMQ instance deployed inside Docker on Ubuntu virtual machine by the official guide
    let rabbitmq_address = "amqp://192.168.0.189:5672/%2f";
    let connection = Connection::connect(&rabbitmq_address, ConnectionProperties::default())
        .await
        .expect("Connection error");

    let tokio_runtime = Handle::current();
    let mut consumer = build_task_data_consumer(&connection).await;
    let mut solution_sender_channel = build_solution_sender_channel(&connection).await;
    while let Some(incoming_message) = consumer.next().await {
        match incoming_message {
            Ok(incoming_message) => {
                //println!("{:?}", incoming_message);
                if let Err(err) = solve_vrp(&incoming_message, &mut solution_sender_channel, tokio_runtime.clone()).await {
                    incoming_message
                        .nack(Default::default())
                        .await
                        .expect("Failed to send nack");
                } else {
                    incoming_message
                        .ack(Default::default())
                        .await
                        .expect("Failed to send ack");
                }

                let channel = solution_sender_channel.clone();
                        let result = tokio::spawn( async move {
                            let send_result = channel
                            .basic_publish(
                                "vrp_solutions_exchange",
                                "vrp_out",
                                BasicPublishOptions::default(),
                                "Solving finished".to_string().as_bytes(),
                                BasicProperties::default(),
                            )
                            .await;
                        });
            }
            Err(err) => {
                println!("Failed to receive message");
            }
        }
    }

    connection.close(200, "VRP solver service successfully ended work").await;
    println!("done");
}

async fn solve_vrp(delivery: &Delivery, solution_sender_channel: &mut Channel, tokio_runtime: Handle) -> Result<()> {

    let vrp_json_string = match std::str::from_utf8(&delivery.data) {
        Ok(v) => v.to_string(),
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };
    let vrp_json: Value = serde_json::from_str(&vrp_json_string).unwrap();


    let domain_builder = DomainBuilder::new(&vrp_json);
    let cotwin_builder = CotwinBuilder::new(true, true);

    let termination_strategy = SNI(ScoreNoImprovement::new(5*1000));
    let agent_builder = TS(TabuSearch::new(1024, 0.2, None, Some(vec![0.5, 0.5, 0.0, 0.0, 0.0, 0.0]), 10, termination_strategy));

    let rabbitmq_observer = RabbitMQObserver::new(domain_builder.clone(), solution_sender_channel.clone(), tokio_runtime.clone());
    let mut observers: Vec<Box<dyn ObserverTrait + Send>> = Vec::new();
    observers.push(Box::new(rabbitmq_observer));
    Solver::solve(
        domain_builder.clone(), cotwin_builder, agent_builder, 
        10, Some(vec![0, 0, 3]), SolverLoggingLevels::FreshOnly, 
        Some(observers), None,
    );

    return Ok(());
}

async fn build_task_data_consumer(connection: &Connection) -> Consumer{

    let channel = connection.create_channel().await.expect("Channel creation error");

    channel.exchange_declare(
        "vrp_exchange", 
        ExchangeKind::Direct, 
        ExchangeDeclareOptions::default(), 
        FieldTable::default())
        .await
        .expect("Failed to create exchange");

    channel.queue_declare(
        "vrp_task_data",
        QueueDeclareOptions::default(),
        FieldTable::default()
    )
    .await
    .expect("Failded to bind queue");

    let mut consumer = channel
    .basic_consume(
        "vrp_task_data",
        "vrp_in",
        BasicConsumeOptions::default(),
        FieldTable::default(),
    )
    .await
    .unwrap();

    return consumer;
} 

async fn build_solution_sender_channel(connection: &Connection) -> Channel{
    let channel = connection.create_channel().await.expect("Failed to create channel");

    channel
        .exchange_declare(
            "vrp_solutions_exchange",
            lapin::ExchangeKind::Fanout,
            Default::default(),
            FieldTable::default(),
        )
        .await
        .expect("Failed to create exchange");

    let queue = channel
        .queue_declare(
            "vrp_solutions",
            QueueDeclareOptions {
                exclusive: false,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .expect("Failed to create queue");

    channel
        .queue_bind(
            "vrp_solutions",
            "vrp_solutions_exchange",
            "vrp_out",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Failed to bind queue");

    return channel;
}