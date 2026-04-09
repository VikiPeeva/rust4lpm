use process_mining::core::process_models::dfg::DirectlyFollowsGraph;
use process_mining::{EventLog, Importable};
use rust4lpm::core::models::DotRenderable;
use rust4lpm::discovery::hierarchical::top_down::utils::cutoff_strategy::{CutoffStrategy, HighestDifference};
use rust4lpm::discovery::utils::dfg::{filter_dfg_absolute, inverted_filter_dfg_absolute};

fn main() {
    println!("{}", std::env::current_dir().unwrap().display());
    let log = EventLog::import_from_path("examples/example_data/artificialSmall.xes")
        .expect("failed to import log");
    let dfg = DirectlyFollowsGraph::discover(&log);

    // render dfg
    println!("{}", dfg.to_dot());
    dfg.render();

    // filter dfg
    let mut values: Vec<u32> = dfg.directly_follows_relations.values().copied().collect();
    values.sort();
    let cutoff_value = HighestDifference.cutoff(&values);
    let filtered_dfg = inverted_filter_dfg_absolute(&dfg, cutoff_value);
    println!("{}", filtered_dfg.to_dot());
    filtered_dfg.render();
}
