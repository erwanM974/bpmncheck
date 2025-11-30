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


use std::{collections::{BTreeMap, BTreeSet, HashMap, HashSet}, rc::Rc};
use itertools::Itertools;
use map_macro::{btree_set, hash_map};
use petricheck::model::{label::PetriTransitionLabel, marking::Marking, net::PetriNet, transition::PetriTransition};

use crate::{model::{ diagram::{Diagram}, gateway::GatewayType, id::BpmnId}, petri::{error::BpmnToPetriTranslationError, subprocess::sub_process_to_petri}};




pub struct BpmnToPetriRetVal {
    pub petri_net : PetriNet,
    pub initial_places : HashSet<usize>,
    pub bpmn_id_to_transitions_labels : HashMap<BpmnId,Rc<PetriTransitionLabel>>
}

impl BpmnToPetriRetVal {
    pub fn new(petri_net: PetriNet, initial_places: HashSet<usize>, bpmn_id_to_transitions_labels: HashMap<BpmnId,Rc<PetriTransitionLabel>>) -> Self {
        Self { petri_net, initial_places, bpmn_id_to_transitions_labels }
    }

    pub fn get_initial_marking(&self) -> Marking {
        let mut tokens = BTreeMap::new();
        for place_id in &self.initial_places {
            tokens.insert(*place_id,1);
        }
        Marking::new(tokens)
    }

}





pub fn bpmn_to_petri(
    bpmn : &Diagram
) -> Result<BpmnToPetriRetVal,BpmnToPetriTranslationError> {
    let mut petri_net = PetriNet::new_empty();
    let mut initial_places = HashSet::new();
    let mut bpmn_id_to_incoming_place = HashMap::new();
    let mut bpmn_id_to_outgoing_place = HashMap::new();
    let mut bpmn_id_to_transitions_labels = HashMap::new();
    for process in bpmn.top_level_processes.values() {
        let petri_part = sub_process_to_petri(
            &mut petri_net, 
            &mut bpmn_id_to_incoming_place,
            &mut bpmn_id_to_outgoing_place,
            &mut bpmn_id_to_transitions_labels,
            bpmn, 
            &process.content
        )?;
        for part_initial_place in petri_part.initial_places {
            initial_places.insert(part_initial_place);
        }
    }
    let mut gateways_inputs : HashMap<BpmnId, BTreeSet<usize>> = HashMap::new();
    let mut gateways_outputs : HashMap<BpmnId, BTreeSet<usize>> = HashMap::new();
    for flow in bpmn.flows.values() {
        let source_is_gate = bpmn.gateways.contains_key(&flow.source_ref);
        let target_is_gate = bpmn.gateways.contains_key(&flow.target_ref);
        match (source_is_gate,target_is_gate) {
            (false, false) => {
                // both source and target of the flow are not gates
                // thus they already have corresponding outgoing and incoming places in the net
                // we then simply add the corresponding transition
                let origin = bpmn_id_to_outgoing_place.get(&flow.source_ref).unwrap();
                let target = bpmn_id_to_incoming_place.get(&flow.target_ref).unwrap();
                initial_places.remove(target);
                let tx = PetriTransition::new(None,hash_map! {*origin=>1}, hash_map! {*target=>1});
                petri_net.add_transition(tx);
            },
            (true, false) => {
                // the source of the flow is a gate and its target is not
                // the target already has a corresponding incoming place in the net
                // we therefore add to our "gateways_output" the relevant information
                let target = bpmn_id_to_incoming_place.get(&flow.target_ref).unwrap();
                initial_places.remove(target);
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
    for (gate_id,gate) in &bpmn.gateways {

        let gate_transition_label = Rc::new(PetriTransitionLabel::new(gate_id.id.clone()));
        bpmn_id_to_transitions_labels.insert(gate_id.clone(),gate_transition_label.clone());

        let inputs = gateways_inputs.get(gate_id).unwrap();
        let outputs = gateways_outputs.get(gate_id).unwrap();
        match gate.gateway_type {
            GatewayType::Parallel => {
                // there is a single transition 
                // from all input places
                // to all output places
                let _ = petri_net.add_transition(
                    PetriTransition::new(
                        Some(gate_transition_label.clone()),
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
                                Some(gate_transition_label.clone()),
                                hash_map!{*input_place=>1}, 
                                hash_map! {*output_place=>1}
                            )
                        );
                        tr_ids.insert(tr_id);
                    }
                }
            },
            GatewayType::Inclusive => {
                // for each subset X of the inputs and each subset Y of the outputs
                // there is a transition X->Y
                let inputs_combinations = inputs.iter().powerset().collect::<Vec<_>>();
                let outputs_combinations = outputs.iter().powerset().collect::<Vec<_>>();
                let mut tr_ids = HashSet::new();
                for input_subset in inputs_combinations {
                    for output_subset in outputs_combinations.clone() {
                        let tr_id = petri_net.add_transition(
                            PetriTransition::new(
                                Some(gate_transition_label.clone()),
                                input_subset.clone().into_iter().cloned().map(|x| (x,1)).collect(), 
                                output_subset.into_iter().cloned().map(|x| (x,1)).collect(),
                                )
                        );
                        tr_ids.insert(tr_id);
                    }
                }
            }
        };
    }
    Ok(BpmnToPetriRetVal::new(petri_net,initial_places,bpmn_id_to_transitions_labels))
}



