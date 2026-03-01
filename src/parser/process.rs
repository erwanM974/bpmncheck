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


use std::collections::{BTreeSet, HashMap, HashSet};
use std::io::BufRead;


use xml::reader::XmlEvent;
use xml::EventReader;

use crate::model::activity::{Activity, ActivityType, TaskKind};
use crate::model::diagram::ProcessContentRef;
use crate::model::flow::{Flow, FlowKind};
use crate::model::gateway::Gateway;
use crate::model::id::BpmnId;
use crate::parser::elements::{read_data_association, read_data_object_reference, read_event, read_flow, read_gate};
use crate::parser::grammar::*;
use crate::parser::util::collect_attributes;
use crate::{parser::error::BpmnParsingError};

use crate::model::event::{Event};






pub struct NestedElements {
    pub events         : HashMap<BpmnId,Event>,
    pub activities     : HashMap<BpmnId,Activity>,
    pub gateways       : HashMap<BpmnId,Gateway>,
    pub sequence_flows : HashMap<BpmnId,Flow>,
    pub data           : HashMap<BpmnId,String>
}

impl NestedElements {
    pub fn new(events: HashMap<BpmnId,Event>, activities: HashMap<BpmnId,Activity>, gateways : HashMap<BpmnId,Gateway>, sequence_flows: HashMap<BpmnId,Flow>, data: HashMap<BpmnId,String>) -> Self {
        Self { events, activities, gateways, sequence_flows, data }
    }
    
    pub fn new_empty() -> Self {
        Self::new(
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new()
        )
    }
    pub fn updated_with(&mut self, other : NestedElements) {
        self.events.extend(other.events);
        self.activities.extend(other.activities);
        self.gateways.extend(other.gateways);
        self.sequence_flows.extend(other.sequence_flows);
        self.data.extend(other.data);
    }
}

pub struct ActivityParsingReturnValue {
    pub nested_elements         : NestedElements,
    pub direct_child_events     : BTreeSet<BpmnId>,
    pub direct_child_activities : BTreeSet<BpmnId>,
    pub direct_child_gateways   : BTreeSet<BpmnId>,
    pub direct_child_flows      : BTreeSet<BpmnId>,
    pub input_data              : HashSet<BpmnId>, 
    pub output_data             : HashSet<BpmnId>,
}

impl ActivityParsingReturnValue {
    pub fn new(
        nested_elements         : NestedElements, 
        direct_child_events     : BTreeSet<BpmnId>, 
        direct_child_activities : BTreeSet<BpmnId>, 
        direct_child_gateways   : BTreeSet<BpmnId>, 
        direct_child_flows      : BTreeSet<BpmnId>,
        input_data              : HashSet<BpmnId>, 
        output_data             : HashSet<BpmnId>) -> Self {
        Self { nested_elements, direct_child_events, direct_child_activities, direct_child_gateways, direct_child_flows, input_data, output_data }
    }
}



