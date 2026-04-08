use process_mining::core::process_models::case_centric::dfg::DirectlyFollowsGraph;

pub fn filter_dfg_absolute<'a>(dfg: &DirectlyFollowsGraph<'a>, threshold: u32) -> DirectlyFollowsGraph<'a> {
    let mut result = DirectlyFollowsGraph::new();

    for (activity, freq) in &dfg.activities {
        result.add_activity(activity.clone(), *freq);
    }

    for ((from, to), freq) in &dfg.directly_follows_relations {
        if *freq >= threshold {
            result.add_df_relation(from.clone(), to.clone(), *freq);
        }
    }

    for activity in &dfg.start_activities {
        result.add_start_activity(activity.clone());
    }

    for activity in &dfg.end_activities {
        result.add_end_activity(activity.clone());
    }

    result
}
