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



pub const BPMN_DEFINITIONS              : &str = "definitions";
pub const BPMN_COLLABORATION            : &str = "collaboration";
pub const BPMN_PROCESS                  : &str = "process";
pub const BPMN_ID                       : &str = "id";

// Collaboration
pub const BPMN_PARTICIPANT              : &str = "participant";

// Participant
pub const BPMN_NAME                     : &str = "name";
pub const BPMN_PROCESS_REF              : &str = "processRef";

// Flow
pub const BPMN_MESSAGE_FLOW             : &str = "messageFlow";
pub const BPMN_SEQUENCE_FLOW            : &str = "sequenceFlow";
// Flow content
pub const BPMN_TARGET_REF               : &str = "targetRef";
pub const BPMN_SOURCE_REF               : &str = "sourceRef";


// Event
pub const BPMN_START_EVENT              : &str = "startEvent";
pub const BPMN_END_EVENT                : &str = "endEvent";
pub const BPMN_BOUNDARY_EVENT           : &str = "boundaryEvent";
pub const BPMN_INTERMEDIATE_CATCH_EVENT : &str = "intermediateCatchEvent";
pub const BPMN_INTERMEDIATE_THROW_EVENT : &str = "intermediateThrowEvent";

pub const BPMN_BOUNDARY_ATTACHED_REFERENCE : &str = "attachedToRef";


/* 
// Event symbol
pub const CANCEL_EVENT_DEFINITION       : &str = "cancelEventDefinition";
pub const COMPENSATE_EVENT_DEFINITION: &str = "compensateEventDefinition";
pub const CONDITIONAL_EVENT_DEFINITION: &str = "conditionalEventDefinition";
pub const ERROR_EVENT_DEFINITION: &str = "errorEventDefinition";
pub const ESCALATION_EVENT_DEFINITION: &str = "escalationEventDefinition";
pub const MESSAGE_EVENT_DEFINITION: &str = "messageEventDefinition";
pub const LINK_EVENT_DEFINITION: &str = "linkEventDefinition";
pub const SIGNAL_EVENT_DEFINITION: &str = "signalEventDefinition";
pub const TERMINATE_EVENT_DEFINITION: &str = "terminateEventDefinition";
pub const TIMER_EVENT_DEFINITION: &str = "timerEventDefinition";
*/

// Activities
pub const BPMN_TASK               : &str = "task";
pub const BPMN_SERVICE_TASK       : &str = "serviceTask";
pub const BPMN_USER_TASK          : &str = "userTask";
pub const BPMN_SCRIPT_TASK        : &str = "scriptTask";
pub const BPMN_RECEIVE_TASK       : &str = "receiveTask";
pub const BPMN_SEND_TASK          : &str = "sendTask";
pub const BPMN_MANUAL_TASK        : &str = "manualTask";
pub const BPMN_BUSINESS_RULE_TASK : &str = "businessRuleTask";
pub const BPMN_CALL_ACTIVITY      : &str = "callActivity";
pub const BPMN_SUB_PROCESS        : &str = "subProcess";


// Gateway
pub const BPMN_EXCLUSIVE_GATEWAY  : &str = "exclusiveGateway";
pub const BPMN_PARALLEL_GATEWAY   : &str = "parallelGateway";
pub const BPMN_INCLUSIVE_GATEWAY  : &str = "inclusiveGateway";


// Data
pub const BPMN_DATA_OBJECT_REFERENCE   : &str = "dataObjectReference";
pub const BPMN_DATA_INPUT_ASSOCIATION  : &str = "dataInputAssociation";
pub const BPMN_DATA_OUTPUT_ASSOCIATION : &str = "dataOutputAssociation";


