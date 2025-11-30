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



use std::collections::{BTreeMap, HashMap, HashSet};

use bpmncheck::{model::{activity::{Activity, ActivityType}, diagram::{Process, ProcessContentRef}, event::{Event, EventType}, flow::{Flow, FlowKind}, gateway::{Gateway, GatewayType}, id::BpmnId}, parser::bpmn::read_bpmn_diagram_from_file_path};
use map_macro::{hash_map, hash_set, btree_set};




#[test]
fn test_parser_basic_bpmn_with_data() {
    let bpmn = read_bpmn_diagram_from_file_path("tests/files/basic/data.bpmn").unwrap();
    // ***
    let data = hash_map! {
        BpmnId::new("DataObjectReference_1rmzqxh".to_string()) => "myoutput".to_string(),
        BpmnId::new("DataObjectReference_0o6dovj".to_string()) => "myinput".to_string()
    };
    assert_eq!(data,bpmn.data);
    // ***
    let events = hash_map! {
        BpmnId::new("StartEvent_12jqtlo".to_string()) => 
            Event::new(EventType::Start, None, BpmnId::new("StartEvent_12jqtlo".to_string()), None),
        BpmnId::new("Event_1eivmpm".to_string()) =>
            Event::new(EventType::End, None, BpmnId::new("Event_1eivmpm".to_string()), None)
    };
    assert_eq!(events,bpmn.events);
    // ***
    let mut activities = HashMap::new();
    {
        let input = hash_set!{
            BpmnId::new("DataObjectReference_0o6dovj".to_string())
        };
        let output = hash_set!{
            BpmnId::new("DataObjectReference_1rmzqxh".to_string())
        };
        activities.insert(
            BpmnId::new("Activity_1glraf1".to_string()),  
            Activity::new(
                ActivityType::Task, 
                BpmnId::new("Activity_1glraf1".to_string()), 
                Some("mytask".to_string()), 
                input, 
                output
            )
        );
    }
    assert_eq!(activities,bpmn.activities);
    // ***
    assert_eq!(HashMap::new(),bpmn.gateways);
    // ***
    let mut flows = HashMap::new();
    flows.insert(
        BpmnId::new("Flow_1pgpjfa".to_string()), 
        Flow::new(
            BpmnId::new("Flow_1pgpjfa".to_string()), 
            FlowKind::Sequence, 
            None, 
            BpmnId::new("StartEvent_12jqtlo".to_string()),
            BpmnId::new("Activity_1glraf1".to_string())
        )
    );
    flows.insert(
        BpmnId::new("Flow_1ipixfc".to_string()), 
        Flow::new(
            BpmnId::new("Flow_1ipixfc".to_string()), 
            FlowKind::Sequence, 
            None, 
            BpmnId::new("Activity_1glraf1".to_string()),
            BpmnId::new("Event_1eivmpm".to_string())
        )
    );
    assert_eq!(flows,bpmn.flows);
    // ***
    let mut top_level_processes = BTreeMap::new();
    {
        let content_ref = ProcessContentRef::new(
            btree_set! {
                BpmnId::new("StartEvent_12jqtlo".to_string()),
                BpmnId::new("Event_1eivmpm".to_string())
            }, btree_set! {
                BpmnId::new("Activity_1glraf1".to_string())
            }, btree_set! {}
        );
        let proc = Process::new(
            BpmnId::new("Process_19fxk2u".to_string()), 
            Some("myparticipant".to_string()),
             content_ref
        );
        top_level_processes.insert(BpmnId::new("Process_19fxk2u".to_string()), proc);
    }
    assert_eq!(top_level_processes,bpmn.top_level_processes);
}





#[test]
fn test_parser_basic_bpmn_with_gateway() {
    let bpmn = read_bpmn_diagram_from_file_path("tests/files/basic/gate.bpmn").unwrap();
    // ***
    assert_eq!(HashMap::new(),bpmn.data);
    // ***
    let events = hash_map! {
        BpmnId::new("StartEvent_1yr15k3".to_string()) => 
            Event::new(EventType::Start, None, BpmnId::new("StartEvent_1yr15k3".to_string()), None)
    };
    assert_eq!(events,bpmn.events);
    // ***
    let mut activities = HashMap::new();
    {
        activities.insert(
            BpmnId::new("Activity_0z8knjr".to_string()),  
            Activity::new(
                ActivityType::Task, 
                BpmnId::new("Activity_0z8knjr".to_string()), 
                Some("choiceA".to_string()), 
                HashSet::new(), 
                HashSet::new()
            )
        );
        activities.insert(
            BpmnId::new("Activity_0bia3e9".to_string()),  
            Activity::new(
                ActivityType::Task, 
                BpmnId::new("Activity_0bia3e9".to_string()), 
                Some("choiceB".to_string()), 
                HashSet::new(), 
                HashSet::new()
            )
        );
    }
    assert_eq!(activities,bpmn.activities);
    // ***
    let gateways = hash_map! {
        BpmnId::new("Gateway_19h1ahe".to_string()) =>
            Gateway::new(GatewayType::Exclusive, BpmnId::new("Gateway_19h1ahe".to_string()), None)
    };
    assert_eq!(gateways,bpmn.gateways);
    // ***
    let mut flows = HashMap::new();
    flows.insert(
        BpmnId::new("Flow_17iodak".to_string()), 
        Flow::new(
            BpmnId::new("Flow_17iodak".to_string()), 
            FlowKind::Sequence, 
            None, 
            BpmnId::new("StartEvent_1yr15k3".to_string()),
            BpmnId::new("Gateway_19h1ahe".to_string())
        )
    );
    flows.insert(
        BpmnId::new("Flow_0vvuynt".to_string()), 
        Flow::new(
            BpmnId::new("Flow_0vvuynt".to_string()), 
            FlowKind::Sequence, 
            None, 
            BpmnId::new("Gateway_19h1ahe".to_string()),
            BpmnId::new("Activity_0z8knjr".to_string())
        )
    );
    flows.insert(
        BpmnId::new("Flow_0ebe3xq".to_string()), 
        Flow::new(
            BpmnId::new("Flow_0ebe3xq".to_string()), 
            FlowKind::Sequence, 
            None, 
            BpmnId::new("Gateway_19h1ahe".to_string()),
            BpmnId::new("Activity_0bia3e9".to_string())
        )
    );
    assert_eq!(flows,bpmn.flows);
    // ***
    let mut top_level_processes = BTreeMap::new();
    {
        let content_ref = ProcessContentRef::new(
            btree_set! {
                BpmnId::new("StartEvent_1yr15k3".to_string())
            }, btree_set! {
                BpmnId::new("Activity_0z8knjr".to_string()),
                BpmnId::new("Activity_0bia3e9".to_string())
            }, btree_set! {
                BpmnId::new("Gateway_19h1ahe".to_string())
            }
        );
        let proc = Process::new(
            BpmnId::new("Process_1a99iew".to_string()), 
            Some("myparticipant".to_string()),
             content_ref
        );
        top_level_processes.insert(BpmnId::new("Process_1a99iew".to_string()), proc);
    }
    assert_eq!(top_level_processes,bpmn.top_level_processes);
}