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



use std::{collections::{BTreeSet, HashMap, HashSet}, rc::Rc};

use itertools::Itertools;
use map_macro::{btree_set, hash_map};
use petricheck::{model::{label::{PetriStateLabel, PetriTransitionLabel}, net::PetriNet, transition::PetriTransition}, reduction::reduce::reduce_petri_net};

use crate::{model::{activity::ActivityType, diagram::{Diagram, ProcessContentRef}, event::EventType, gateway::GatewayType, id::BpmnId}, petri::{error::BpmnToPetriTranslationError, initial_marking::get_initial_marking_from_initial_places, nesting::nest_sub_process}};




pub struct Bpmn2PetriSubProcessRetVal {
    // the PN that is the translation of the BPMN subprocess
    // its initial and final places are the first and last places
    pub petri_net : PetriNet,
    pub kind : Bpmn2PetriProcessRetValKind
}

impl Bpmn2PetriSubProcessRetVal {
    pub fn as_top_level(self) -> HashSet<usize> {
        match self.kind {
            Bpmn2PetriProcessRetValKind::TopLevelProcess { initial_places } => initial_places,
            Bpmn2PetriProcessRetValKind::SubProcess(_) => panic!(),
        }
    }
}




pub enum Bpmn2PetriProcessRetValKind {
    TopLevelProcess{
        initial_places : HashSet<usize>
    },
    SubProcess(Bpmn2PetriSubProcessStartEndInfo)
}


pub struct Bpmn2PetriSubProcessStartEndInfo {
    pub initial_place : usize,
    pub initial_transition : usize,
    pub final_place : usize,
    pub final_transition : usize
}


