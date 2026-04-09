use process_mining::PetriNet;
use std::collections::HashSet;

pub struct Language {
    pub traces: HashSet<Vec<String>>,
}

impl Language {
    pub fn generate(net: &PetriNet, max_steps: usize) -> Self {
        Self {
            traces: crate::core::models::petri_net::language::generate_language(net, max_steps)
                .into_iter()
                .collect(),
        }
    }
}