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
use petricheck::{model::{label::PetriTransitionLabel, marking::Marking, net::PetriNet, transition::PetriTransition}, reduction::reduce::reduce_petri_net};

use crate::{model::{ diagram::Diagram, event::EventType, id::BpmnId}, petri::{error::BpmnToPetriTranslationError, initial_marking::get_initial_marking_from_initial_places, subprocess::sub_process_to_petri}};




pub struct BpmnToPetriRetVal {
    pub petri_net : PetriNet,
    pub initial_marking : Marking
}

impl BpmnToPetriRetVal {
    pub fn new(petri_net: PetriNet, initial_marking: Marking) -> Self {
        Self { petri_net, initial_marking }
    }
}




/** 
 * Translates a BPMN diagram to a Petri Net, relabelling events/tasks/etc names and reducing 
 * as the PN is generated to ensure getting a small PN
 * **/
pub fn bpmn_to_petri(
    bpmn : &Diagram,
    transitions_labelling : &HashMap<BpmnId, Option<Rc<PetriTransitionLabel>>>
) -> Result<BpmnToPetriRetVal,BpmnToPetriTranslationError> {
    let mut petri_net = PetriNet::new_empty();
    let mut initial_places = HashSet::new();
    for process in bpmn.top_level_processes.values() {
        let petri_part = sub_process_to_petri(
            bpmn, 
            true,
            &process.content,
            transitions_labelling
        )?;
        let (places_shift,_) = petri_net.integrate_sub_net(&petri_part.petri_net);
        for init_place in petri_part.as_top_level() {
            initial_places.insert(init_place + places_shift);
        }
    }
    // *** 
    let mut throw_evts_outgoing_places = HashMap::new();
    let mut catch_evts_incoming_places = HashMap::new();
    for (place_id,opt_place_label) in petri_net.places.iter().enumerate() {
        if let Some(place_label) = opt_place_label {
            // we have labelled only throw and catch events
            let evt_bpmn_id = BpmnId { id: place_label.label.clone() };
            let evt = bpmn.events.get(&evt_bpmn_id).unwrap();
            match evt.event_type {
                EventType::IntermediateCatch => {
                    catch_evts_incoming_places.insert(evt_bpmn_id, place_id);
                },
                EventType::IntermediateThrow => {
                    throw_evts_outgoing_places.insert(evt_bpmn_id, place_id);
                },
                _ => {
                    panic!("should not occur")
                },
            }
        }
    }
    // ***
    for msg_flow in bpmn.message_flows.values() {
        let origin_place_id = *throw_evts_outgoing_places.get(&msg_flow.source_ref)
            .ok_or(BpmnToPetriTranslationError::MessageFlowSourceIsNotAnIntermediateThrowEvent)?;
        let target_place_id = *catch_evts_incoming_places.get(&msg_flow.target_ref)
            .ok_or(BpmnToPetriTranslationError::MessageFlowTargetIsNotAnIntermediateCatchEvent)?;
        let tx = PetriTransition::new(None,hash_map! {origin_place_id=>1}, hash_map! {target_place_id=>1});
        petri_net.add_transition(tx);
    }
    // The throw/catch event place labels were only needed to locate the places for message-flow
    // wiring. Strip them now so the series-place reduction can merge connected pairs.
    for place_id in throw_evts_outgoing_places.values().chain(catch_evts_incoming_places.values()) {
        petri_net.places[*place_id] = None;
    }

    let mut initial_marking = Some(get_initial_marking_from_initial_places(&initial_places));
    reduce_petri_net(&mut petri_net, &mut initial_marking);

    
    Ok(BpmnToPetriRetVal::new(petri_net,initial_marking.unwrap()))
}