pub fn sub_process_to_petri(
    bpmn : &Diagram, 
    // if the process is top-level, no need to have start/end events
    is_top_level : bool,
    sub_process : &ProcessContentRef,
    transitions_labelling : &HashMap<BpmnId, Option<Rc<PetriTransitionLabel>>>
) -> Result<Bpmn2PetriSubProcessRetVal,BpmnToPetriTranslationError> {
    let mut petri_net = PetriNet::new_empty();
    let mut bpmn_id_to_incoming_place = HashMap::new();
    let mut bpmn_id_to_outgoing_place = HashMap::new();
    // ***
    // add places for all events and extract pertinent information
    let (initial_places,final_places,mut boundary_events_on_sub_processes) = translate_events(
        bpmn,
        is_top_level,
        transitions_labelling,
        sub_process,
        &mut petri_net,
        &mut bpmn_id_to_incoming_place,
        &mut bpmn_id_to_outgoing_place
    )?;
    // ***
    if !is_top_level {
        // ***
        // add the initial and final places
        let initial_place = petri_net.add_place(Some(Rc::new(PetriStateLabel::new("initial".to_string()))));
        let final_place = petri_net.add_place(Some(Rc::new(PetriStateLabel::new("final".to_string()))));
        // add a transition from the initial place to all the possible multiple initial places
        let _ = petri_net.add_transition(
            PetriTransition::new(
                Some(Rc::new(PetriTransitionLabel::new("initial".to_string()))),
                hash_map!{initial_place=>1}, 
                initial_places.iter().map(|p| (*p,1)).collect()
            )
        );
        // ***
        // add a transition from all the possible multiple final places to the final place
        let _ = petri_net.add_transition(
            PetriTransition::new(
                Some(Rc::new(PetriTransitionLabel::new("final".to_string()))),
                final_places.into_iter().map(|p| (p,1)).collect(),
                hash_map!{final_place=>1}, 
            )
        );
    }
    

    // translate each BPMN activity into a subnet "(incoming) -**activity**> (outgoing)"
    // where **activity** is either:
    // - a Petri net transition if the activity is not a sub-process
    // - a sub_net if the activity is a sub-process
    for act_id in &sub_process.direct_child_activities {
        let act = bpmn.activities.get(act_id).unwrap();
        if let ActivityType::SubProcess(sub_proc) = &act.activity_type {
            let sub_proc_ret_val: Bpmn2PetriSubProcessRetVal = sub_process_to_petri(
                bpmn, 
                false,
                sub_proc,
                transitions_labelling
            )?;
            let boundary_event = boundary_events_on_sub_processes.remove(act_id);
            let Bpmn2PetriSubProcessRetVal { petri_net: sub_proc_pn, kind } = sub_proc_ret_val;
            let sub_proc_start_end_info = match kind {
                Bpmn2PetriProcessRetValKind::SubProcess(info) => info,
                Bpmn2PetriProcessRetValKind::TopLevelProcess { .. } => unreachable!(),
            };
            nest_sub_process(
                transitions_labelling,
                &mut petri_net,
                &mut bpmn_id_to_incoming_place,
                &mut bpmn_id_to_outgoing_place,
                act_id,
                sub_proc_pn,
                sub_proc_start_end_info,
                boundary_event
            )?;
        } else {
            let incoming = petri_net.add_place(None);
            let outgoing = petri_net.add_place(None);
            // ***
            let transition_label = transitions_labelling.get(act_id).unwrap().clone();
            let _: usize = petri_net.add_transition(
                PetriTransition::new(
                    transition_label,
                    hash_map!{incoming=>1}, 
                    hash_map!{outgoing=>1}
                )
            );
            // ***
            bpmn_id_to_incoming_place.insert(act_id.clone(), incoming);
            bpmn_id_to_outgoing_place.insert(act_id.clone(), outgoing);
        }
    }


    let (gateways_inputs,gateways_outputs) = translate_sequence_flows(
        bpmn, 
        sub_process, 
        &mut petri_net, 
        bpmn_id_to_incoming_place, 
        bpmn_id_to_outgoing_place
    );
    
    for gate_id in &sub_process.direct_child_gateways {
        let gate = bpmn.gateways.get(gate_id).unwrap();
        let gate_transition_label = transitions_labelling.get(gate_id).unwrap().clone();

        let inputs = gateways_inputs.get(gate_id).unwrap();
        let outputs = gateways_outputs.get(gate_id).unwrap();
        match gate.gateway_type {
            GatewayType::Parallel => {
                // there is a single transition 
                // from all input places
                // to all output places
                let _ = petri_net.add_transition(
                    PetriTransition::new(
                        gate_transition_label,
                        inputs.iter().cloned().map(|x| (x,1)).collect(), 
                        outputs.iter().cloned().map(|x| (x,1)).collect(), 
                    )
                );
            },
            GatewayType::Exclusive => {
                // for each pair (input,output) there is a transition input->output
                let mut tr_ids = HashSet::new();
                for input_place in inputs {
                    for output_place in outputs {
                        let tr_id = petri_net.add_transition(
                            PetriTransition::new(
                                gate_transition_label.clone(),
                                hash_map!{*input_place=>1}, 
                                hash_map! {*output_place=>1}
                            )
                        );
                        tr_ids.insert(tr_id);
                    }
                }
            },
            GatewayType::Inclusive => {
                // for each non-empty subset X of the inputs and each non-empty subset Y of the outputs
                // there is a transition X->Y
                let inputs_combinations = inputs.iter().powerset().collect::<Vec<_>>();
                let outputs_combinations = outputs.iter().powerset().collect::<Vec<_>>();
                let mut tr_ids = HashSet::new();
                for input_subset in inputs_combinations {
                    if !input_subset.is_empty() {
                        for output_subset in outputs_combinations.clone() {
                            if !output_subset.is_empty() {
                                let tr_id = petri_net.add_transition(
                                    PetriTransition::new(
                                        gate_transition_label.clone(),
                                        input_subset.clone().into_iter().cloned().map(|x| (x,1)).collect(), 
                                        output_subset.into_iter().cloned().map(|x| (x,1)).collect(),
                                        )
                                );
                                tr_ids.insert(tr_id);
                            }
                        }
                    }
                }
            }
        };
    }

    let mut initial_marking = Some(get_initial_marking_from_initial_places(&initial_places));
    reduce_petri_net(&mut petri_net, &mut initial_marking);

    let ret_val = if !is_top_level {
        // The pre-reduction indices are stale after reduce_petri_net may have shifted places.
        // Recover the correct post-reduction indices by searching for the sentinel labels,
        // then strip those labels so they don't conflict when this sub-net is integrated into a parent.
        let initial_place = petri_net.places.iter().position(|p| {
            p.as_ref().map_or(false, |l| l.label == "initial")
        }).unwrap();
        let final_place = petri_net.places.iter().position(|p| {
            p.as_ref().map_or(false, |l| l.label == "final")
        }).unwrap();
        let initial_transition = petri_net.transitions.iter().position(|t| {
            t.transition_label.as_ref().map_or(false, |l| l.label == "initial")
        }).unwrap();
        let final_transition = petri_net.transitions.iter().position(|t| {
            t.transition_label.as_ref().map_or(false, |l| l.label == "final")
        }).unwrap();
        petri_net.places[initial_place] = None;
        petri_net.places[final_place] = None;
        petri_net.transitions[initial_transition].transition_label = None;
        petri_net.transitions[final_transition].transition_label = None;

        Bpmn2PetriSubProcessRetVal{
            petri_net,
            kind: Bpmn2PetriProcessRetValKind::SubProcess(Bpmn2PetriSubProcessStartEndInfo {
                initial_place,
                initial_transition,
                final_place,
                final_transition,
            })
        }
    } else {
        // Extract post-reduction initial places from the updated marking;
        // the pre-reduction `initial_places` set has stale indices after reduce_petri_net.
        let post_reduction_initial_places: HashSet<usize> = initial_marking
            .unwrap()
            .iter_tokens()
            .map(|(place_id, _)| *place_id)
            .collect();
        Bpmn2PetriSubProcessRetVal{
            petri_net,
            kind: Bpmn2PetriProcessRetValKind::TopLevelProcess {
                initial_places: post_reduction_initial_places,
            }
        }
    };
    
    Ok(ret_val)
}




