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


use std::{collections::HashMap, io::BufRead};


use xml::{reader::XmlEvent, EventReader};

use crate::{model::{event::{Event, EventType}, flow::{Flow, FlowKind}, gateway::{Gateway, GatewayType}, id::BpmnId}, parser::{error::BpmnParsingError, grammar::*}};







pub fn read_flow(
    kind : FlowKind,
    mut attrs : HashMap<String,String>
) -> Result<Flow,BpmnParsingError> {
    let id: String = attrs.remove(BPMN_ID).ok_or(BpmnParsingError::MissingAttribute{att:BPMN_ID,parent:BPMN_MESSAGE_FLOW})?;
    let source_ref = attrs.remove(BPMN_SOURCE_REF).ok_or(BpmnParsingError::MissingAttribute{att:BPMN_SOURCE_REF,parent:BPMN_MESSAGE_FLOW})?;
    let target_ref = attrs.remove(BPMN_TARGET_REF).ok_or(BpmnParsingError::MissingAttribute{att:BPMN_TARGET_REF,parent:BPMN_MESSAGE_FLOW})?;
    let flow = Flow::new(
        BpmnId::new(id), 
        kind,
        None,
        BpmnId::new(source_ref),
        BpmnId::new(target_ref)
    );
    Ok(flow)
}

pub fn read_data_object_reference(
    mut attrs : HashMap<String,String>
) -> Result<(BpmnId,String),BpmnParsingError> {
    let id: String = attrs.remove(BPMN_ID).ok_or(BpmnParsingError::MissingAttribute{att:BPMN_ID,parent:BPMN_DATA_OBJECT_REFERENCE})?;
    let name = attrs.remove(BPMN_NAME).ok_or(BpmnParsingError::MissingAttribute{att:BPMN_NAME,parent:BPMN_DATA_OBJECT_REFERENCE})?;
    Ok((BpmnId::new(id),name))
}



pub fn read_event<R: BufRead>(
    reader: &mut EventReader<R>,
    event_type : &str,
    mut attrs : HashMap<String,String>
) -> Result<Event,BpmnParsingError> {
    let (static_tag ,kind) = match event_type {
        BPMN_START_EVENT              => Ok((BPMN_START_EVENT,EventType::Start)),
        BPMN_END_EVENT                => Ok((BPMN_END_EVENT,EventType::End)),
        BPMN_INTERMEDIATE_CATCH_EVENT => Ok((BPMN_INTERMEDIATE_CATCH_EVENT,EventType::IntermediateCatch)),
        BPMN_INTERMEDIATE_THROW_EVENT => Ok((BPMN_INTERMEDIATE_THROW_EVENT,EventType::IntermediateThrow)),
        BPMN_BOUNDARY_EVENT           => {
            let attached : String = attrs
                .remove(BPMN_BOUNDARY_ATTACHED_REFERENCE)
                .ok_or(BpmnParsingError::MissingAttribute{att:BPMN_BOUNDARY_ATTACHED_REFERENCE,parent:BPMN_BOUNDARY_EVENT})?;
            Ok((BPMN_BOUNDARY_EVENT,EventType::Boundary(BpmnId::new(attached))))
        },
        _ => {
            Err(BpmnParsingError::ExpectedEventType)
        }
    }?;
    let id: String = attrs.remove(BPMN_ID).ok_or(BpmnParsingError::MissingAttribute{att:BPMN_ID,parent:static_tag})?;
    let evt = Event::new(
        kind,
        None,
        BpmnId::new(id),
        attrs.remove(BPMN_NAME)
    );
    loop {
        match reader.next() {
            Err(e) => {return Err(BpmnParsingError::Xml(e))}
            Ok(XmlEvent::EndElement{name}) => {
                if name.local_name == event_type {
                    break;
                }
            }
            _ => {}
        }
    }
    Ok(evt)
}




pub fn read_gate<R: BufRead>(
    reader: &mut EventReader<R>,
    gate_type : &str,
    mut attrs : HashMap<String,String>
) -> Result<Gateway,BpmnParsingError> {
    let (static_tag ,kind) = match gate_type {
        BPMN_PARALLEL_GATEWAY  => Ok((BPMN_PARALLEL_GATEWAY,GatewayType::Parallel)),
        BPMN_EXCLUSIVE_GATEWAY => Ok((BPMN_EXCLUSIVE_GATEWAY,GatewayType::Exclusive)),
        BPMN_INCLUSIVE_GATEWAY => Ok((BPMN_INCLUSIVE_GATEWAY,GatewayType::Inclusive)),
        _ => {
            Err(BpmnParsingError::ExpectedGatewayType)
        }
    }?;
    let id: String = attrs.remove(BPMN_ID).ok_or(BpmnParsingError::MissingAttribute{att:BPMN_ID,parent:static_tag})?;
    let evt = Gateway::new(
        kind,
        BpmnId::new(id),
        attrs.remove(BPMN_NAME)
    );
    loop {
        match reader.next() {
            Err(e) => {return Err(BpmnParsingError::Xml(e))}
            Ok(XmlEvent::EndElement{name}) => {
                if name.local_name == gate_type {
                    break;
                }
            }
            _ => {}
        }
    }
    Ok(evt)
}



pub fn read_text_then_close<R: BufRead>(
    reader: &mut EventReader<R>,
    expected_end_tag : &'static str
) -> Result<String,BpmnParsingError> {
    let got_text = match reader.next() {
        Err(e) => Err(BpmnParsingError::Xml(e)),
        Ok(XmlEvent::Characters(txt)) => {
            Ok(txt)
        },
        _ => {
            Err(BpmnParsingError::ExpectedTextStart{tag:expected_end_tag})
        }
    }?;
    match reader.next() {
        Err(e) => Err(BpmnParsingError::Xml(e)),
        Ok(XmlEvent::EndElement{name}) => {
            if name.local_name == expected_end_tag {
                Ok(got_text)
            } else {
                Err(BpmnParsingError::ExpectedTextEnd{tag:expected_end_tag})
            }
        },
        _ => {
            Err(BpmnParsingError::ExpectedTextEnd{tag:expected_end_tag})
        }
    }
}




pub fn read_data_association<R: BufRead>(
    reader: &mut EventReader<R>,
    is_input : bool
) -> Result<BpmnId,BpmnParsingError> {
    let mut got_text = None;
    loop {
        match reader.next() {
            Err(e) => return Err(BpmnParsingError::Xml(e)),
            Ok(XmlEvent::StartElement{name,..}) => {
                match name.local_name.as_str() {
                    BPMN_SOURCE_REF => {
                        if is_input {
                            let txt = read_text_then_close(reader,BPMN_SOURCE_REF)?;
                            got_text = Some(txt);
                        }
                    },
                    BPMN_TARGET_REF => {
                        if !is_input {
                            let txt = read_text_then_close(reader,BPMN_TARGET_REF)?;
                            got_text = Some(txt);
                        }
                    }
                    _ => {}
                }
            },
            Ok(XmlEvent::EndElement{name}) => {
                if is_input {
                    if name.local_name.as_str() == BPMN_DATA_INPUT_ASSOCIATION {
                        break;
                    }
                } else if name.local_name.as_str() == BPMN_DATA_OUTPUT_ASSOCIATION {
                    break;
                }
            }
            _ => {}
        }
    }
    match got_text {
        None => {
            Err(BpmnParsingError::CouldNotFindDataAssociation)
        },
        Some(txt) => {
            Ok(BpmnId::new(txt))
        }
    }
}