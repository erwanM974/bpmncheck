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





use std::{collections::{BTreeSet, HashSet}, rc::Rc};

use bpmncheck::{parser::bpmn::read_bpmn_diagram_from_file_path, petri::bpmn_to_petri::bpmn_to_petri};
use citreelo::solve::is_ctl_formula_sat;
use map_macro::{btree_set, hash_set};
use petricheck::{model::label::PetriTransitionLabel, model_checking::to_kripke::{PetriKripkeGenerationSafenessRequirement, PetriKripkeStateProducer, petri_to_kripke}};


mod common;
use common::ctl_from_traces_existential::traces_to_ctl;


fn tool_test_bpmn_semantic_from_traces(
    bpmn_file_path : &str,
    traces: HashSet<Vec<String>>,
    alphabet: BTreeSet<String>,
) {
    let bpmn = read_bpmn_diagram_from_file_path(bpmn_file_path).unwrap();
    // ***
    let transitions_relabelling = bpmn.get_all_bpmn_ids().into_iter().map(|bpmnid| {
        let new_label = if let Some(act) = bpmn.activities.get(&bpmnid) && act.activity_type.is_task() {
            match &act.name {
                Some(x) => {
                    let new_tr_lab = PetriTransitionLabel::new(x.to_string());
                    Some(Rc::new(new_tr_lab))
                }
                None => None,
            }
        } else {
            None
        };
        (bpmnid, new_label)
    }).collect();

    // ***
    let petri_retval = bpmn_to_petri(&bpmn, &transitions_relabelling).unwrap();
    let tagged_transition_labels: HashSet<PetriTransitionLabel> = transitions_relabelling
        .into_iter()
        .filter_map(|(_, x)| x)
        .map(|rc| (*rc).clone())
        .collect();
    // ***
    let kripke = petri_to_kripke(
        &petri_retval.petri_net,
        petri_retval.initial_marking,
        &PetriKripkeStateProducer::new(tagged_transition_labels),
        &PetriKripkeGenerationSafenessRequirement::KSafeness(1)
    ).unwrap();

    let semantic_formula = traces_to_ctl(traces, alphabet);
    assert!(
        is_ctl_formula_sat(&kripke, &hash_set!{0}, &semantic_formula)
    );
    
}




#[test]
fn test_semantics_traces_trie_data() {
    tool_test_bpmn_semantic_from_traces(
        "tests/files/data.bpmn",
        hash_set! {vec!["mytask".to_string()]},
        btree_set! {"mytask".to_string()}
    );
}




#[test]
fn test_semantics_traces_trie_gate() {
    tool_test_bpmn_semantic_from_traces(
        "tests/files/gate.bpmn",
        hash_set! {vec!["choiceA".to_string()],vec!["choiceB".to_string()]},
        btree_set! {"choiceA".to_string(),"choiceB".to_string()}
    );
}



#[test]
fn test_semantics_traces_trie_participants() {
    tool_test_bpmn_semantic_from_traces(
        "tests/files/participants.bpmn",
        hash_set! {vec!["actA".to_string(),"actB".to_string()]},
        btree_set! {"actA".to_string(),"actB".to_string()}
    );
}

#[test]
fn test_semantics_traces_trie_participants_and_gates() {
    tool_test_bpmn_semantic_from_traces(
        "tests/files/participants_and_gates.bpmn",
        hash_set! {
            vec!["actA".to_string(),"actB".to_string()],
            vec!["actA".to_string(),"actC".to_string()],
        },
        btree_set! {"actA".to_string(),"actB".to_string(),"actC".to_string()}
    );
}



#[test]
fn test_semantics_traces_trie_gate_to_gate() {
    tool_test_bpmn_semantic_from_traces(
        "tests/files/gate_to_gate_edge.bpmn",
        hash_set! {
            vec!["task".to_string()],
            vec![],
        },
        btree_set! {"task".to_string()}
    );
}


#[test]
fn test_semantics_traces_trie_exception_simple() {
    tool_test_bpmn_semantic_from_traces(
        "tests/files/exception_simple.bpmn",
        hash_set! {
            vec!["k1".to_string()],
            vec!["k1".to_string(),"k3".to_string()],
            vec!["k3".to_string()],
        },
        btree_set! {"k1".to_string(),"k3".to_string()}
    );
}




