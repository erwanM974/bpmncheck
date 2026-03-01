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



use std::{collections::HashMap, rc::Rc};

use map_macro::hash_map;
use petricheck::model::{label::PetriTransitionLabel, net::PetriNet, transition::PetriTransition};

use crate::model::id::BpmnId;
use crate::petri::{error::BpmnToPetriTranslationError, subprocess::Bpmn2PetriSubProcessRetVal};



pub fn nest_sub_process(
    petri_net : &mut PetriNet,
    // maps bpmn_id of events and activities to the place in the Petri Net from which it starts
    bpmn_id_to_incoming_place : &mut HashMap<BpmnId,usize>, 
    // maps bpmn_id of events and activities to the place in the Petri Net at which it ends
    bpmn_id_to_outgoing_place : &mut HashMap<BpmnId,usize>, 
    // maps bpmn_id of events and activities to reference to the transition labels in the Petri Net
    bpmn_id_to_transitions_labels : &mut HashMap<BpmnId,Rc<PetriTransitionLabel>>,
    act_id : &BpmnId,
    //sub_proc : &ProcessContentRef,
    sub_proc_ret_val : Bpmn2PetriSubProcessRetVal,
    boundary_event : Option<BpmnId>
) -> Result<(),BpmnToPetriTranslationError> {
    // ***

    // **
    let (places_shift,transitions_shift) = petri_net.integrate_sub_net(&sub_proc_ret_val.petri_net);
    bpmn_id_to_incoming_place.extend(
        sub_proc_ret_val.bpmn_id_to_incoming_place.into_iter().map(|(x,y)| (x,y+places_shift))
    );
    bpmn_id_to_outgoing_place.extend(
        sub_proc_ret_val.bpmn_id_to_outgoing_place.into_iter().map(|(x,y)| (x,y+places_shift))
    );
    bpmn_id_to_transitions_labels.extend(
        sub_proc_ret_val.bpmn_id_to_transitions_labels
    );
    // ***
    bpmn_id_to_incoming_place.insert(act_id.clone(),sub_proc_ret_val.initial_place + places_shift);
    bpmn_id_to_outgoing_place.insert(act_id.clone(),sub_proc_ret_val.final_place + places_shift);
    if let Some(boundary) = boundary_event {
        // cf page 10 in https://www.researchgate.net/publication/27467826_Formal_Semantics_and_Automated_Analysis_of_BPMN_Process_Models 
        let ok_flag_place = petri_net.add_place(None);
        let sub_proc_start_tx_id = sub_proc_ret_val.initial_transition + transitions_shift;
        let sub_proc_end_tx_id = sub_proc_ret_val.final_transition + transitions_shift;
        // the start transition of the subprocess needs to activate the "ok_flag_place"
        {
            let subproc_start_tx = petri_net.transitions.get_mut(sub_proc_start_tx_id).unwrap();
            subproc_start_tx.postset_tokens.insert(ok_flag_place,1); 
        }
        let nok_flag_place = petri_net.add_place(None);
        {
            // the transition between the "ok" and "nok" corresponds to the occurrence of the exception 
            let exception_transition_label = Rc::new(PetriTransitionLabel::new(boundary.id.clone()));
            petri_net.add_transition(
                PetriTransition::new(
                    Some(exception_transition_label.clone()),
                    hash_map! {ok_flag_place=>1}, 
                    hash_map! {nok_flag_place=>1}
                )
            );
            bpmn_id_to_transitions_labels.insert(boundary.clone(), exception_transition_label);
        }
        // if the "nok" place is active, every transition (other than the initial and final) in the subprocess can be "skipped"
        for unshifted_tx_id in 0..sub_proc_ret_val.petri_net.transitions.len() {
            let tx_id = unshifted_tx_id + transitions_shift;
            if tx_id != sub_proc_start_tx_id && tx_id != sub_proc_end_tx_id {
                let original_subproc_tx = petri_net.transitions.get(tx_id).unwrap();
                // we create an alternative transition 
                // so that the original can be skipped if the "nok" place is active
                let mut preset = original_subproc_tx.preset_tokens.clone();
                preset.insert(nok_flag_place,1); 
                let mut postset = original_subproc_tx.postset_tokens.clone();
                postset.insert(nok_flag_place,1); 
                petri_net.add_transition(
                    PetriTransition::new(
                        None,
                        preset, 
                        postset
                    )
                );
            }
        }
        // we add a place corresponding to an alternative final place of the subprocess
        // in case the exception occurred
        let after_exception_place = petri_net.add_place(None);
        // this place is linked by a transition that accept tokens from:
        // - the nok place
        // - and all the preset places of the subprocess's final transition
        {
            let subproc_end_tx = petri_net.transitions.get(sub_proc_end_tx_id).unwrap();
            let mut preset = subproc_end_tx.preset_tokens.clone();
            preset.insert(nok_flag_place,1);
            petri_net.add_transition(
                PetriTransition::new(
                    None,
                    preset, 
                    hash_map! {after_exception_place=>1}
                )
            );
        }
        // the end transition of the subprocess needs to take tokens from the "ok_flag_place"
        {
            let subproc_end_tx = petri_net.transitions.get_mut(sub_proc_end_tx_id).unwrap();
            subproc_end_tx.preset_tokens.insert(ok_flag_place,1); 
        }
        // all the other transitions inside the subprocess require the "ok_flag_place" to have one token 
        for unshifted_tx_id in 0..sub_proc_ret_val.petri_net.transitions.len() {
            let tx_id = unshifted_tx_id + transitions_shift;
            if tx_id != sub_proc_start_tx_id && tx_id != sub_proc_end_tx_id {
                let subproc_tx = petri_net.transitions.get_mut(tx_id).unwrap();
                subproc_tx.preset_tokens.insert(ok_flag_place,1); 
                subproc_tx.postset_tokens.insert(ok_flag_place,1); 
            }
        }
        // finally we update the map of bpmnid to outgoing place
        bpmn_id_to_outgoing_place.insert(boundary,after_exception_place);
    }
    Ok(())
}