fn translate_events(
    bpmn : &Diagram, 
    // if the process is top-level, no need to have start/end events
    is_top_level : bool,
    transitions_relabelling : &HashMap<BpmnId, Option<Rc<PetriTransitionLabel>>>,
    sub_process : &ProcessContentRef,
    petri_net : &mut PetriNet,
    bpmn_id_to_incoming_place : &mut HashMap<BpmnId,usize>, 
    bpmn_id_to_outgoing_place : &mut HashMap<BpmnId,usize>, 
) -> Result<(HashSet<usize>,HashSet<usize>,HashMap<BpmnId,BpmnId>),BpmnToPetriTranslationError> {

    let mut initial_places = HashSet::new();
    let mut final_places = HashSet::new();
    let mut boundary_events_on_sub_processes : HashMap<BpmnId,BpmnId> = HashMap::new();
    for evt_id in &sub_process.direct_child_events {
        let evt = bpmn.events.get(evt_id).unwrap();
        if let EventType::Boundary(associated_sub_proc) = &evt.event_type {
            // we treat boundary events 
            // (that are exclusively used to model exception flows interrupting sub-processes)
            // differently
            if boundary_events_on_sub_processes.contains_key(&associated_sub_proc) {
                return Err(BpmnToPetriTranslationError::SubProcessCanHaveAtMostOneBoundaryEvent);
            }
            boundary_events_on_sub_processes.insert(associated_sub_proc.clone(), evt_id.clone());
        } else {
            // for the other event kinds,
            let (incoming_place_label,outgoing_place_label) = match evt.event_type {
                // for catch events, we keep track of the incoming places to be able to connect the message flows
                // even if the PN is reduced
                EventType::IntermediateCatch => {
                    let place_label = Rc::new(
                        PetriStateLabel::new(evt_id.id.clone())
                    );
                    (Some(place_label),None)
                },
                // for throw events, we keep track of the outgoing places to be able to connect the message flows
                // even if the PN is reduced
                EventType::IntermediateThrow => {
                    let place_label = Rc::new(
                        PetriStateLabel::new(evt_id.id.clone())
                    );
                    (None,Some(place_label))
                },
                _ => {
                    (None,None)
                }
            };
            // we crate two places (incoming and outgoing) and a transition between them
            let incoming = petri_net.add_place(incoming_place_label);
            if evt.event_type == EventType::Start {
                initial_places.insert(incoming);
            }
            let outgoing = petri_net.add_place(outgoing_place_label);
            if evt.event_type == EventType::End {
                final_places.insert(outgoing);
            }
            // ***
            // ***
            let transition_label = transitions_relabelling.get(&evt_id).unwrap().clone();
            let _ = petri_net.add_transition(
                PetriTransition::new(
                    transition_label,
                    hash_map!{incoming=>1}, 
                    hash_map!{outgoing=>1}
                )
            );
            bpmn_id_to_incoming_place.insert(evt_id.clone(), incoming);
            bpmn_id_to_outgoing_place.insert(evt_id.clone(), outgoing);
        }
    }

    if !is_top_level && initial_places.is_empty() {
        Err(BpmnToPetriTranslationError::SubProcessMustHaveOneStartEvent)
    } else if !is_top_level && final_places.is_empty() {
        Err(BpmnToPetriTranslationError::SubProcessMustHaveOneEndEvent)
    } else {
        Ok((initial_places,final_places,boundary_events_on_sub_processes))
    }
}





