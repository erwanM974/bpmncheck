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



use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use bpmncheck::{model::{activity::{Activity, ActivityType, TaskKind}, diagram::{Diagram, Process, ProcessContentRef}, event::{Event, EventType}, flow::{Flow, FlowKind}, gateway::{Gateway, GatewayType}, id::BpmnId}, parser::bpmn::read_bpmn_diagram_from_file_path};
use map_macro::{btree_map, btree_set, hash_map, hash_set};






fn tool_test_parse_bpmn(
    bpmn_file_path : &str,
    expected_bpmn : Diagram
) {
    let got_bpmn = read_bpmn_diagram_from_file_path(bpmn_file_path).unwrap();
    {
        let expected_data_bpmnid_labels : BTreeSet<String> = expected_bpmn.data.keys().map(|x| x.id.to_owned()).collect();
        let got_data_bpmnid_labels : BTreeSet<String> = got_bpmn.data.keys().map(|x| x.id.to_owned()).collect();
        assert_eq!(got_data_bpmnid_labels,expected_data_bpmnid_labels);
        for data_bpmnid_label in expected_data_bpmnid_labels {
            let data_bpmn_id = BpmnId::new(data_bpmnid_label);
            let expected_data = expected_bpmn.data.get(&data_bpmn_id).unwrap();
            let got_data = got_bpmn.data.get(&data_bpmn_id).unwrap();
            assert_eq!(got_data,expected_data);
        }
    }
    {
        let expected_events_bpmnid_labels : BTreeSet<String> = expected_bpmn.events.keys().map(|x| x.id.to_owned()).collect();
        let got_events_bpmnid_labels : BTreeSet<String> = got_bpmn.events.keys().map(|x| x.id.to_owned()).collect();
        assert_eq!(got_events_bpmnid_labels,expected_events_bpmnid_labels);
        for event_bpmnid_label in expected_events_bpmnid_labels {
            let event_bpmn_id = BpmnId::new(event_bpmnid_label);
            let expected_event = expected_bpmn.events.get(&event_bpmn_id).unwrap();
            let got_event = got_bpmn.events.get(&event_bpmn_id).unwrap();
            assert_eq!(got_event,expected_event);
        }
    }
    {
        let expected_activities_bpmnid_labels : BTreeSet<String> = expected_bpmn.activities.keys().map(|x| x.id.to_owned()).collect();
        let got_activities_bpmnid_labels : BTreeSet<String> = got_bpmn.activities.keys().map(|x| x.id.to_owned()).collect();
        assert_eq!(got_activities_bpmnid_labels,expected_activities_bpmnid_labels);
        for activity_bpmnid_label in expected_activities_bpmnid_labels {
            let activity_bpmn_id = BpmnId::new(activity_bpmnid_label);
            let expected_act = expected_bpmn.activities.get(&activity_bpmn_id).unwrap();
            let got_act = got_bpmn.activities.get(&activity_bpmn_id).unwrap();
            assert_eq!(got_act,expected_act);
        }
    }
    {
        let expected_gates_bpmnid_labels : BTreeSet<String> = expected_bpmn.gateways.keys().map(|x| x.id.to_owned()).collect();
        let got_gates_bpmnid_labels : BTreeSet<String> = got_bpmn.gateways.keys().map(|x| x.id.to_owned()).collect();
        assert_eq!(got_gates_bpmnid_labels,expected_gates_bpmnid_labels);
        for gate_bpmnid_label in expected_gates_bpmnid_labels {
            let gate_bpmn_id = BpmnId::new(gate_bpmnid_label);
            let expected_gate = expected_bpmn.gateways.get(&gate_bpmn_id).unwrap();
            let got_gate = got_bpmn.gateways.get(&gate_bpmn_id).unwrap();
            assert_eq!(got_gate,expected_gate);
        }
    }
    {
        let expected_flows_bpmnid_labels : BTreeSet<String> = expected_bpmn.sequence_flows.keys().map(|x| x.id.to_owned()).collect();
        let got_flows_bpmnid_labels : BTreeSet<String> = got_bpmn.sequence_flows.keys().map(|x| x.id.to_owned()).collect();
        assert_eq!(got_flows_bpmnid_labels,expected_flows_bpmnid_labels);
        for flow_bpmnid_label in expected_flows_bpmnid_labels {
            let flow_bpmn_id = BpmnId::new(flow_bpmnid_label);
            let expected_flow = expected_bpmn.sequence_flows.get(&flow_bpmn_id).unwrap();
            let got_flow = got_bpmn.sequence_flows.get(&flow_bpmn_id).unwrap();
            assert_eq!(got_flow,expected_flow);
        }
    }
    {
        let expected_flows_bpmnid_labels : BTreeSet<String> = expected_bpmn.message_flows.keys().map(|x| x.id.to_owned()).collect();
        let got_flows_bpmnid_labels : BTreeSet<String> = got_bpmn.message_flows.keys().map(|x| x.id.to_owned()).collect();
        assert_eq!(got_flows_bpmnid_labels,expected_flows_bpmnid_labels);
        for flow_bpmnid_label in expected_flows_bpmnid_labels {
            let flow_bpmn_id = BpmnId::new(flow_bpmnid_label);
            let expected_flow = expected_bpmn.message_flows.get(&flow_bpmn_id).unwrap();
            let got_flow = got_bpmn.message_flows.get(&flow_bpmn_id).unwrap();
            assert_eq!(got_flow,expected_flow);
        }
    }
    {
        let expected_top_level_procs_bpmnid_labels : BTreeSet<String> = expected_bpmn.top_level_processes.keys().map(|x| x.id.to_owned()).collect();
        let got_top_level_procs_bpmnid_labels : BTreeSet<String> = got_bpmn.top_level_processes.keys().map(|x| x.id.to_owned()).collect();
        assert_eq!(got_top_level_procs_bpmnid_labels,expected_top_level_procs_bpmnid_labels);
        for top_level_proc_bpmnid_label in expected_top_level_procs_bpmnid_labels {
            let top_level_proc_bpmn_id = BpmnId::new(top_level_proc_bpmnid_label);
            let expected_top_level_proc = expected_bpmn.top_level_processes.get(&top_level_proc_bpmn_id).unwrap();
            let got_top_level_proc = got_bpmn.top_level_processes.get(&top_level_proc_bpmn_id).unwrap();
            assert_eq!(got_top_level_proc,expected_top_level_proc);
        }
    }
}




