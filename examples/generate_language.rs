use process_mining::{Importable, PetriNet};
use rust4lpm::core::language::Language;

fn main() {
    let net = PetriNet::import_from_path("example_data/artificialSmall_0.pnml").unwrap();
    let language = Language::generate(&net, 20);

    println!("Found {} unique traces:", language.traces.len());
    for trace in &language.traces {
        println!("  {:?}", trace);
    }
}