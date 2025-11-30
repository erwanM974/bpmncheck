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


use crate::model::id::BpmnId;



#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FlowKind {
    Sequence,
    Message 
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Flow {
    pub id: BpmnId,
    pub kind : FlowKind,
    pub name: Option<String>,
    pub source_ref: BpmnId,
    pub target_ref: BpmnId
}

impl Flow {
    pub fn new(id: BpmnId, kind: FlowKind, name: Option<String>, source_ref: BpmnId, target_ref: BpmnId) -> Self {
        Self { id, kind, name, source_ref, target_ref }
    }
}
