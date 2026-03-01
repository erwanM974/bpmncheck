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

use bpmncheck::{parser::bpmn::read_bpmn_diagram_from_file_path, petri::{bpmn_to_petri::bpmn_to_petri, initial_marking::get_initial_marking_from_initial_places}};
use citreelo::{parser::CtlFormulaParser, solve::is_ctl_formula_sat, util::viz_kripke::KripkeStructureGraphvizDrawer};
use graphviz_dot_builder::traits::{DotPrintable, GraphVizOutputFormat};
use map_macro::hash_set;
use petricheck::{model::label::PetriTransitionLabel, model_checking::to_kripke::{PetriKripkeGenerationSafenessRequirement, PetriKripkeStateProducer, petri_to_kripke}, reduction::reduce::reduce_petri_net, util::{parse_ctl::parser::BuiltinPetriCtlParser, vizualisation::{kripke_viz::PetriKripkeVisualizer, petri_viz::petri_repr}}};




fn tool_test_bpmn_semantic_manual(
    bpmn_name : &str,
    bpmn_file_path : &str,
    semantic_formula_str : &str
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
    {
        let _ = std::fs::create_dir("tests_outputs");
        let _ = std::fs::create_dir("tests_outputs/semantic_manual");
        let gv = petri_repr(&petri_net,&initial_marking);
        let _ = gv.print_dot(
            &["tests_outputs".to_string(),"semantic_manual".to_string()], 
            &format!("{}_petri_reduced", bpmn_name), 
            &GraphVizOutputFormat::png
        );
    }
    // *** 
    // we then generate a kripke structure with all transition labelled as "previous"
    let im = initial_marking.unwrap();
    let kripke = petri_to_kripke(
        &petri_net, 
        im, 
        &PetriKripkeStateProducer::new(tagged_transition_labels), 
        &PetriKripkeGenerationSafenessRequirement::KSafeness(1)
    ).unwrap();

    {
        let gv = PetriKripkeVisualizer::new(&petri_net).get_kripke_repr(&kripke);
        gv.print_dot(
            &["tests_outputs".to_string(),"semantic_manual".to_string()], 
            &format!("{}_kripke", bpmn_name),
            &GraphVizOutputFormat::png
        ).unwrap();
    }

    let ctl_parser = BuiltinPetriCtlParser::from_net(&petri_net).unwrap();
    let (_,semantic_formula) = ctl_parser.parse_ctl_formula::<nom::error::Error<&str>>(semantic_formula_str).unwrap();
    assert!(
        is_ctl_formula_sat(&kripke, &hash_set!{0}, &semantic_formula),
        "{}", semantic_formula_str
    );
    
}




#[test]
fn test_semantics_manual_data() {
    tool_test_bpmn_semantic_manual(
        "data",
        "tests/files/data.bpmn",
        r#"A(X(is-previous("mytask")))"#
    );
}




#[test]
fn test_semantics_manual_gate() {
    tool_test_bpmn_semantic_manual(
        "gate",
        "tests/files/gate.bpmn",
        r#"A(X( (is-previous("choiceA")) | (is-previous("choiceB")) ))"#
    );
    tool_test_bpmn_semantic_manual(
        "gate",
        "tests/files/gate.bpmn",
        r#"!(A(X( is-previous("choiceA") )))"#
    );
    tool_test_bpmn_semantic_manual(
        "gate",
        "tests/files/gate.bpmn",
        r#"!(A(X( is-previous("choiceB") )))"#
    );
    tool_test_bpmn_semantic_manual(
        "gate",
        "tests/files/gate.bpmn",
        r#"
    
        (E(
            (!(is-previous("choiceB")))
            U 
            (is-previous("choiceA"))
))
        | 
        (E(
            (!(is-previous("choiceA")))
            U 
            (is-previous("choiceB"))
))
    
    "#
    );
}



#[test]
fn test_semantics_manual_participants() {
    tool_test_bpmn_semantic_manual(
        "participants",
        "tests/files/participants.bpmn",
        r#"A(X( (is-previous("actA")) & ( A(X( is-previous("actB") ))  ) ))"#
    );
}

#[test]
fn test_semantics_manual_participants_and_gates() {
    tool_test_bpmn_semantic_manual(
        "participants_and_gates",
        "tests/files/participants_and_gates.bpmn",
        r#"
        A(X( 
            (is-previous("actA")) 
            & 
            ( 
              A(F(   
                  (is-previous("actB")) 
                  | 
                  (is-previous("actC"))   
              ))  
            ) 
        ))
        "#
    );
}


#[test]
fn test_semantics_manual_exception_simple() {
    tool_test_bpmn_semantic_manual(
        "exception_simple",
        "tests/files/exception_simple.bpmn",
        r#"
        E(F( 
            (is-previous("k1")) 
            & 
            (
                E(F(   
                    is-previous("k3")
                ))  
            )
        ))
        "#
    );
}



