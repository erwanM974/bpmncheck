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


use bpmncheck::{parser::bpmn::read_bpmn_diagram_from_file_path, petri::{bpmn_to_petri::bpmn_to_petri, initial_marking::get_initial_marking_from_initial_places}, viz::bpmn_viz::bpmn_repr};
use graphviz_dot_builder::traits::{DotPrintable, GraphVizOutputFormat};
use petricheck::reduction::reduce::reduce_petri_net;

use petricheck::util::vizualisation::petri_viz::petri_repr;




fn tool_test_to_petri(bpmn_file_path : &str, name : &str) {
    let folder_name = "tests_outputs/test_to_petri";
    let _ = std::fs::create_dir("tests_outputs");
    let _ = std::fs::create_dir(folder_name);
    let bpmn = read_bpmn_diagram_from_file_path(bpmn_file_path).unwrap();
    {
        let gv = bpmn_repr(&bpmn);
        let _ = gv.print_dot(
            &[".".to_string()], 
            &format!("{}/{}_bpmn",folder_name, name), 
            &GraphVizOutputFormat::png
        );
    }
    let petri_retval = bpmn_to_petri(&bpmn).unwrap();
    let mut initial_marking = Some(get_initial_marking_from_initial_places(&petri_retval.initial_places));
    let mut petri_net = petri_retval.petri_net.clone();
    {
        let gv = petri_repr(&petri_net,&initial_marking);
        let _ = gv.print_dot(
            &[".".to_string()], 
            &format!("{}/{}_petri",folder_name, name), 
            &GraphVizOutputFormat::png
        );
    }
    reduce_petri_net(&mut petri_net, &mut initial_marking);
    {
        let gv = petri_repr(&petri_net,&initial_marking);
        let _ = gv.print_dot(
            &[".".to_string()], 
            &format!("{}/{}_petri_reduced",folder_name, name), 
            &GraphVizOutputFormat::png
        );
    }
}



#[test]
fn test_bpmn2petri_data() {
    tool_test_to_petri("tests/files/data.bpmn","data");
}

#[test]
fn test_bpmn2petri_gate() {
    tool_test_to_petri("tests/files/gate.bpmn","gate");
}




#[test]
fn test_bpmn2petri_participants() {
    tool_test_to_petri("tests/files/participants.bpmn","participants");
}





#[test]
fn test_bpmn2petri_participants_and_gates() {
    tool_test_to_petri("tests/files/participants_and_gates.bpmn","participants_and_gates");
}




#[test]
fn test_bpmn2petri_exception() {
    tool_test_to_petri("tests/files/exception_simple.bpmn","exception_simple");
}



#[test]
fn test_bpmn2petri_gate_to_gate() {
    tool_test_to_petri("tests/files/gate_to_gate_edge.bpmn","gate_to_gate_edge");
}


/*
#[test]
fn test_bpmn2petri_participants_and_data() {
    tool_test_to_petri("tests/files/basic/participants_and_data.bpmn","participants_and_data");
}
*/
