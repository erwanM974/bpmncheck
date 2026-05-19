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


use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use bpmncheck::{
    model::{
        activity::{Activity, ActivityType, TaskKind},
        diagram::{Diagram, Process, ProcessContentRef},
        event::{Event, EventType},
        flow::{Flow, FlowKind},
        id::BpmnId,
    },
    wellformedness::{
        check::check_model_well_formedness,
        error::{ModelWellFormednessViolation, ProcessWellFormednessViolation},
    },
};
use map_macro::{btree_map, btree_set, hash_map};


fn id(s: &str) -> BpmnId {
    BpmnId::new(s.to_string())
}

fn start_event(eid: &str) -> Event {
    Event::new(EventType::Start, None, id(eid), None)
}

fn end_event(eid: &str) -> Event {
    Event::new(EventType::End, None, id(eid), None)
}

fn throw_event(eid: &str) -> Event {
    Event::new(EventType::IntermediateThrow, None, id(eid), None)
}

fn catch_event(eid: &str) -> Event {
    Event::new(EventType::IntermediateCatch, None, id(eid), None)
}

fn task(aid: &str) -> Activity {
    Activity::new(ActivityType::Task(TaskKind::DefaultTask), id(aid), None, HashSet::new(), HashSet::new())
}

fn seq_flow(fid: &str, from: &str, to: &str) -> Flow {
    Flow::new(id(fid), FlowKind::Sequence, None, id(from), id(to))
}

fn msg_flow(fid: &str, from: &str, to: &str) -> Flow {
    Flow::new(id(fid), FlowKind::Message, None, id(from), id(to))
}


// NoTopLevelProcess 

#[test]
fn test_wf_ko_no_top_level_process() {
    let bpmn = Diagram::new(
        BTreeMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
    );
    assert_eq!(
        check_model_well_formedness(&bpmn),
        Err(ModelWellFormednessViolation::NoTopLevelProcess)
    );
}


// ProcessHasNoStartEvent
//
// Use IntermediateCatch (incoming=0, outgoing=1) → End (incoming=1, outgoing=0).
// All event flow counts are valid, but there is no Start event.
// check_process_well_formedness returns ProcessHasNoStartEvent before any
// message-flow check is reached.

#[test]
fn test_wf_ko_no_start_event() {
    let events = hash_map! {
        id("e_catch") => catch_event("e_catch"),
        id("e_end")   => end_event("e_end")
    };
    let flows = hash_map! {
        id("f1") => seq_flow("f1", "e_catch", "e_end")
    };
    let content = ProcessContentRef::new(
        btree_set! { id("e_catch"), id("e_end") },
        BTreeSet::new(),
        BTreeSet::new(),
        btree_set! { id("f1") },
    );
    let proc = Process::new(id("proc"), None, content);
    let bpmn = Diagram::new(
        btree_map! { id("proc") => proc },
        events,
        HashMap::new(),
        HashMap::new(),
        flows,
        HashMap::new(),
        HashMap::new(),
    );
    assert_eq!(
        check_model_well_formedness(&bpmn),
        Err(ModelWellFormednessViolation::ProcessViolation(
            id("proc"),
            ProcessWellFormednessViolation::ProcessHasNoStartEvent
        ))
    );
}


// ProcessHasNoEndEvent 
//
// Use Start (incoming=0, outgoing=1) → IntermediateThrow (incoming=1, outgoing=0).
// All event flow counts are valid, but there is no End event.
// check_process_well_formedness returns ProcessHasNoEndEvent before any
// message-flow check is reached.

#[test]
fn test_wf_ko_no_end_event() {
    let events = hash_map! {
        id("e_start") => start_event("e_start"),
        id("e_throw") => throw_event("e_throw")
    };
    let flows = hash_map! {
        id("f1") => seq_flow("f1", "e_start", "e_throw")
    };
    let content = ProcessContentRef::new(
        btree_set! { id("e_start"), id("e_throw") },
        BTreeSet::new(),
        BTreeSet::new(),
        btree_set! { id("f1") },
    );
    let proc = Process::new(id("proc"), None, content);
    let bpmn = Diagram::new(
        btree_map! { id("proc") => proc },
        events,
        HashMap::new(),
        HashMap::new(),
        flows,
        HashMap::new(),
        HashMap::new(),
    );
    assert_eq!(
        check_model_well_formedness(&bpmn),
        Err(ModelWellFormednessViolation::ProcessViolation(
            id("proc"),
            ProcessWellFormednessViolation::ProcessHasNoEndEvent
        ))
    );
}


