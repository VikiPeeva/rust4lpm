use process_mining::core::process_models::petri_net::{ArcType, TransitionID};
use process_mining::PetriNet;
use std::ops::Deref;

pub struct TransitionBorderedPetriNet {
    net: PetriNet,
    pub source_transitions: Vec<TransitionID>,
    pub sink_transitions: Vec<TransitionID>,
}

impl TryFrom<PetriNet> for TransitionBorderedPetriNet {
    type Error = &'static str;

    fn try_from(net: PetriNet) -> Result<Self, Self::Error> {
        if !is_transition_bordered(&net) {
            return Err("Petri net is not transition-bordered: some place has no incoming or outgoing arc");
        }
        let input_transitions = find_input_transitions(&net);
        let output_transitions = find_output_transitions(&net);
        Ok(Self { net, source_transitions: input_transitions, sink_transitions: output_transitions })
    }
}

fn is_transition_bordered(net: &PetriNet) -> bool {
    net.places.keys().all(|p_uuid| {
        let has_incoming = net
            .arcs
            .iter()
            .any(|arc| matches!(arc.from_to, ArcType::TransitionPlace(_, to) if &to == p_uuid));
        let has_outgoing = net
            .arcs
            .iter()
            .any(|arc| matches!(arc.from_to, ArcType::PlaceTransition(from, _) if &from == p_uuid));
        has_incoming && has_outgoing
    })
}

fn find_input_transitions(net: &PetriNet) -> Vec<TransitionID> {
    net.transitions
        .keys()
        .filter(|t| !net.arcs.iter().any(|arc| matches!(arc.from_to, ArcType::PlaceTransition(_, to) if &to == *t)))
        .map(|t| TransitionID(*t))
        .collect()
}

fn find_output_transitions(net: &PetriNet) -> Vec<TransitionID> {
    net.transitions
        .keys()
        .filter(|t| !net.arcs.iter().any(|arc| matches!(arc.from_to, ArcType::TransitionPlace(from, _) if &from == *t)))
        .map(|t| TransitionID(*t))
        .collect()
}

impl TransitionBorderedPetriNet {
    pub fn into_inner(self) -> PetriNet {
        self.net
    }
}

impl Deref for TransitionBorderedPetriNet {
    type Target = PetriNet;
    fn deref(&self) -> &Self::Target {
        &self.net
    }
}