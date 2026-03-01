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


use std::collections::HashSet;

use crate::model::{diagram::ProcessContentRef, id::BpmnId};




#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Activity {
    pub activity_type: ActivityType,
    pub id: BpmnId,
    pub name: Option<String>,
    pub input_data : HashSet<BpmnId>,
    pub output_data : HashSet<BpmnId>
}

impl Activity {
    pub fn new(activity_type: ActivityType, id: BpmnId, name: Option<String>, input_data: HashSet<BpmnId>, output_data: HashSet<BpmnId>) -> Self {
        Self { activity_type, id, name, input_data, output_data }
    }
}




#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivityType {
    Task(TaskKind),
    //CallActivity,
    SubProcess(ProcessContentRef),
}

impl ActivityType {
    pub fn is_task(&self) -> bool {
        match self {
            ActivityType::Task(_) => true,
            ActivityType::SubProcess(_) => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskKind {
    DefaultTask,
    ServiceTask,
    UserTask,
    ScriptTask,
    ReceiveTask,
    SendTask,
    ManualTask,
    BusinessRuleTask,
}