#[test]
fn test_parser_basic_bpmn_with_data() {
    // ***
    let data = hash_map! {
        BpmnId::new("DataObjectReference_1rmzqxh".to_string()) => "myoutput".to_string(),
        BpmnId::new("DataObjectReference_0o6dovj".to_string()) => "myinput".to_string()
    };
    // ***
    let events = hash_map! {
        BpmnId::new("StartEvent_12jqtlo".to_string()) => 
            Event::new(EventType::Start, None, BpmnId::new("StartEvent_12jqtlo".to_string()), None),
        BpmnId::new("Event_1eivmpm".to_string()) =>
            Event::new(EventType::End, None, BpmnId::new("Event_1eivmpm".to_string()), None)
    };
    // ***
    let activities = {
        let input = hash_set!{
            BpmnId::new("DataObjectReference_0o6dovj".to_string())
        };
        let output = hash_set!{
            BpmnId::new("DataObjectReference_1rmzqxh".to_string())
        };
        hash_map! {
            BpmnId::new("Activity_1glraf1".to_string()) => Activity::new(
                ActivityType::Task(TaskKind::DefaultTask), 
                BpmnId::new("Activity_1glraf1".to_string()), 
                Some("mytask".to_string()), 
                input, 
                output
            )
        }
    };
    // ***
    let sequence_flows = hash_map! {
        BpmnId::new("Flow_1pgpjfa".to_string()) => Flow::new(
            BpmnId::new("Flow_1pgpjfa".to_string()), 
            FlowKind::Sequence, 
            None, 
            BpmnId::new("StartEvent_12jqtlo".to_string()),
            BpmnId::new("Activity_1glraf1".to_string())
        ),
        BpmnId::new("Flow_1ipixfc".to_string()) => Flow::new(
            BpmnId::new("Flow_1ipixfc".to_string()), 
            FlowKind::Sequence, 
            None, 
            BpmnId::new("Activity_1glraf1".to_string()),
            BpmnId::new("Event_1eivmpm".to_string())
        )
    };
    // ***
    let top_level_processes = {
        let content_ref = ProcessContentRef::new(
            btree_set! {
                BpmnId::new("StartEvent_12jqtlo".to_string()),
                BpmnId::new("Event_1eivmpm".to_string())
            }, btree_set! {
                BpmnId::new("Activity_1glraf1".to_string())
            }, btree_set! {}, 
            btree_set! {
                BpmnId::new("Flow_1pgpjfa".to_string()),
                BpmnId::new("Flow_1ipixfc".to_string()),
            },
        );
        let proc = Process::new(
            BpmnId::new("Process_19fxk2u".to_string()), 
            Some("myparticipant".to_string()),
             content_ref
        );
        btree_map! {
            BpmnId::new("Process_19fxk2u".to_string()) => proc
        }
    };
    let expected_bpmn = Diagram::new(
        top_level_processes, 
        events, 
        activities, 
        HashMap::new(), 
        sequence_flows, 
        HashMap::new(), 
        data,
    );
    tool_test_parse_bpmn("tests/files/data.bpmn", expected_bpmn);
}





