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



use xml::reader::XmlEvent;
use xml::EventReader;

use crate::model::diagram::{Diagram, Process, ProcessContentRef};
use crate::model::flow::Flow;
use crate::model::id::BpmnId;
use crate::model::*;
use crate::parser::elements::read_flow;
use crate::parser::error::BpmnParsingError;
use crate::parser::grammar::*;
use crate::parser::process::{read_activity, NestedElements};
use crate::parser::util::collect_attributes;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{BufRead, BufReader};





pub fn read_bpmn_diagram_from_file_path(path : &str) -> Result<Diagram,BpmnParsingError> {
    match File::open(path) {
        Ok(file) => {
            let file = BufReader::new(file);
            let reader = EventReader::new(file);
            read_bpmn(reader)
        },
        Err(_) => {
            Err(BpmnParsingError::CouldNotOpenFile)
        },
    }
}




// Read BPMN content and return the Diagram
pub fn read_bpmn<R: BufRead>(mut reader: EventReader<R>) -> Result<Diagram, BpmnParsingError> {
    loop {
        match reader.next() {
            Ok(XmlEvent::StartElement{name,..}) => {
                if name.local_name.as_str() == BPMN_DEFINITIONS {
                    return read_bpmn_content(reader);
                }
            },
            Ok(_) => {
                // 
            },
            Err(e) => {
                return Err(BpmnParsingError::Xml(e))
            }
        }
    }
}




// Read BPMN content and return the Diagram
pub fn read_bpmn_content<R: BufRead>(mut reader: EventReader<R>) -> Result<Diagram, BpmnParsingError> {
    let mut collab : Option<(HashMap<String,String>,Vec<Flow>)> = None;
    let mut top_level_processes : BTreeMap<BpmnId,Process> = BTreeMap::new();
    let mut content = NestedElements::new_empty();
    loop {
        match reader.next() {
            Ok(XmlEvent::StartElement{name,attributes,..}) => match name.local_name.as_str() {
                BPMN_COLLABORATION => {
                    let got_collab = read_collaboration(&mut reader)?;
                    collab = Some(got_collab);
                },
                BPMN_PROCESS => {
                    let mut attrs = collect_attributes(attributes);
                    let id: String = attrs.remove(BPMN_ID).ok_or(BpmnParsingError::MissingAttribute{att:BPMN_ID,parent:BPMN_PROCESS})?;
                    // ***
                    let got_proc = read_activity(&mut reader,  BPMN_PROCESS)?;
                    content.updated_with(got_proc.nested_elements);
                    let proc = Process::new(
                        BpmnId::new(id), 
                        None, 
                        ProcessContentRef::new(
                            got_proc.direct_child_events,
                            got_proc.direct_child_activities,
                            got_proc.direct_child_gateways,
                            got_proc.direct_child_flows,
                        )
                    );
                    top_level_processes.insert(proc.id.clone(),proc);
                }
                _ => {}
            },
            Ok(XmlEvent::EndElement { name }) => {
                if name.local_name.as_str() == BPMN_DEFINITIONS {
                    break;
                }
            },
            Err(e) => {
                return Err(BpmnParsingError::Xml(e))
            }
            _ => {}
        }
    }
    let mut message_flows = HashMap::new();
    if let Some((mut pref_to_name,msg_flows)) = collab {
        for (proc_id,proc) in top_level_processes.iter_mut() {
            if let Some(participant_name) = pref_to_name.remove(&proc_id.id) {
                proc.name = Some(participant_name);
            }
        }
        for f in msg_flows {
            message_flows.insert(f.id.clone(), f);
        }
    }
    Ok(
        Diagram::new(
            top_level_processes,
            content.events,
            content.activities,
            content.gateways,
            content.sequence_flows,
            message_flows,
            content.data
        )
    )
}










fn read_collaboration<R: BufRead>(
    reader: &mut EventReader<R>
) -> Result<(HashMap<String,String>,Vec<Flow>), BpmnParsingError> {
    let mut participants  = HashMap::new();
    let mut message_flows  = Vec::new();
    loop {
        match reader.next() {
            Ok(XmlEvent::StartElement{name,attributes,..}) => {
                match name.local_name.as_str() {
                    BPMN_PARTICIPANT => {
                        let mut attrs = collect_attributes(attributes);
                        let name = attrs.remove(BPMN_NAME).ok_or(BpmnParsingError::MissingAttribute{att:BPMN_NAME,parent:BPMN_PARTICIPANT})?;
                        let pref = attrs.remove(BPMN_PROCESS_REF).ok_or(BpmnParsingError::MissingAttribute{att:BPMN_PROCESS_REF,parent:BPMN_PARTICIPANT})?;
                        participants.insert(pref,name);
                    },
                    BPMN_MESSAGE_FLOW => {
                        let flow = read_flow(flow::FlowKind::Message, collect_attributes(attributes))?;
                        message_flows.push(flow);
                    },
                    _ => {}
                }
            },
            Ok(XmlEvent::EndElement{name}) => if name.local_name.as_str() == BPMN_COLLABORATION {
                break;
            }
            _ => {}
        }
    }
    Ok((participants,message_flows))
}