fn translate_sequence_flows(
    bpmn : &Diagram, 
    sub_process : &ProcessContentRef,
    petri_net : &mut PetriNet,
    bpmn_id_to_incoming_place : HashMap<BpmnId,usize>, 
    bpmn_id_to_outgoing_place : HashMap<BpmnId,usize>, 
) -> (HashMap<BpmnId, BTreeSet<usize>>,HashMap<BpmnId, BTreeSet<usize>>) {
    let mut gateways_inputs : HashMap<BpmnId, BTreeSet<usize>> = HashMap::new();
    let mut gateways_outputs : HashMap<BpmnId, BTreeSet<usize>> = HashMap::new();
    for flow_id in &sub_process.direct_child_flows {
        let flow = bpmn.sequence_flows.get(flow_id).unwrap();
        let source_is_gate = bpmn.gateways.contains_key(&flow.source_ref);
        let target_is_gate = bpmn.gateways.contains_key(&flow.target_ref);
        match (source_is_gate,target_is_gate) {
            (false, false) => {
                // both source and target of the flow are not gates
                // thus they already have corresponding outgoing and incoming places in the net
                // we then simply add the corresponding transition
                let origin = bpmn_id_to_outgoing_place.get(&flow.source_ref).unwrap();
                let target = bpmn_id_to_incoming_place.get(&flow.target_ref).unwrap();
                let tx = PetriTransition::new(None,hash_map! {*origin=>1}, hash_map! {*target=>1});
                petri_net.add_transition(tx);
            },
            (true, false) => {
                // the source of the flow is a gate and its target is not
                // the target already has a corresponding incoming place in the net
                // we therefore add to our "gateways_output" the relevant information
                let target = bpmn_id_to_incoming_place.get(&flow.target_ref).unwrap();
                gateways_outputs
                    .entry(flow.source_ref.clone())
                    .and_modify(|x| {x.insert(*target);})
                    .or_insert(btree_set!{*target});
            },
            (false, true) => {
                // similar to above case
                let source = bpmn_id_to_outgoing_place.get(&flow.source_ref).unwrap();
                gateways_inputs
                    .entry(flow.target_ref.clone())
                    .and_modify(|x| {x.insert(*source);})
                    .or_insert(btree_set!{*source});
            },
            (true, true) => {
                // both source and target of the flow are gates
                // we add an intermediate place to handle the complexity
                let new_place = petri_net.add_place(None);
                gateways_outputs
                    .entry(flow.source_ref.clone())
                    .and_modify(|x| {x.insert(new_place);})
                    .or_insert(btree_set!{new_place});
                gateways_inputs
                    .entry(flow.target_ref.clone())
                    .and_modify(|x| {x.insert(new_place);})
                    .or_insert(btree_set!{new_place});
            },
        }
    }
    (gateways_inputs,gateways_outputs)
}