#[test]
fn test_parser_basic_bpmn_with_gateway() {
    // ***
    let events = hash_map! {
        BpmnId::new("StartEvent_1yr15k3".to_string()) => 
            Event::new(EventType::Start, None, BpmnId::new("StartEvent_1yr15k3".to_string()), None),
        BpmnId::new("Event_1ak0z14".to_string()) => 
            Event::new(EventType::End, None, BpmnId::new("Event_1ak0z14".to_string()), None),
    };
    // ***
    let mut activities = HashMap::new();
    {
        activities.insert(
            BpmnId::new("Activity_0z8knjr".to_string()),  
            Activity::new(
                ActivityType::Task(TaskKind::DefaultTask), 
                BpmnId::new("Activity_0z8knjr".to_string()), 
                Some("choiceA".to_string()), 
                HashSet::new(), 
                HashSet::new()
            )
        );
        activities.insert(
            BpmnId::new("Activity_0bia3e9".to_string()),  
            Activity::new(
                ActivityType::Task(TaskKind::DefaultTask), 
                BpmnId::new("Activity_0bia3e9".to_string()), 
                Some("choiceB".to_string()), 
                HashSet::new(), 
                HashSet::new()
            )
        );
    }
    // ***
    let gateways = hash_map! {
        BpmnId::new("Gateway_19h1ahe".to_string()) =>
            Gateway::new(GatewayType::Exclusive, BpmnId::new("Gateway_19h1ahe".to_string()), None),
        BpmnId::new("Gateway_1ulbs6o".to_string()) =>
            Gateway::new(GatewayType::Exclusive, BpmnId::new("Gateway_1ulbs6o".to_string()), None),
    };
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
    flows.insert(
        BpmnId::new("Flow_0lh30m8".to_string()), 
        Flow::new(
            BpmnId::new("Flow_0lh30m8".to_string()), 
            FlowKind::Sequence, 
            None, 
            BpmnId::new("Activity_0z8knjr".to_string()),
            BpmnId::new("Gateway_1ulbs6o".to_string())
        )
    );
    flows.insert(
        BpmnId::new("Flow_0fzpuql".to_string()), 
        Flow::new(
            BpmnId::new("Flow_0fzpuql".to_string()), 
            FlowKind::Sequence, 
            None, 
            BpmnId::new("Activity_0bia3e9".to_string()),
            BpmnId::new("Gateway_1ulbs6o".to_string())
        )
    );
    flows.insert(
        BpmnId::new("Flow_1jqwhpf".to_string()), 
        Flow::new(
            BpmnId::new("Flow_1jqwhpf".to_string()), 
            FlowKind::Sequence, 
            None, 
            BpmnId::new("Gateway_1ulbs6o".to_string()),
            BpmnId::new("Event_1ak0z14".to_string())
        )
    );
    // ***
    let mut top_level_processes = BTreeMap::new();
    {
        let content_ref = ProcessContentRef::new(
            btree_set! {
                BpmnId::new("StartEvent_1yr15k3".to_string()),
                BpmnId::new("Event_1ak0z14".to_string()),
            }, btree_set! {
                BpmnId::new("Activity_0z8knjr".to_string()),
                BpmnId::new("Activity_0bia3e9".to_string())
            }, btree_set! {
                BpmnId::new("Gateway_19h1ahe".to_string()),
                BpmnId::new("Gateway_1ulbs6o".to_string()),
            },
            btree_set! {
                BpmnId::new("Flow_17iodak".to_string()), 
                BpmnId::new("Flow_0vvuynt".to_string()), 
                BpmnId::new("Flow_0ebe3xq".to_string()), 
                BpmnId::new("Flow_0lh30m8".to_string()), 
                BpmnId::new("Flow_0fzpuql".to_string()), 
                BpmnId::new("Flow_1jqwhpf".to_string()), 
            }
        );
        let proc = Process::new(
            BpmnId::new("Process_1a99iew".to_string()), 
            Some("myparticipant".to_string()),
             content_ref
        );
        top_level_processes.insert(BpmnId::new("Process_1a99iew".to_string()), proc);
    }
    
    let expected_bpmn = Diagram::new(
        top_level_processes, 
        events, 
        activities, 
        gateways, 
        flows, 
        HashMap::new(),
        HashMap::new()
    );
    tool_test_parse_bpmn("tests/files/gate.bpmn", expected_bpmn);
}








