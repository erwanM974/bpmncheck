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





/// BPMN Symbols
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Symbol {
    Cancel,
    Compensation,
    Conditional,
    Error,
    Escalation,
    Link,
    Message,
    Signal,
    Terminate,
    Timer,
}


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Event {
    pub event_type: EventType,
    pub symbol: Option<Symbol>,
    pub id: BpmnId,
    pub name: Option<String>
}

impl Event {
    pub fn new(event_type: EventType, symbol: Option<Symbol>, id: BpmnId, name: Option<String>) -> Self {
        Self { event_type, symbol, id, name }
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventType {
    Boundary(BpmnId),
    End,
    IntermediateCatch,
    IntermediateThrow,
    Start,
}

