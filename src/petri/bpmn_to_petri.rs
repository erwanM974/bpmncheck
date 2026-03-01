/*
Copyright 2025 Erwan Mahe (github.com/erwanM974)

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/


use std::{collections::{HashMap, HashSet}, rc::Rc};
use map_macro::{hash_map};
use petricheck::{model::{label::PetriTransitionLabel, net::PetriNet, transition::PetriTransition}};

use crate::{model::{ diagram::Diagram, id::BpmnId}, petri::{error::BpmnToPetriTranslationError, subprocess::sub_process_to_petri}};




pub struct BpmnToPetriRetVal {
    pub petri_net : PetriNet,
    pub initial_places : HashSet<usize>,
    pub bpmn_id_to_transitions_labels : HashMap<BpmnId,Rc<PetriTransitionLabel>>
}

impl BpmnToPetriRetVal {
    pub fn new(petri_net: PetriNet, initial_places: HashSet<usize>, bpmn_id_to_transitions_labels: HashMap<BpmnId,Rc<PetriTransitionLabel>>) -> Self {
        Self { petri_net, initial_places, bpmn_id_to_transitions_labels }
    }
}





/** 
 * Translates a BPMN diagram to a Petri Net, relabelling events/tasks/etc names and reducing 
 * as the PN is generated to ensure getting a small PN
 * **/
pub fn bpmn_to_petri(
    bpmn : &Diagram,
) -> Result<BpmnToPetriRetVal,BpmnToPetriTranslationError> {
    let mut petri_net = PetriNet::new_empty();
    let mut initial_places = HashSet::new();
    let mut bpmn_id_to_incoming_place : HashMap<BpmnId,usize> = HashMap::new();
    let mut bpmn_id_to_outgoing_place : HashMap<BpmnId,usize> = HashMap::new();
    let mut bpmn_id_to_transitions_labels = HashMap::new();
    for process in bpmn.top_level_processes.values() {
        let petri_part = sub_process_to_petri(
            bpmn, 
            &process.content,
        )?;
        let (places_shift,_) = petri_net.integrate_sub_net(&petri_part.petri_net);
        initial_places.insert(petri_part.initial_place + places_shift);
        bpmn_id_to_incoming_place.extend(
            petri_part.bpmn_id_to_incoming_place.into_iter().map(|(x,y)| (x,y+places_shift))
        );
        bpmn_id_to_outgoing_place.extend(
            petri_part.bpmn_id_to_outgoing_place.into_iter().map(|(x,y)| (x,y+places_shift))
        );
        bpmn_id_to_transitions_labels.extend(
            petri_part.bpmn_id_to_transitions_labels
        );
    }
    for msg_flow in bpmn.message_flows.values() {
        let origin = bpmn_id_to_outgoing_place.get(&msg_flow.source_ref).unwrap();
        let target = bpmn_id_to_incoming_place.get(&msg_flow.target_ref).unwrap();
        initial_places.remove(target);
        let tx = PetriTransition::new(None,hash_map! {*origin=>1}, hash_map! {*target=>1});
        petri_net.add_transition(tx);
    }
    
    Ok(BpmnToPetriRetVal::new(petri_net,initial_places,bpmn_id_to_transitions_labels))
}



