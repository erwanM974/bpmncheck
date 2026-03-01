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


use bpmncheck::{parser::bpmn::read_bpmn_diagram_from_file_path, wellformedness::check::check_model_well_formedness};



fn tool_test_well_formedness(bpmn_file_path : &str) {
    let bpmn = read_bpmn_diagram_from_file_path(bpmn_file_path).unwrap();
    let well_formedness = check_model_well_formedness(&bpmn);
    assert!(well_formedness.is_ok(), "{:?}", well_formedness);
}



#[test]
fn test_well_formedness_data() {
    tool_test_well_formedness("tests/files/data.bpmn");
}

#[test]
fn test_well_formedness_gate() {
    tool_test_well_formedness("tests/files/gate.bpmn");
}




#[test]
fn test_well_formedness_participants() {
    tool_test_well_formedness("tests/files/participants.bpmn");
}





#[test]
fn test_well_formedness_participants_and_gates() {
    tool_test_well_formedness("tests/files/participants_and_gates.bpmn");
}




#[test]
fn test_well_formedness_exception() {
    tool_test_well_formedness("tests/files/exception_simple.bpmn");
}



#[test]
fn test_well_formedness_gate_to_gate() {
    tool_test_well_formedness("tests/files/gate_to_gate_edge.bpmn");
}

