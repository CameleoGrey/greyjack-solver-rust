
use std::time::Instant;
use std::vec::Vec;
use threadpool::*;
use crossbeam_channel::*;

fn main() {

    let n_jobs:usize = 12;

    let mut agents: Vec<Agent> = Vec::new();
    let mut update_senders: Vec<Sender<usize>> = Vec::new();
    let mut update_receivers: Vec<Receiver<usize>> = Vec::new();

    for i in 0..n_jobs {
        let (sender_i, receiver_i): (Sender<usize>, Receiver<usize>) = bounded(1);
        update_senders.push(sender_i);
        update_receivers.push(receiver_i);
    }
    update_receivers.rotate_right(1);

    for i in 0..n_jobs {
        let current_sender = update_senders[i].clone();
        let current_receiver = update_receivers[i].clone();
        let current_agent = Agent::new(i, current_sender, current_receiver);
        agents.push(current_agent);
    }

    let agents_pool = ThreadPool::new(n_jobs);
    for agent in agents {
        let mut agent = agent;
        agents_pool.execute(move || agent.launch_counting());
    }

    agents_pool.join();

    println!("done");

}


#[derive(Debug)]
struct Agent {
    agent_id: usize,
    counter: i64,
    updates_sender: Sender<usize>,
    updates_receiver: Receiver<usize>

}

impl Agent {

    pub fn new(agent_id: usize, sender: Sender<usize>, receiver: Receiver<usize>) -> Self {
        Agent {
            agent_id: agent_id,
            counter: 0,
            updates_sender: sender,
            updates_receiver: receiver
        }
    }

    pub fn launch_counting(&mut self) {
        
        let start = Instant::now();
        loop {
            self.counter += 1;

            if self.counter >= 1_000_000_000 {
                break;
            }

            if self.counter % 1_000_000 == 0 {
                if self.agent_id % 2 == 0 {
                    let sending_updates_result = self.updates_sender.send(self.agent_id).unwrap();
                    let update = self.updates_receiver.recv().unwrap();
                    println!("Agent {} received update from agent {}", self.agent_id, update);

                } else {
                    let update = self.updates_receiver.recv().unwrap();
                    println!("Agent {} received update from agent {}", self.agent_id, update);
                    let sending_updates_result = self.updates_sender.send(self.agent_id).unwrap();

                }
            }
        }

        let total_time = start.elapsed();
        println!("agent {} is dead. Execution time: {:?}", self.agent_id, total_time);
    }
}