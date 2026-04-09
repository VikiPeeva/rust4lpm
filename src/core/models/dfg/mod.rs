use process_mining::core::process_models::case_centric::dfg::DirectlyFollowsGraph;
use std::fmt::Write;
use crate::core::models::DotRenderable;

impl<'a> DotRenderable for DirectlyFollowsGraph<'a> {
    fn to_dot(&self) -> String {
        let mut out = String::new();
        writeln!(out, "digraph DFG {{").unwrap();
        writeln!(out, "    node [shape=box];").unwrap();

        // Activity nodes with frequency as label
        for (activity, freq) in &self.activities {
            writeln!(out, "    \"{}\" [label=\"{}({})\"];", activity, activity, freq).unwrap();
        }

        // Directly-follows edges with frequency labels
        for ((from, to), freq) in &self.directly_follows_relations {
            writeln!(out, "    \"{}\" -> \"{}\" [label=\"{}\"];", from, to, freq).unwrap();
        }

        writeln!(out, "}}").unwrap();
        out
    }
}
