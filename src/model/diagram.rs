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



use crate::model::{activity::Activity, event::Event, flow::Flow, gateway::Gateway, id::BpmnId};
use std::collections::{BTreeMap, BTreeSet, HashMap};



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessContentRef {
    pub direct_child_events     : BTreeSet<BpmnId>,
    pub direct_child_activities : BTreeSet<BpmnId>,
    pub direct_child_gateways   : BTreeSet<BpmnId>,
    pub direct_child_flows      : BTreeSet<BpmnId>,
}

impl ProcessContentRef {
    pub fn new(
        direct_child_events: BTreeSet<BpmnId>, 
        direct_child_activities: BTreeSet<BpmnId>, 
        direct_child_gateways : BTreeSet<BpmnId>, 
        direct_child_flows   : BTreeSet<BpmnId>) -> Self {
        Self { direct_child_events, direct_child_activities, direct_child_gateways, direct_child_flows }
    }
}



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Process {
    // id of the process or the sub-process
    pub id : BpmnId,
    // name of the participant (if a pool process) or name of the activity (if a sub-process)
    pub name : Option<String>,
    pub content : ProcessContentRef
}

impl Process {
    pub fn new(id: BpmnId, name: Option<String>, content: ProcessContentRef) -> Self {
        Self { id, name, content }
    }
}




#[derive(Debug)]
pub struct Diagram {
    pub top_level_processes : BTreeMap<BpmnId,Process>,
    pub events : HashMap<BpmnId,Event>,
    pub activities : HashMap<BpmnId,Activity>,
    pub gateways : HashMap<BpmnId,Gateway>,
    pub sequence_flows : HashMap<BpmnId,Flow>,
    pub message_flows  : HashMap<BpmnId,Flow>,
    pub data : HashMap<BpmnId,String>
}

impl Diagram {
    pub fn new(
        top_level_processes: BTreeMap<BpmnId,Process>, 
        events: HashMap<BpmnId,Event>, 
        activities: HashMap<BpmnId,Activity>, 
        gateways: HashMap<BpmnId,Gateway>, 
        sequence_flows: HashMap<BpmnId,Flow>, 
        message_flows: HashMap<BpmnId,Flow>, 
        data: HashMap<BpmnId,String>) -> Self {
        Self { top_level_processes, events, activities, gateways, sequence_flows, message_flows, data }
    }
}