// WrongNumberOfIncomingFlows (start event)
//
// f_extra gives the start event 1 incoming flow (expected 0).
// BTreeSet iteration order is "e_end" < "e_start", so e_end is checked first
// (incoming=1 ✓, outgoing=0 ✓), then e_start triggers the error.

#[test]
fn test_wf_ko_wrong_incoming_flows_event() {
    let events = hash_map! {
        id("e_start") => start_event("e_start"),
        id("e_end")   => end_event("e_end")
    };
    let activities = hash_map! { id("act") => task("act") };
    let flows = hash_map! {
        id("f1")      => seq_flow("f1",      "e_start", "act"),
        id("f2")      => seq_flow("f2",      "act",     "e_end"),
        id("f_extra") => seq_flow("f_extra", "act",     "e_start")
    };
    let content = ProcessContentRef::new(
        btree_set! { id("e_start"), id("e_end") },
        btree_set! { id("act") },
        BTreeSet::new(),
        btree_set! { id("f1"), id("f2"), id("f_extra") },
    );
    let proc = Process::new(id("proc"), None, content);
    let bpmn = Diagram::new(
        btree_map! { id("proc") => proc },
        events,
        activities,
        HashMap::new(),
        flows,
        HashMap::new(),
        HashMap::new(),
    );
    assert_eq!(
        check_model_well_formedness(&bpmn),
        Err(ModelWellFormednessViolation::ProcessViolation(
            id("proc"),
            ProcessWellFormednessViolation::WrongNumberOfIncomingFlows(id("e_start"))
        ))
    );
}


// WrongNumberOfOutgoingFlows (end event) 
//
// f_extra gives the end event 1 outgoing flow (expected 0).
// "e_end" < "e_start" in BTree order, so e_end is visited first and triggers the error.

#[test]
fn test_wf_ko_wrong_outgoing_flows_event() {
    let events = hash_map! {
        id("e_start") => start_event("e_start"),
        id("e_end")   => end_event("e_end")
    };
    let activities = hash_map! { id("act") => task("act") };
    let flows = hash_map! {
        id("f1")      => seq_flow("f1",      "e_start", "act"),
        id("f2")      => seq_flow("f2",      "act",     "e_end"),
        id("f_extra") => seq_flow("f_extra", "e_end",   "act")
    };
    let content = ProcessContentRef::new(
        btree_set! { id("e_start"), id("e_end") },
        btree_set! { id("act") },
        BTreeSet::new(),
        btree_set! { id("f1"), id("f2"), id("f_extra") },
    );
    let proc = Process::new(id("proc"), None, content);
    let bpmn = Diagram::new(
        btree_map! { id("proc") => proc },
        events,
        activities,
        HashMap::new(),
        flows,
        HashMap::new(),
        HashMap::new(),
    );
    assert_eq!(
        check_model_well_formedness(&bpmn),
        Err(ModelWellFormednessViolation::ProcessViolation(
            id("proc"),
            ProcessWellFormednessViolation::WrongNumberOfOutgoingFlows(id("e_end"))
        ))
    );
}


// WrongNumberOfIncomingFlows (activity)
//
// "act" is in direct_child_activities but no sequence flow targets it (0 incoming).

#[test]
fn test_wf_ko_wrong_incoming_flows_activity() {
    let events = hash_map! {
        id("e_start") => start_event("e_start"),
        id("e_end")   => end_event("e_end")
    };
    let activities = hash_map! { id("act") => task("act") };
    let flows = hash_map! {
        id("f1") => seq_flow("f1", "e_start", "e_end")
    };
    let content = ProcessContentRef::new(
        btree_set! { id("e_start"), id("e_end") },
        btree_set! { id("act") },
        BTreeSet::new(),
        btree_set! { id("f1") },
    );
    let proc = Process::new(id("proc"), None, content);
    let bpmn = Diagram::new(
        btree_map! { id("proc") => proc },
        events,
        activities,
        HashMap::new(),
        flows,
        HashMap::new(),
        HashMap::new(),
    );
    assert_eq!(
        check_model_well_formedness(&bpmn),
        Err(ModelWellFormednessViolation::ProcessViolation(
            id("proc"),
            ProcessWellFormednessViolation::WrongNumberOfIncomingFlows(id("act"))
        ))
    );
}


