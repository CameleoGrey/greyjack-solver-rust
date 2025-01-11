
use crate::agents::tabu::Tabu;
use crate::agents::lshade::LSHADE;

mod agents;

fn main() {

    //let agent = Tabu::default();
    //agent.print_info();

    let mut agent = Tabu::new(1000, 0.2);
    agent.print_info();

    agent.set_tabu_size(100);
    agent.print_info();

    let mut agent = LSHADE::new(0.02, 0.5);
    agent.print_info();

}
