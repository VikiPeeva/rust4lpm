use crate::core::models::petri_net::TransitionBorderedPetriNet;
use process_mining::core::process_models::petri_net::ArcType;
use process_mining::PetriNet;
use std::ops::Deref;

pub struct WorkflowNet {
    net: PetriNet,
}

impl TryFrom<TransitionBorderedPetriNet> for WorkflowNet {
    type Error = &'static str;

    fn try_from(tbpn: TransitionBorderedPetriNet) -> Result<Self, Self::Error> {
        let source_t = match tbpn.source_transitions.as_slice() {
            [t] => *t,
            [] => return Err("no source transition found"),
            _ => return Err("more than one source transition found"),
        };
        let sink_t = match tbpn.sink_transitions.as_slice() {
            [t] => *t,
            [] => return Err("no sink transition found"),
            _ => return Err("more than one sink transition found"),
        };

        let mut net = tbpn.into_inner();

        let source = net.add_place(None);
        net.add_arc(ArcType::place_to_transition(source, source_t), None);

        let sink = net.add_place(None);
        net.add_arc(ArcType::transition_to_place(sink_t, sink), None);

        Ok(Self { net })
    }
}

impl Deref for WorkflowNet {
    type Target = PetriNet;
    fn deref(&self) -> &Self::Target {
        &self.net
    }
}