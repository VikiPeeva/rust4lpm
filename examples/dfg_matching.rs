use std::collections::{HashMap, HashSet, VecDeque};

// ---- Data Structures ----

#[derive(Debug, Clone)]
pub struct DFG {
    pub edges: HashMap<String, HashSet<String>>,
}

impl DFG {
    pub fn new() -> Self {
        DFG { edges: HashMap::new() }
    }

    pub fn add_edge(&mut self, from: &str, to: &str) {
        self.edges.entry(from.to_string()).or_default().insert(to.to_string());
        self.edges.entry(to.to_string()).or_default();
    }

    pub fn nodes(&self) -> HashSet<&String> {
        self.edges.keys().collect()
    }

    fn reversed(&self) -> HashMap<String, HashSet<String>> {
        let mut rev: HashMap<String, HashSet<String>> = self.edges.keys()
            .map(|k| (k.clone(), HashSet::new()))
            .collect();
        for (from, targets) in &self.edges {
            for to in targets {
                rev.entry(to.clone()).or_default().insert(from.clone());
            }
        }
        rev
    }
}

// ---- Partial Order Trace ----

#[derive(Debug, Clone)]
pub enum NodeLabel {
    Activity(String),
    Abstract(String),
}

#[derive(Debug, Clone)]
pub struct PartialOrderTrace {
    pub nodes: Vec<NodeLabel>,
    pub edges: HashSet<(usize, usize)>,
}

// ---- SCC (Kosaraju's algorithm) ----

fn kosaraju_pass1<'a>(
    start: &'a String,
    edges: &'a HashMap<String, HashSet<String>>,
    visited: &mut HashSet<&'a String>,
    finish_order: &mut Vec<&'a String>,
) {
    // Iterative DFS with a "finished" flag to simulate post-order
    let mut stack: Vec<(&'a String, bool)> = vec![(start, false)];
    while let Some((node, finished)) = stack.pop() {
        if finished {
            finish_order.push(node);
            continue;
        }
        if visited.contains(node) { continue; }
        visited.insert(node);
        stack.push((node, true));
        if let Some(neighbors) = edges.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    stack.push((neighbor, false));
                }
            }
        }
    }
}

fn find_sccs(pattern: &DFG) -> Vec<Vec<String>> {
    // Pass 1: DFS on original graph, record finish order
    let mut visited: HashSet<&String> = HashSet::new();
    let mut finish_order: Vec<&String> = Vec::new();
    for node in pattern.edges.keys() {
        kosaraju_pass1(node, &pattern.edges, &mut visited, &mut finish_order);
    }

    // Pass 2: DFS on reversed graph in reverse finish order
    let rev = pattern.reversed();
    let mut visited2: HashSet<String> = HashSet::new();
    let mut sccs: Vec<Vec<String>> = Vec::new();

    for node in finish_order.into_iter().rev() {
        if visited2.contains(node) { continue; }
        let mut scc = Vec::new();
        let mut stack = vec![node.to_string()];
        while let Some(cur) = stack.pop() {
            if visited2.contains(&cur) { continue; }
            visited2.insert(cur.clone());
            scc.push(cur.clone());
            if let Some(neighbors) = rev.get(&cur) {
                for n in neighbors {
                    if !visited2.contains(n) {
                        stack.push(n.clone());
                    }
                }
            }
        }
        sccs.push(scc);
    }
    sccs
}

fn scc_has_cycle(scc: &[String], pattern: &DFG) -> bool {
    scc.len() > 1
        || pattern.edges.get(&scc[0]).map_or(false, |t| t.contains(&scc[0]))
}

