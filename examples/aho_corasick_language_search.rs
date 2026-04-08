use aho_corasick::AhoCorasick;
use process_mining::core::event_data::case_centric::AttributeValue;
use process_mining::{EventLog, Importable, PetriNet};
use rust4lpm::core::petri_net::language::generate_language;
use std::collections::{HashMap, HashSet};

/// Returns the `concept:name` attribute value of an event, if present.
fn concept_name(event: &process_mining::core::event_data::case_centric::Event) -> Option<&str> {
    event.attributes.iter().find(|a| a.key == "concept:name").and_then(|a| {
        if let AttributeValue::String(s) = &a.value { Some(s.as_str()) } else { None }
    })
}

/// Reads the event log and Petri net from example_data, generates the Petri net language,
/// then uses Aho-Corasick to find all (overlapping) occurrences of each language pattern
/// in every event log trace — after projecting each trace onto the language alphabet.
///
/// Projection is necessary because Aho-Corasick matches contiguous byte sub-sequences:
/// filtering out activities that are not in the language alphabet lets us treat the
/// remaining events as a direct sub-word search problem.
///
/// Note: this approach only finds *contiguous* matches — patterns with gaps between
/// alphabet activities will be missed. Proper subsequence matching requires a different
/// approach (e.g. automaton-based replay / conformance checking).
fn main() {
    // 1. Load data
    let net = PetriNet::import_from_path("examples/example_data/artificialSmall_0.pnml")
        .expect("failed to read Petri net");
    let log = EventLog::import_from_path("examples/example_data/artificialSmall.xes")
        .expect("failed to import event log");

    // 2. Generate the Petri net language (all valid firing sequences up to max_steps)
    let language = generate_language(&net, 20);
    assert!(!language.is_empty(), "language must not be empty");
    println!("Language ({} trace(s)):", language.len());
    for t in &language {
        println!("  {t:?}");
    }

    // 3. Build the language alphabet and assign each symbol a unique byte (1-indexed)
    let alphabet: HashSet<&str> =
        language.iter().flat_map(|t| t.iter().map(String::as_str)).collect();
    let mut sorted: Vec<&str> = alphabet.into_iter().collect();
    sorted.sort_unstable();
    let label_to_byte: HashMap<&str, u8> =
        sorted.iter().enumerate().map(|(i, &s)| (s, (i + 1) as u8)).collect();

    // 4. Encode each language trace as a byte pattern
    let patterns: Vec<Vec<u8>> = language
        .iter()
        .map(|t| t.iter().map(|a| label_to_byte[a.as_str()]).collect())
        .collect();

    // 5. Build Aho-Corasick automaton over all language patterns
    let ac = AhoCorasick::new(&patterns).expect("failed to build Aho-Corasick automaton");

    // 6. For each event log trace, project it onto the language alphabet and search
    let mut total_matches: usize = 0;
    for (i, trace) in log.traces.iter().enumerate() {
        let projected: Vec<u8> = trace
            .events
            .iter()
            .filter_map(|e| concept_name(e))
            .filter_map(|name| label_to_byte.get(name).copied())
            .collect();

        for m in ac.find_overlapping_iter(&projected) {
            let pat = &language[m.pattern().as_usize()];
            println!("Trace {i}: pattern {pat:?} at positions {}..{}", m.start(), m.end());
            total_matches += 1;
        }
    }

    println!("Total matches: {total_matches}");
}