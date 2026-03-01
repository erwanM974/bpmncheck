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

use std::collections::{HashMap, HashSet};

use crate::{model::{activity::ActivityType, diagram::{Diagram, ProcessContentRef}, event::EventType, id::BpmnId}, wellformedness::error::{ModelWellFormednessViolation, ProcessWellFormednessViolation}};



pub fn check_process_well_formedness(
    bpmn : &Diagram,
    proc_id : &BpmnId,
    proc_content : &ProcessContentRef
) -> Result<(),ModelWellFormednessViolation> {
    let mut number_of_start_events = 0;
    let mut number_of_end_events = 0;
    for evt_id in &proc_content.direct_child_events {
        let evt = bpmn.events.get(evt_id).unwrap();
        let (num_possible_incoming_flows,num_possible_outgoing_flows) = match evt.event_type {
            EventType::Start => {
                number_of_start_events += 1;
                // start, catch and boundary events (interpreted as non-error exception events) have:
                // - 0 incoming flows from other activities of the same subprocess 
                // - 1 outgoing flow to other activities of the same subprocess
                (vec![0],vec![1])
            },
            EventType::IntermediateCatch | EventType::Boundary(_)=> {
                (vec![0],vec![1])
            },
            EventType::End => {
                number_of_end_events += 1;
                (vec![1],vec![0])
            },
            EventType::IntermediateThrow => {
                (vec![1],vec![0])
            }
        };
        let num_actual_incoming_flows = proc_content.direct_child_flows.iter().filter(|flow_id| {
            let flow = bpmn.sequence_flows.get(*flow_id).unwrap();
            flow.target_ref == *evt_id
        }).collect::<Vec<_>>().len();
        if !num_possible_incoming_flows.contains(&num_actual_incoming_flows) {
            return Err(
                ModelWellFormednessViolation::ProcessViolation(
                    proc_id.clone(), 
                    ProcessWellFormednessViolation::WrongNumberOfIncomingFlows(evt_id.clone())
                )
            )
        }
        let num_actual_outgoing_flows = proc_content.direct_child_flows.iter().filter(|flow_id| {
            let flow = bpmn.sequence_flows.get(*flow_id).unwrap();
            flow.source_ref == *evt_id
        }).collect::<Vec<_>>().len();
        if !num_possible_outgoing_flows.contains(&num_actual_outgoing_flows) {
            return Err(
                ModelWellFormednessViolation::ProcessViolation(
                    proc_id.clone(), 
                    ProcessWellFormednessViolation::WrongNumberOfOutgoingFlows(evt_id.clone())
                )
            )
        }
    }

    if number_of_start_events < 1 {
        return Err(
            ModelWellFormednessViolation::ProcessViolation(
                proc_id.clone(), 
                ProcessWellFormednessViolation::ProcessHasNoStartEvent
            )
        );
    }
    if number_of_end_events < 1 {
        return Err(
            ModelWellFormednessViolation::ProcessViolation(
                proc_id.clone(), 
                ProcessWellFormednessViolation::ProcessHasNoEndEvent
            )
        );
    }

    for act_id in &proc_content.direct_child_activities {
        let act = bpmn.activities.get(act_id).unwrap();
        if let ActivityType::SubProcess(sub_proc_content) = &act.activity_type {
            if let Err(e) = check_process_well_formedness(bpmn,act_id,sub_proc_content) {
                return Err(e);
            }
        }
        let num_actual_incoming_flows = proc_content.direct_child_flows.iter().filter(|flow_id| {
            let flow = bpmn.sequence_flows.get(*flow_id).unwrap();
            flow.target_ref == *act_id
        }).collect::<Vec<_>>().len();
        if num_actual_incoming_flows != 1 {
            return Err(
                ModelWellFormednessViolation::ProcessViolation(
                    proc_id.clone(), 
                    ProcessWellFormednessViolation::WrongNumberOfIncomingFlows(act_id.clone())
                )
            )
        }
        let num_actual_outgoing_flows = proc_content.direct_child_flows.iter().filter(|flow_id| {
            let flow = bpmn.sequence_flows.get(*flow_id).unwrap();
            flow.source_ref == *act_id
        }).collect::<Vec<_>>().len();
        if num_actual_outgoing_flows != 1 {
            return Err(
                ModelWellFormednessViolation::ProcessViolation(
                    proc_id.clone(), 
                    ProcessWellFormednessViolation::WrongNumberOfOutgoingFlows(act_id.clone())
                )
            )
        }
    }

    Ok(())
}

pub fn check_model_well_formedness(
    bpmn : &Diagram
) -> Result<(),ModelWellFormednessViolation> {
    if bpmn.top_level_processes.is_empty() {
        return Err(ModelWellFormednessViolation::NoTopLevelProcess);
    }
    for (proc_id,proc) in &bpmn.top_level_processes {
        check_process_well_formedness(
            bpmn,
            proc_id,
            &proc.content,
        )?
    }
    let mut throw_to_catch : HashMap<BpmnId,BpmnId> = HashMap::new();
    let mut catched_catches = HashSet::new();
    // check that every throw/catch event does not have more than one outgoing/incoming message flow
    for msg_flow in bpmn.message_flows.values() {
        if throw_to_catch.contains_key(&msg_flow.source_ref) {
            return Err(
                ModelWellFormednessViolation::ThrowEventHasMoreThanOneSuccessor(msg_flow.source_ref.clone())
            );
        }
        if catched_catches.contains(&msg_flow.target_ref) {
            return Err(
                ModelWellFormednessViolation::CatchEventHasMoreThanOnePredecessor(msg_flow.target_ref.clone())
            );
        }
        throw_to_catch.insert(msg_flow.source_ref.clone(), msg_flow.target_ref.clone());
        catched_catches.insert(msg_flow.target_ref.clone());
    }
    // check that every throw/catch event has one outgoing/incoming message flow
    for (evt_id,evt) in bpmn.events.iter() {
        match evt.event_type {
            EventType::IntermediateCatch => {
                if !catched_catches.contains(evt_id) {
                    return Err(
                        ModelWellFormednessViolation::CatchEventHasNoPredecessor(evt_id.clone())
                    );
                }
            },
            EventType::IntermediateThrow => {
                if !throw_to_catch.contains_key(evt_id) {
                    return Err(
                        ModelWellFormednessViolation::ThrowEventHasNoSuccessor(evt_id.clone())
                    );
                }
            },
            _ => {
                // do nothing
            }
        }
    }
    Ok(())
}