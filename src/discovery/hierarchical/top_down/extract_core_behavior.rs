use process_mining::discovery::case_centric::alphappp::log_repair::filter_dfg;
use process_mining::discovery::case_centric::dfg::discover_dfg;
use process_mining::EventLog;
use crate::discovery::hierarchical::top_down::utils::cutoff_strategy::{CutoffStrategy, HighestDifference};

pub fn default(event_log: &EventLog) {
    with_dfg(event_log, HighestDifference);
}

pub fn with_dfg(event_log: &EventLog, cutoff: impl CutoffStrategy) {
    // 1. create a dfg
    let dfg = discover_dfg(event_log);

    // 2. find a cutoff
    let mut values: Vec<u32> = dfg.directly_follows_relations.values().copied().collect();
    values.sort();
    let cutoff_value = cutoff.cutoff(&values);

    // 3. extract main behavior
}

pub fn with_lcs(event_log: &EventLog) {

}