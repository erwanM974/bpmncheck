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

use map_macro::hash_map;
use petricheck::model::{label::PetriTransitionLabel, net::PetriNet, transition::PetriTransition};

use crate::{model::{activity::ActivityType, diagram::{Diagram, ProcessContentRef}, event::EventType, id::BpmnId}, petri::error::BpmnToPetriTranslationError};




pub struct Bpmn2PetriSubProcessRetVal {
    // the ids of the places in the Petri Net that correspond to the origins of the BPMN sub-process' start events
    pub initial_places : HashSet<usize>,
    // the ids of the places in the Petri Net that correspond to the targets of the BPMN sub-process' end events
    pub final_places   : HashSet<usize>,
}

impl Bpmn2PetriSubProcessRetVal {
    fn new(initial_places: HashSet<usize>, final_places: HashSet<usize>) -> Self {
        Self { initial_places, final_places }
    }
}



pub fn sub_process_to_petri(
    petri_net : &mut PetriNet,
    // maps bpmn_id of events and activities to the place in the Petri Net from which it starts
    bpmn_id_to_incoming_place : &mut HashMap<BpmnId,usize>, 
    // maps bpmn_id of events and activities to the place in the Petri Net at which it ends
    bpmn_id_to_outgoing_place : &mut HashMap<BpmnId,usize>, 
    // maps bpmn_id of events and activities to reference to the transition labels in the Petri Net
    bpmn_id_to_transitions_labels : &mut HashMap<BpmnId,Rc<PetriTransitionLabel>>,
    bpmn : &Diagram, 
    sub_process : &ProcessContentRef
) -> Result<Bpmn2PetriSubProcessRetVal,BpmnToPetriTranslationError> {
    let mut initial_places = HashSet::new();
    let mut final_places = HashSet::new();
    // ***
    for evt_id in &sub_process.direct_child_events {
        let evt = bpmn.events.get(evt_id).unwrap();
        let incoming = petri_net.add_place(None);
        if evt.event_type == EventType::Start {
            initial_places.insert(incoming);
        }
        let outgoing = petri_net.add_place(None);
        if evt.event_type == EventType::End {
            final_places.insert(outgoing);
        }
        let transition_label = Rc::new(PetriTransitionLabel::new(evt_id.id.clone()));
        bpmn_id_to_transitions_labels.insert(evt_id.clone(),transition_label.clone());
        let _ = petri_net.add_transition(
            PetriTransition::new(
                Some(transition_label),
                hash_map!{incoming=>1}, 
                hash_map!{outgoing=>1}
            )
        );
        bpmn_id_to_incoming_place.insert(evt_id.clone(), incoming);
        bpmn_id_to_outgoing_place.insert(evt_id.clone(), outgoing);
    }
    // translate each BPMN activity into a subnet "(incoming) -**activity**> (outgoing)"
    // where **activity** is either:
    // - a Petri net transition if the activity is not a sub-process
    // - a sub_net if the activity is a sub-process
    for act_id in &sub_process.direct_child_activities {
        let act = bpmn.activities.get(act_id).unwrap();
        if let ActivityType::SubProcess(sub_proc) = &act.activity_type {
            let sub_proc_ret_val = sub_process_to_petri(
                petri_net,
                bpmn_id_to_incoming_place,
                bpmn_id_to_outgoing_place,
                bpmn_id_to_transitions_labels, 
                bpmn, 
                sub_proc
            )?;
            match sub_proc_ret_val.initial_places.len() {
                0 => {
                    return Err(BpmnToPetriTranslationError::SubProcessMustHaveOneStartEvent);
                },
                1 => {
                    // if the sub-process has a single start event, 
                    // we can coincide the place that represent the origin of this start event
                    // with the incoming place of the overall activity
                    let initial_place_id = sub_proc_ret_val.initial_places.iter().next().unwrap();
                    bpmn_id_to_incoming_place.insert(act_id.clone(),*initial_place_id);
                },
                _ => {
                    // if the sub-process has several start events
                    // we create an additional place
                    // and a transition that feeds tokens into each of the place corresponding to the origins of these start events
                    let new_place = petri_net.add_place(None);
                    bpmn_id_to_incoming_place.insert(act_id.clone(),new_place);
                    let mut to_places = HashMap::new();
                    for p_id in sub_proc_ret_val.initial_places {
                        to_places.insert(p_id,1);
                    }
                    let tx = PetriTransition::new(None,hash_map!{new_place=>1}, to_places);
                    petri_net.add_transition(tx);
                }
            }
            match sub_proc_ret_val.final_places.len() {
                0 => {
                    return Err(BpmnToPetriTranslationError::SubProcessMustHaveOneEndEvent);
                },
                1 => {
                    // if the sub-process has a single end event, 
                    // we can coincide the place that represent the target of this end event
                    // with the outgoing place of the overall activity
                    let final_place_id = sub_proc_ret_val.final_places.iter().next().unwrap();
                    bpmn_id_to_outgoing_place.insert(act_id.clone(),*final_place_id);
                },
                _ => {
                    // if the sub-process has several end events
                    // we create an additional place
                    // and a transition that accepts tokens from each of the place corresponding to the targets of these end events
                    let new_place = petri_net.add_place(None);
                    bpmn_id_to_outgoing_place.insert(act_id.clone(),new_place);
                    let mut from_places = HashMap::new();
                    for p_id in sub_proc_ret_val.final_places {
                        from_places.insert(p_id,1);
                    }
                    let tx = PetriTransition::new(None,from_places, hash_map!{new_place=>1});
                    petri_net.add_transition(tx);
                }
            }
        } else {
            let incoming = petri_net.add_place(None);
            let outgoing = petri_net.add_place(None);
            // ***
            let transition_label = Rc::new(PetriTransitionLabel::new(act_id.id.clone()));
            bpmn_id_to_transitions_labels.insert(act_id.clone(),transition_label.clone());
            let _: usize = petri_net.add_transition(
                PetriTransition::new(
                    Some(transition_label),
                    hash_map!{incoming=>1}, 
                    hash_map!{outgoing=>1}
                )
            );
            // ***
            bpmn_id_to_incoming_place.insert(act_id.clone(), incoming);
            bpmn_id_to_outgoing_place.insert(act_id.clone(), outgoing);
        }
    }
    // 
    let ret_val = Bpmn2PetriSubProcessRetVal::new(
        initial_places, 
        final_places
    );
    Ok(ret_val)
}