// WrongNumberOfOutgoingFlows (activity)
//
// "act" has 1 incoming flow but 0 outgoing flows (expected 1).
// Two start events are used so each start event has exactly 1 outgoing flow,
// avoiding a WrongNumberOfOutgoingFlows error on the start event first.

#[test]
fn test_wf_ko_wrong_outgoing_flows_activity() {
    let events = hash_map! {
        id("e_start1") => start_event("e_start1"),
        id("e_start2") => start_event("e_start2"),
        id("e_end")    => end_event("e_end")
    };
    let activities = hash_map! { id("act") => task("act") };
    let flows = hash_map! {
        id("f1") => seq_flow("f1", "e_start1", "act"),
        id("f2") => seq_flow("f2", "e_start2", "e_end")
    };
    let content = ProcessContentRef::new(
        btree_set! { id("e_start1"), id("e_start2"), id("e_end") },
        btree_set! { id("act") },
        BTreeSet::new(),
        btree_set! { id("f1"), id("f2") },
    );
    let proc = Process::new(id("proc"), None, content);
    let bpmn = Diagram::new(
        btree_map! { id("proc") => proc },
        events,
        activities,
        HashMap::new(),
        flows,
        HashMap::new(),
        HashMap::new(),
    );
    assert_eq!(
        check_model_well_formedness(&bpmn),
        Err(ModelWellFormednessViolation::ProcessViolation(
            id("proc"),
            ProcessWellFormednessViolation::WrongNumberOfOutgoingFlows(id("act"))
        ))
    );
}


// ThrowEventHasNoSuccessor
//
// IntermediateThrow: incoming=1, outgoing=0 in sequence flows.
// proc2 uses two start events: one leads to the valid end path, one leads to e_throw.
// No message flow for e_throw → ThrowEventHasNoSuccessor.
// Both process WF checks pass; the model-level event loop fires the error.

#[test]
fn test_wf_ko_throw_event_has_no_successor() {
    let throw_id = id("e_throw");
    let events_p1: HashMap<BpmnId, Event> = hash_map! {
        id("e_start") => start_event("e_start"),
        id("e_end")   => end_event("e_end")
    };
    let activities_p1: HashMap<BpmnId, Activity> = hash_map! { id("act") => task("act") };
    let flows_p1: HashMap<BpmnId, Flow> = hash_map! {
        id("f1") => seq_flow("f1", "e_start", "act"),
        id("f2") => seq_flow("f2", "act", "e_end")
    };
    let content_p1 = ProcessContentRef::new(
        btree_set! { id("e_start"), id("e_end") },
        btree_set! { id("act") },
        BTreeSet::new(),
        btree_set! { id("f1"), id("f2") },
    );
    // proc2: start2a → end2 (valid end path); start2b → e_throw (1 in, 0 out ✓)
    let events_p2: HashMap<BpmnId, Event> = hash_map! {
        id("e_start2a") => start_event("e_start2a"),
        id("e_start2b") => start_event("e_start2b"),
        id("e_end2")    => end_event("e_end2"),
        throw_id.clone()    => throw_event("e_throw")
    };
    let flows_p2: HashMap<BpmnId, Flow> = hash_map! {
        id("f3") => seq_flow("f3", "e_start2a", "e_end2"),
        id("f4") => seq_flow("f4", "e_start2b", "e_throw")
    };
    let content_p2 = ProcessContentRef::new(
        btree_set! { id("e_start2a"), id("e_start2b"), id("e_end2"), throw_id.clone() },
        BTreeSet::new(),
        BTreeSet::new(),
        btree_set! { id("f3"), id("f4") },
    );

    let mut all_evts = events_p1;
    for (k, v) in events_p2 { all_evts.insert(k, v); }
    let mut all_flows = flows_p1;
    for (k, v) in flows_p2 { all_flows.insert(k, v); }

    let bpmn = Diagram::new(
        btree_map! {
            id("proc1") => Process::new(id("proc1"), None, content_p1),
            id("proc2") => Process::new(id("proc2"), None, content_p2)
        },
        all_evts,
        activities_p1,
        HashMap::new(),
        all_flows,
        HashMap::new(),
        HashMap::new(),
    );
    assert_eq!(
        check_model_well_formedness(&bpmn),
        Err(ModelWellFormednessViolation::ThrowEventHasNoSuccessor(throw_id))
    );
}


