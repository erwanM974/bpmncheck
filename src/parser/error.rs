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



#[derive(thiserror::Error, Debug)]
pub enum BpmnParsingError {
    #[error("CouldNotFindDataAssociation")]
    CouldNotFindDataAssociation,
    #[error("After the start tag '{}', we expect text", .tag)]
    ExpectedTextStart{tag:&'static str},
    #[error("After some text, we expect the end tag '{}'", .tag)]
    ExpectedTextEnd{tag:&'static str},
    #[error("ExpectedEventType")]
    ExpectedEventType,
    #[error("ExpectedGatewayType")]
    ExpectedGatewayType,
    #[error("attribute {} missing under {}", .att, .parent)]
    MissingAttribute{att:&'static str,parent:&'static str},
    #[error(transparent)]
    Xml(#[from] xml::reader::Error),
    #[error("CouldNotOpenFile")]
    CouldNotOpenFile,
    #[error("UnknownActivityType")]
    UnknownActivityType,
}




