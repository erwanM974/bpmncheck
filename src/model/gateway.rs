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
pub struct Gateway {
    pub gateway_type: GatewayType,
    pub id: BpmnId,
    pub name: Option<String>
}

impl Gateway {
    pub fn new(gateway_type: GatewayType, id: BpmnId, name: Option<String>) -> Self {
        Self { gateway_type, id, name }
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayType {
    Exclusive,
    Inclusive,
    Parallel
}