// Returns a linearization of one iteration through the SCC.
// For a single node (self-loop): [node].
// For a multi-node SCC: DFS from the node with the lowest internal in-degree.
fn scc_linearization<'a>(scc: &'a [String], pattern: &'a DFG) -> Vec<&'a String> {
    if scc.len() == 1 {
        return vec![&scc[0]];
    }
    let scc_set: HashSet<&String> = scc.iter().collect();
    let mut in_degree: HashMap<&String, usize> = scc.iter().map(|n| (n, 0usize)).collect();
    for node in scc {
        if let Some(targets) = pattern.edges.get(node) {
            for t in targets {
                if scc_set.contains(t) {
                    *in_degree.get_mut(t).unwrap() += 1;
                }
            }
        }
    }
    let start = in_degree.iter().min_by_key(|&(_, &d)| d).map(|(&n, _)| n).unwrap();

    let mut order = Vec::new();
    let mut visited: HashSet<&String> = HashSet::new();
    let mut stack = vec![start];
    while let Some(node) = stack.pop() {
        if visited.contains(node) { continue; }
        visited.insert(node);
        order.push(node);
        if let Some(targets) = pattern.edges.get(node) {
            let mut neighbors: Vec<&String> = targets.iter()
                .filter(|t| scc_set.contains(*t) && !visited.contains(*t))
                .collect();
            neighbors.sort();
            for n in neighbors.into_iter().rev() {
                stack.push(n);
            }
        }
    }
    order
}

// ---- Matching ----

fn match_one_iteration(
    linearization: &[&String],
    trace: &[String],
    consumed: &[bool],
    min_pos: usize,
) -> Option<Vec<usize>> {
    let mut positions = Vec::new();
    let mut cur = min_pos;
    for activity in linearization {
        let pos = (cur..trace.len()).find(|&i| !consumed[i] && &trace[i] == *activity)?;
        positions.push(pos);
        cur = pos + 1;
    }
    Some(positions)
}

// For acyclic SCCs: match exactly one iteration.
// For cyclic SCCs: greedily match as many iterations as possible (at least one).
// Returns (consumed positions, last matched position).
fn match_scc_greedy(
    scc: &[String],
    pattern: &DFG,
    trace: &[String],
    consumed: &mut Vec<bool>,
    min_pos: usize,
) -> Option<(Vec<usize>, usize)> {
    let linearization = scc_linearization(scc, pattern);
    let has_cycle = scc_has_cycle(scc, pattern);

    let first = match_one_iteration(&linearization, trace, consumed, min_pos)?;
    let mut all_positions = first;
    for &p in &all_positions { consumed[p] = true; }

    if has_cycle {
        loop {
            let next_min = all_positions.iter().max().unwrap() + 1;
            match match_one_iteration(&linearization, trace, consumed, next_min) {
                Some(positions) => {
                    for &p in &positions { consumed[p] = true; }
                    all_positions.extend(positions);
                }
                None => break,
            }
        }
    }

    let last = *all_positions.iter().max().unwrap();
    Some((all_positions, last))
}

fn topo_sort_condensation(n: usize, adj: &[HashSet<usize>]) -> Vec<usize> {
    let mut in_degree = vec![0usize; n];
    for targets in adj {
        for &t in targets { in_degree[t] += 1; }
    }
    let mut queue: VecDeque<usize> = (0..n).filter(|&i| in_degree[i] == 0).collect();
    let mut order = Vec::new();
    while let Some(node) = queue.pop_front() {
        order.push(node);
        for &t in &adj[node] {
            in_degree[t] -= 1;
            if in_degree[t] == 0 { queue.push_back(t); }
        }
    }
    order
}

fn find_match(
    pattern: &DFG,
    trace: &[String],
    consumed: &[bool],
) -> Option<Vec<usize>> {
    let sccs = find_sccs(pattern);

    let mut scc_of: HashMap<&str, usize> = HashMap::new();
    for (i, scc) in sccs.iter().enumerate() {
        for node in scc { scc_of.insert(node.as_str(), i); }
    }

    // Build condensation adjacency (DAG of SCCs)
    let mut condensation: Vec<HashSet<usize>> = vec![HashSet::new(); sccs.len()];
    for (from, targets) in &pattern.edges {
        let fi = scc_of[from.as_str()];
        for to in targets {
            let ti = scc_of[to.as_str()];
            if fi != ti { condensation[fi].insert(ti); }
        }
    }

    let topo = topo_sort_condensation(sccs.len(), &condensation);

    let mut consumed_copy = consumed.to_vec();
    let mut scc_last_pos: Vec<Option<usize>> = vec![None; sccs.len()];
    let mut all_positions = Vec::new();

    for &scc_idx in &topo {
        // Must start after all predecessor SCCs have finished
        let min_pos = (0..sccs.len())
            .filter(|&pred| condensation[pred].contains(&scc_idx))
            .filter_map(|pred| scc_last_pos[pred])
            .map(|p| p + 1)
            .max()
            .unwrap_or(0);

        let (positions, last) = match_scc_greedy(
            &sccs[scc_idx], pattern, trace, &mut consumed_copy, min_pos,
        )?;

        scc_last_pos[scc_idx] = Some(last);
        all_positions.extend(positions);
    }

    Some(all_positions)
}

