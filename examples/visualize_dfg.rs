use process_mining::{Importable,EventLog};
use process_mining::core::process_models::dfg::DirectlyFollowsGraph;
use rust4lpm::core::models::{dfg, DotRenderable};

fn main() {
    println!("{}", std::env::current_dir().unwrap().display());
    let log = EventLog::import_from_path("examples/example_data/artificialSmall.xes")
        .expect("failed to import log");
    let dfg = DirectlyFollowsGraph::discover(&log);
    println!("{}", dfg.to_dot());
    dfg.render();
}