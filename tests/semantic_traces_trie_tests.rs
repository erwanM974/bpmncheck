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

use bpmncheck::{parser::bpmn::read_bpmn_diagram_from_file_path, petri::{bpmn_to_petri::bpmn_to_petri, initial_marking::get_initial_marking_from_initial_places}};
use citreelo::solve::is_ctl_formula_sat;
use map_macro::{btree_set, hash_set};
use petricheck::{model::label::PetriTransitionLabel, model_checking::to_kripke::{PetriKripkeGenerationSafenessRequirement, PetriKripkeStateProducer, petri_to_kripke}, reduction::reduce::reduce_petri_net};


mod common;
use common::ctl_from_traces_existential::traces_to_ctl;


fn tool_test_bpmn_semantic_from_traces(
    bpmn_file_path : &str,
    traces: HashSet<Vec<String>>,
    alphabet: BTreeSet<String>,
) {
    let bpmn = read_bpmn_diagram_from_file_path(bpmn_file_path).unwrap();
    // ***
    let petri_retval = bpmn_to_petri(&bpmn).unwrap();
    let mut initial_marking = Some(get_initial_marking_from_initial_places(&petri_retval.initial_places));
    let mut petri_net = petri_retval.petri_net.clone();
    // we will relabel the petri net
    // replacing all labels by the empty label
    // except for tasks, which will be labelled by their names (we do suppose all of them do have names)
    let mut relabelling : HashMap<PetriTransitionLabel, Option<Rc<PetriTransitionLabel>>> = HashMap::new();
    let mut tagged_transition_labels = HashSet::new();
    for (bpmnid,tr_lab) in petri_retval.bpmn_id_to_transitions_labels {
        let new_label = if let Some(act) = bpmn.activities.get(&bpmnid) && act.activity_type.is_task() {
            match &act.name {
                Some(x) => {
                    let new_tr_lab = PetriTransitionLabel::new(x.to_string());
                    tagged_transition_labels.insert(new_tr_lab.clone());
                    Some(Rc::new(new_tr_lab))
                }
                None => None,
            }
        } else {
            None
        };
        relabelling.insert((*tr_lab).clone(), new_label);
    }
    petri_net.relabel_transitions(&relabelling);
    // ***
    reduce_petri_net(&mut petri_net, &mut initial_marking);
    // *** 
    // we then generate a kripke structure with all transition labelled as "previous"
    let im = initial_marking.unwrap();
    let kripke = petri_to_kripke(
        &petri_net, 
        im, 
        &PetriKripkeStateProducer::new(tagged_transition_labels), 
        &PetriKripkeGenerationSafenessRequirement::KSafeness(1)
    ).unwrap();

    /*{
        let gv = PetriKripkeVisualizer::new(&petri_net).get_kripke_repr(&kripke);
        gv.print_dot(
            &[".".to_string()], 
            &format!("kripke"), 
            &GraphVizOutputFormat::png
        ).unwrap();
    }*/

    let semantic_formula = traces_to_ctl(traces,alphabet);
    assert!(
        is_ctl_formula_sat(&kripke, &hash_set!{0}, &semantic_formula),
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