// ---- Main transformation ----

pub fn abstract_pattern(
    trace: &[String],
    pattern: &DFG,
    abstract_label: &str,
) -> PartialOrderTrace {
    let mut consumed = vec![false; trace.len()];
    let mut matches: Vec<Vec<usize>> = Vec::new();

    loop {
        match find_match(pattern, trace, &consumed) {
            Some(positions) => {
                for &pos in &positions { consumed[pos] = true; }
                matches.push(positions);
            }
            None => break,
        }
    }

    // Build nodes: unmatched activities + one abstract node per match
    let mut nodes: Vec<NodeLabel> = Vec::new();
    let mut trace_pos_to_node: HashMap<usize, usize> = HashMap::new();

    for (i, activity) in trace.iter().enumerate() {
        if !consumed[i] {
            trace_pos_to_node.insert(i, nodes.len());
            nodes.push(NodeLabel::Activity(activity.clone()));
        }
    }

    let mut abstract_nodes: Vec<(usize, usize, usize)> = Vec::new();
    for positions in &matches {
        let first_pos = *positions.iter().min().unwrap();
        let last_pos = *positions.iter().max().unwrap();
        let node_idx = nodes.len();
        nodes.push(NodeLabel::Abstract(abstract_label.to_string()));
        abstract_nodes.push((node_idx, first_pos, last_pos));
    }

    // Build ordering edges
    let mut edges: HashSet<(usize, usize)> = HashSet::new();

    let unmatched_nodes: Vec<(usize, usize)> = trace_pos_to_node
        .iter()
        .map(|(&pos, &nidx)| (pos, nidx))
        .collect::<std::collections::BTreeMap<_, _>>()
        .into_iter()
        .collect();

    // Unmatched activities keep their relative order
    for i in 0..unmatched_nodes.len() {
        for j in (i + 1)..unmatched_nodes.len() {
            edges.insert((unmatched_nodes[i].1, unmatched_nodes[j].1));
        }
    }

    // Order unmatched activities relative to abstract nodes
    for &(abs_idx, first_pos, last_pos) in &abstract_nodes {
        for &(pos, node_idx) in &unmatched_nodes {
            if pos < first_pos {
                edges.insert((node_idx, abs_idx));
            } else if pos > last_pos {
                edges.insert((abs_idx, node_idx));
            }
            // gaps (first_pos <= pos <= last_pos) -> parallel, no edge
        }
    }

    // Order abstract nodes relative to each other
    let mut sorted_abstracts = abstract_nodes.clone();
    sorted_abstracts.sort_by_key(|&(_, first, _)| first);
    for i in 0..sorted_abstracts.len() {
        for j in (i + 1)..sorted_abstracts.len() {
            let (idx_i, _, last_i) = sorted_abstracts[i];
            let (idx_j, first_j, _) = sorted_abstracts[j];
            if last_i < first_j {
                edges.insert((idx_i, idx_j));
            }
        }
    }

    PartialOrderTrace { nodes, edges }
}

// ---- Example ----

fn main() {
    let mut pattern = DFG::new();
    pattern.add_edge("A", "A");

    let trace = vec!["A", "A", "B", "C", "Y", "D"]
        .into_iter().map(String::from).collect::<Vec<_>>();

    let result = abstract_pattern(&trace, &pattern, "P");

    println!("Nodes:");
    for (i, node) in result.nodes.iter().enumerate() {
        println!("  {}: {:?}", i, node);
    }
    println!("Edges (a < b):");
    let mut sorted_edges: Vec<_> = result.edges.iter().collect();
    sorted_edges.sort();
    for (a, b) in sorted_edges {
        println!("  {} < {}", a, b);
    }
}
