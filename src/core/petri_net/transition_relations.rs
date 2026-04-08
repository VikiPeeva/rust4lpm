use process_mining::core::process_models::petri_net::{ArcType, TransitionID};
use process_mining::PetriNet;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Returns all (A, B) transition pairs where A can transitively reach B through the net.
pub fn sequence_pairs(net: &PetriNet) -> HashSet<(TransitionID, TransitionID)> {
    //TODO: probably a more efficient implementation exists directly on the Petri net or finding
    // the transitive closure instead of all pairs
    let direct = direct_successors(net);

    net.transitions
        .keys()
        .flat_map(|t| {
            reachable_from(t, &direct)
                .into_iter()
                .map(|reachable| (TransitionID(*t), TransitionID(reachable)))
        })
        .collect()
}

/// Returns all unordered (A, B) transition pairs that share an input or output place.
pub fn choice_pairs(net: &PetriNet) -> HashSet<(TransitionID, TransitionID)> {
    let mut pairs = HashSet::new();

    for place_uuid in net.places.keys() {
        let consumers: Vec<Uuid> = net.arcs.iter()
            .filter_map(|arc| match arc.from_to {
                ArcType::PlaceTransition(p, t) if &p == place_uuid => Some(t),
                _ => None,
            })
            .collect();

        let producers: Vec<Uuid> = net.arcs.iter()
            .filter_map(|arc| match arc.from_to {
                ArcType::TransitionPlace(t, p) if &p == place_uuid => Some(t),
                _ => None,
            })
            .collect();

        for pair in unordered_pairs(&consumers) { pairs.insert(pair); }
        for pair in unordered_pairs(&producers) { pairs.insert(pair); }
    }
    pairs
}

/// Extends choice pairs by propagating through sequence relations:
/// if (A, B) are in choice and C is in sequence with A but not B (or vice versa),
/// then (C, B) is also a choice pair. Repeats until no new pairs are found.
pub fn extended_choice_pairs(net: &PetriNet) -> HashSet<(TransitionID, TransitionID)> {
    let seq = sequence_pairs(net);
    let seq_set: HashSet<(Uuid, Uuid)> = seq.iter()
        .flat_map(|(a, b)| [(a.0, b.0), (b.0, a.0)])
        .collect();

    let mut choices = choice_pairs(net);
    let mut changed = true;

    while changed {
        changed = false;
        let current: Vec<_> = choices.iter().cloned().collect();
        for (a, b) in current {
            for t in net.transitions.keys() {
                let seq_with_a = seq_set.contains(&(*t, a.0)) || seq_set.contains(&(a.0, *t));
                let seq_with_b = seq_set.contains(&(*t, b.0)) || seq_set.contains(&(b.0, *t));
                if seq_with_a && !seq_with_b {
                    if choices.insert(norm_pair(TransitionID(*t), b)) { changed = true; }
                }
                if seq_with_b && !seq_with_a {
                    if choices.insert(norm_pair(TransitionID(*t), a)) { changed = true; }
                }
            }
        }
    }
    choices
}

fn norm_pair(a: TransitionID, b: TransitionID) -> (TransitionID, TransitionID) {
    if a <= b { (a, b) } else { (b, a) }
}

fn unordered_pairs(ids: &[Uuid]) -> Vec<(TransitionID, TransitionID)> {
    let mut pairs = vec![];
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            pairs.push((TransitionID(ids[i]), TransitionID(ids[j])));
        }
    }
    pairs
}

/// Builds a map from each transition to its directly reachable transitions (via one place).
fn direct_successors(net: &PetriNet) -> HashMap<Uuid, Vec<Uuid>> {
    let mut map: HashMap<Uuid, Vec<Uuid>> = net.transitions.keys().map(|t| (*t, vec![])).collect();

    for arc_ab in &net.arcs {
        let ArcType::TransitionPlace(t_uuid, p_uuid) = arc_ab.from_to else {
            continue;
        };
        for arc_bc in &net.arcs {
            let ArcType::PlaceTransition(p2_uuid, t2_uuid) = arc_bc.from_to else {
                continue;
            };
            if p_uuid == p2_uuid {
                map.entry(t_uuid).or_default().push(t2_uuid);
            }
        }
    }
    map
}

/// BFS from a transition, returning all transitively reachable transition UUIDs (excluding itself).
fn reachable_from(start: &Uuid, graph: &HashMap<Uuid, Vec<Uuid>>) -> HashSet<Uuid> {
    let mut visited = HashSet::new();
    let mut queue = std::collections::VecDeque::new();

    if let Some(neighbors) = graph.get(start) {
        queue.extend(neighbors.iter().copied());
    }

    while let Some(current) = queue.pop_front() {
        if visited.insert(current) {
            if let Some(neighbors) = graph.get(&current) {
                queue.extend(neighbors.iter().copied());
            }
        }
    }
    visited
}