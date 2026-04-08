use process_mining::core::process_models::petri_net::{ArcType, Marking, PlaceID};
use process_mining::PetriNet;
use uuid::Uuid;

/// Returns all traces (sequences of visible transition labels) that are reachable
/// from the initial marking and end in a final marking, with at most `max_steps`
/// total transition firings (including silent ones).
pub fn generate_language(net: &PetriNet, max_steps: usize) -> Vec<Vec<String>> {
    let initial = net.initial_marking.clone().unwrap_or_default();
    let mut results = Vec::new();
    // Stack entries: (current_marking, visible_trace, steps_taken)
    let mut stack = vec![(initial, Vec::<String>::new(), 0usize)];

    while let Some((marking, trace, steps)) = stack.pop() {
        if is_final(net, &marking) {
            results.push(trace.clone());
        }
        if steps >= max_steps {
            continue;
        }
        for (t_uuid, transition) in &net.transitions {
            if is_enabled(net, &marking, t_uuid) {
                let new_marking = fire(net, &marking, t_uuid);
                let mut new_trace = trace.clone();
                if let Some(label) = &transition.label {
                    new_trace.push(label.clone());
                }
                stack.push((new_marking, new_trace, steps + 1));
            }
        }
    }
    results
}

fn is_enabled(net: &PetriNet, marking: &Marking, t_uuid: &Uuid) -> bool {
    for arc in &net.arcs {
        if let ArcType::PlaceTransition(p_uuid, arc_t_uuid) = arc.from_to {
            if &arc_t_uuid == t_uuid {
                let tokens = marking.get(&PlaceID(p_uuid)).copied().unwrap_or(0);
                if tokens < arc.weight as u64 {
                    return false;
                }
            }
        }
    }
    true
}

fn fire(net: &PetriNet, marking: &Marking, t_uuid: &Uuid) -> Marking {
    let mut new_marking = marking.clone();
    for arc in &net.arcs {
        match arc.from_to {
            ArcType::PlaceTransition(p_uuid, arc_t_uuid) if &arc_t_uuid == t_uuid => {
                let p_id = PlaceID(p_uuid);
                let tokens = new_marking.entry(p_id).or_insert(0);
                *tokens -= arc.weight as u64;
                if *tokens == 0 {
                    new_marking.remove(&PlaceID(p_uuid));
                }
            }
            ArcType::TransitionPlace(arc_t_uuid, p_uuid) if &arc_t_uuid == t_uuid => {
                *new_marking.entry(PlaceID(p_uuid)).or_insert(0) += arc.weight as u64;
            }
            _ => {}
        }
    }
    new_marking
}

fn is_final(net: &PetriNet, marking: &Marking) -> bool {
    let Some(final_markings) = &net.final_markings else {
        return false;
    };
    final_markings.iter().any(|fm| {
        fm.iter().all(|(p, &required)| marking.get(p).copied().unwrap_or(0) == required)
            && marking.iter().all(|(p, &tokens)| tokens == 0 || fm.contains_key(p))
    })
}