/* 
BPMN subProcesses are both Processes and Activities (tasks).
Thus we do not have two distinct functions (one to read processes and another for tasks).
*/
pub fn read_activity<R: BufRead>(
    reader: &mut EventReader<R>,
    expected_end_tag : &str
) -> Result<ActivityParsingReturnValue, BpmnParsingError> {
    // if it concerns a process or a subprocess activity, we may have some nested elements that need to be propagated up
    let mut elements = NestedElements::new_empty();
    // if the activity is a task, it may have some input or output data
    let mut input_data = HashSet::new();
    let mut output_data = HashSet::new();
    // if the activity is a subprocess, we keep track of the events/activities it directly owns
    let mut direct_child_events = BTreeSet::new();
    let mut direct_child_activities = BTreeSet::new();
    let mut direct_child_gateways = BTreeSet::new();
    let mut direct_child_flows = BTreeSet::new();
    // ***
    loop {
        match reader.next() {
            Ok(XmlEvent::StartElement{name,attributes,..}) => {
                match name.local_name.as_str() {
                    BPMN_SEQUENCE_FLOW => {
                        let flow = read_flow(FlowKind::Sequence, collect_attributes(attributes))?;
                        direct_child_flows.insert(flow.id.clone());
                        elements.sequence_flows.insert(flow.id.clone(), flow);
                    },
                    BPMN_DATA_OBJECT_REFERENCE => {
                        let (data_id,data_name) = read_data_object_reference(collect_attributes(attributes))?;
                        elements.data.insert(data_id, data_name);
                    }
                    BPMN_DATA_INPUT_ASSOCIATION => {
                        let input_data_obj_ref_id = read_data_association(reader,true)?;
                        input_data.insert(input_data_obj_ref_id);
                    },
                    BPMN_DATA_OUTPUT_ASSOCIATION => {
                        let output_data_obj_ref_id = read_data_association(reader,false)?;
                        output_data.insert(output_data_obj_ref_id);
                    },
                    gateway_type @ (
                        BPMN_EXCLUSIVE_GATEWAY 
                        | BPMN_PARALLEL_GATEWAY 
                        | BPMN_INCLUSIVE_GATEWAY
                    ) => {
                        let gate = read_gate(reader,gateway_type,collect_attributes(attributes))?;
                        direct_child_gateways.insert(gate.id.clone());
                        elements.gateways.insert(gate.id.clone(),gate);
                    }
                    event_type @ (
                        BPMN_START_EVENT
                        |BPMN_END_EVENT
                        |BPMN_INTERMEDIATE_CATCH_EVENT
                        |BPMN_INTERMEDIATE_THROW_EVENT
                        |BPMN_BOUNDARY_EVENT
                    ) => {
                        let evt = read_event(reader,event_type,collect_attributes(attributes))?;
                        direct_child_events.insert(evt.id.clone());
                        elements.events.insert(evt.id.clone(),evt);
                    },
                    // ***
                    act_type @ (
                        BPMN_TASK
                        |BPMN_SERVICE_TASK
                        |BPMN_USER_TASK
                        |BPMN_SCRIPT_TASK
                        |BPMN_RECEIVE_TASK
                        |BPMN_SEND_TASK
                        |BPMN_MANUAL_TASK
                        |BPMN_BUSINESS_RULE_TASK
                        |BPMN_CALL_ACTIVITY
                        |BPMN_SUB_PROCESS
                    ) => {
                        let sub_act = read_activity(reader, act_type)?;
                        let (static_tag,activity_type) = read_activity_type(
                            act_type,
                            ProcessContentRef::new(
                                sub_act.direct_child_events, 
                                sub_act.direct_child_activities, 
                                sub_act.direct_child_gateways, 
                                sub_act.direct_child_flows)
                        )?;
                        let (task_id,task_name) = read_task_attributes(static_tag,collect_attributes(attributes))?;
                        let act = Activity::new(
                            activity_type,
                            task_id,
                            task_name,
                            sub_act.input_data,
                            sub_act.output_data
                        );
                        direct_child_activities.insert(act.id.clone());
                        elements.activities.insert(act.id.clone(),act);
                        elements.updated_with(sub_act.nested_elements);
                    },
                    _ => {}
                }
            },
            Ok(XmlEvent::EndElement{name}) => {
                if name.local_name == expected_end_tag {
                    break;
                }
            }
            _ => {}
        }
    }
    let ret_val = ActivityParsingReturnValue::new(
        elements,
        direct_child_events,
        direct_child_activities,
        direct_child_gateways,
        direct_child_flows,
        input_data,
        output_data
    );
    Ok(ret_val)
}



pub fn read_task_attributes(
    parent_tag : &'static str,
    mut attrs : HashMap<String,String>
) -> Result<(BpmnId,Option<String>),BpmnParsingError> {
    let id: String = attrs.remove(BPMN_ID).ok_or(BpmnParsingError::MissingAttribute{att:BPMN_ID,parent:parent_tag})?;
    let name = attrs.remove(BPMN_NAME);
    Ok((BpmnId::new(id), name))
}

pub fn read_activity_type(
    tag : &str,
    content_ref : ProcessContentRef
) -> Result<(&'static str,ActivityType),BpmnParsingError> {
    match tag {
        BPMN_TASK               => Ok((BPMN_TASK,ActivityType::Task(TaskKind::DefaultTask))),
        BPMN_SERVICE_TASK       => Ok((BPMN_SERVICE_TASK,ActivityType::Task(TaskKind::ServiceTask))),
        BPMN_USER_TASK          => Ok((BPMN_USER_TASK,ActivityType::Task(TaskKind::UserTask))),
        BPMN_SCRIPT_TASK        => Ok((BPMN_SCRIPT_TASK,ActivityType::Task(TaskKind::ScriptTask))),
        BPMN_RECEIVE_TASK       => Ok((BPMN_RECEIVE_TASK,ActivityType::Task(TaskKind::ReceiveTask))),
        BPMN_SEND_TASK          => Ok((BPMN_SEND_TASK,ActivityType::Task(TaskKind::SendTask))),
        BPMN_MANUAL_TASK        => Ok((BPMN_MANUAL_TASK,ActivityType::Task(TaskKind::ManualTask))),
        BPMN_BUSINESS_RULE_TASK => Ok((BPMN_BUSINESS_RULE_TASK,ActivityType::Task(TaskKind::BusinessRuleTask))),
        //BPMN_CALL_ACTIVITY      => Ok((BPMN_CALL_ACTIVITY,ActivityType::CallActivity)),
        BPMN_SUB_PROCESS        => Ok((BPMN_SUB_PROCESS,ActivityType::SubProcess(content_ref))),
        //BPMN_TRANSACTION => Ok(ActivityType::Transaction),
        _ => Err(BpmnParsingError::UnknownActivityType)
    }
}