// CatchEventHasNoPredecessor 
//
// IntermediateCatch: incoming=0, outgoing=1 in sequence flows.
// proc2: start2 → end2 (valid path); e_catch → end3 (0 in, 1 out ✓).
// No message flow targets e_catch → CatchEventHasNoPredecessor.

#[test]
fn test_wf_ko_catch_event_has_no_predecessor() {
    let catch_id = id("e_catch");
    let events_p1: HashMap<BpmnId, Event> = hash_map! {
        id("e_start") => start_event("e_start"),
        id("e_end")   => end_event("e_end")
    };
    let activities_p1: HashMap<BpmnId, Activity> = hash_map! { id("act") => task("act") };
    let flows_p1: HashMap<BpmnId, Flow> = hash_map! {
        id("f1") => seq_flow("f1", "e_start", "act"),
        id("f2") => seq_flow("f2", "act", "e_end")
    };
    let content_p1 = ProcessContentRef::new(
        btree_set! { id("e_start"), id("e_end") },
        btree_set! { id("act") },
        BTreeSet::new(),
        btree_set! { id("f1"), id("f2") },
    );
    let events_p2: HashMap<BpmnId, Event> = hash_map! {
        id("e_start2") => start_event("e_start2"),
        id("e_end2")   => end_event("e_end2"),
        id("e_end3")   => end_event("e_end3"),
        catch_id.clone()   => catch_event("e_catch")
    };
    let flows_p2: HashMap<BpmnId, Flow> = hash_map! {
        id("f3") => seq_flow("f3", "e_start2", "e_end2"),
        id("f4") => seq_flow("f4", "e_catch",  "e_end3")
    };
    let content_p2 = ProcessContentRef::new(
        btree_set! { id("e_start2"), id("e_end2"), id("e_end3"), catch_id.clone() },
        BTreeSet::new(),
        BTreeSet::new(),
        btree_set! { id("f3"), id("f4") },
    );

    let mut all_evts = events_p1;
    for (k, v) in events_p2 { all_evts.insert(k, v); }
    let mut all_flows = flows_p1;
    for (k, v) in flows_p2 { all_flows.insert(k, v); }

    let bpmn = Diagram::new(
        btree_map! {
            id("proc1") => Process::new(id("proc1"), None, content_p1),
            id("proc2") => Process::new(id("proc2"), None, content_p2)
        },
        all_evts,
        activities_p1,
        HashMap::new(),
        all_flows,
        HashMap::new(),
        HashMap::new(),
    );
    assert_eq!(
        check_model_well_formedness(&bpmn),
        Err(ModelWellFormednessViolation::CatchEventHasNoPredecessor(catch_id))
    );
}


// ThrowEventHasMoreThanOneSuccessor 
//
// e_throw is connected to two catch events via two message flows.
// All three processes are structurally valid (each event has its expected flow counts).

#[test]
fn test_wf_ko_throw_event_has_more_than_one_successor() {
    let events: HashMap<BpmnId, Event> = hash_map! {
        id("e_start1a") => start_event("e_start1a"),
        id("e_start1b") => start_event("e_start1b"),
        id("e_end1")    => end_event("e_end1"),
        id("e_throw")   => throw_event("e_throw"),
        id("e_start2")  => start_event("e_start2"),
        id("e_end2")    => end_event("e_end2"),
        id("e_end2b")   => end_event("e_end2b"),
        id("e_catch1")  => catch_event("e_catch1"),
        id("e_start3")  => start_event("e_start3"),
        id("e_end3")    => end_event("e_end3"),
        id("e_end3b")   => end_event("e_end3b"),
        id("e_catch2")  => catch_event("e_catch2")
    };
    let seq_flows: HashMap<BpmnId, Flow> = hash_map! {
        id("f1") => seq_flow("f1", "e_start1a", "e_end1"),
        id("f2") => seq_flow("f2", "e_start1b", "e_throw"),
        id("f3") => seq_flow("f3", "e_start2",  "e_end2"),
        id("f4") => seq_flow("f4", "e_catch1",  "e_end2b"),
        id("f5") => seq_flow("f5", "e_start3",  "e_end3"),
        id("f6") => seq_flow("f6", "e_catch2",  "e_end3b")
    };
    let content1 = ProcessContentRef::new(
        btree_set! { id("e_start1a"), id("e_start1b"), id("e_end1"), id("e_throw") },
        BTreeSet::new(), BTreeSet::new(),
        btree_set! { id("f1"), id("f2") },
    );
    let content2 = ProcessContentRef::new(
        btree_set! { id("e_start2"), id("e_end2"), id("e_end2b"), id("e_catch1") },
        BTreeSet::new(), BTreeSet::new(),
        btree_set! { id("f3"), id("f4") },
    );
    let content3 = ProcessContentRef::new(
        btree_set! { id("e_start3"), id("e_end3"), id("e_end3b"), id("e_catch2") },
        BTreeSet::new(), BTreeSet::new(),
        btree_set! { id("f5"), id("f6") },
    );
    let msg_flows: HashMap<BpmnId, Flow> = hash_map! {
        id("mf1") => msg_flow("mf1", "e_throw", "e_catch1"),
        id("mf2") => msg_flow("mf2", "e_throw", "e_catch2")
    };
    let bpmn = Diagram::new(
        btree_map! {
            id("proc1") => Process::new(id("proc1"), None, content1),
            id("proc2") => Process::new(id("proc2"), None, content2),
            id("proc3") => Process::new(id("proc3"), None, content3)
        },
        events,
        HashMap::new(),
        HashMap::new(),
        seq_flows,
        msg_flows,
        HashMap::new(),
    );
    assert_eq!(
        check_model_well_formedness(&bpmn),
        Err(ModelWellFormednessViolation::ThrowEventHasMoreThanOneSuccessor(id("e_throw")))
    );
}


//  CatchEventHasMoreThanOnePredecessor ─
//
// e_throw1 and e_throw2 each send a message flow to the same e_catch.
// Three start events in proc2 give each throw exactly 1 incoming sequence flow.

#[test]
fn test_wf_ko_catch_event_has_more_than_one_predecessor() {
    let events: HashMap<BpmnId, Event> = hash_map! {
        id("e_start2a") => start_event("e_start2a"),
        id("e_start2b") => start_event("e_start2b"),
        id("e_start2c") => start_event("e_start2c"),
        id("e_end2")    => end_event("e_end2"),
        id("e_throw1")  => throw_event("e_throw1"),
        id("e_throw2")  => throw_event("e_throw2"),
        id("e_start3")  => start_event("e_start3"),
        id("e_end3")    => end_event("e_end3"),
        id("e_end3b")   => end_event("e_end3b"),
        id("e_catch")   => catch_event("e_catch")
    };
    let seq_flows: HashMap<BpmnId, Flow> = hash_map! {
        id("f1") => seq_flow("f1", "e_start2a", "e_end2"),
        id("f2") => seq_flow("f2", "e_start2b", "e_throw1"),
        id("f3") => seq_flow("f3", "e_start2c", "e_throw2"),
        id("f4") => seq_flow("f4", "e_start3",  "e_end3"),
        id("f5") => seq_flow("f5", "e_catch",   "e_end3b")
    };
    let content2 = ProcessContentRef::new(
        btree_set! {
            id("e_start2a"), id("e_start2b"), id("e_start2c"),
            id("e_end2"), id("e_throw1"), id("e_throw2")
        },
        BTreeSet::new(), BTreeSet::new(),
        btree_set! { id("f1"), id("f2"), id("f3") },
    );
    let content3 = ProcessContentRef::new(
        btree_set! { id("e_start3"), id("e_end3"), id("e_end3b"), id("e_catch") },
        BTreeSet::new(), BTreeSet::new(),
        btree_set! { id("f4"), id("f5") },
    );
    let msg_flows: HashMap<BpmnId, Flow> = hash_map! {
        id("mf1") => msg_flow("mf1", "e_throw1", "e_catch"),
        id("mf2") => msg_flow("mf2", "e_throw2", "e_catch")
    };
    let bpmn = Diagram::new(
        btree_map! {
            id("proc2") => Process::new(id("proc2"), None, content2),
            id("proc3") => Process::new(id("proc3"), None, content3)
        },
        events,
        HashMap::new(),
        HashMap::new(),
        seq_flows,
        msg_flows,
        HashMap::new(),
    );
    assert_eq!(
        check_model_well_formedness(&bpmn),
        Err(ModelWellFormednessViolation::CatchEventHasMoreThanOnePredecessor(id("e_catch")))
    );
}
