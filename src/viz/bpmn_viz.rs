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


use graphviz_dot_builder::{edge::{edge::GraphVizEdge}, graph::graph::GraphVizDiGraph, item::{cluster::GraphVizCluster, node::{node::GraphVizNode, style::{GraphvizNodeStyleItem, GvNodeShape}}}, traits::DotBuildable};

use crate::model::{activity::ActivityType, diagram::{Diagram, ProcessContentRef}, event::EventType, gateway::GatewayType, id::BpmnId};




pub fn bpmn_subproc_repr(
    bpmn : &Diagram,
    proc_id : &BpmnId,
    proc_content : &ProcessContentRef
) -> GraphVizCluster {
    let mut cluster = GraphVizCluster::new(
        proc_id.id.clone(), 
        vec![], 
        vec![], 
        vec![]
    );
    for evt_id in &proc_content.direct_child_events {
        let evt = bpmn.events.get(evt_id).unwrap();
        let mut style = vec![
            GraphvizNodeStyleItem::Label("".to_string())
            ];
        match &evt.event_type {
            EventType::Boundary => {
                style.push(GraphvizNodeStyleItem::Shape(GvNodeShape::Hexagon));
            },
            EventType::End => {
                style.push(GraphvizNodeStyleItem::Shape(GvNodeShape::Circle));
                style.push(GraphvizNodeStyleItem::PenWidth(10));
            },
            EventType::IntermediateCatch => {
                style.push(GraphvizNodeStyleItem::Shape(GvNodeShape::DoubleCircle));
            },
            EventType::IntermediateThrow => {
                style.push(GraphvizNodeStyleItem::Shape(GvNodeShape::DoubleCircle));
            },
            EventType::Start => {
                style.push(GraphvizNodeStyleItem::Shape(GvNodeShape::Circle));
            },
        }
        let node = GraphVizNode::new(
            evt_id.id.clone(), 
            style
        );
        cluster.add_node(node);
    }
    for act_id in &proc_content.direct_child_activities {
        let act = bpmn.activities.get(act_id).unwrap();
        match &act.activity_type {
            ActivityType::SubProcess(sub_proc_content) => {
                let sub_cluster = bpmn_subproc_repr(bpmn,act_id,sub_proc_content);
                cluster.add_cluster(sub_cluster);
            },
            _ => {
                let mut style = vec![GraphvizNodeStyleItem::Shape(GvNodeShape::Rectangle)];
                if let Some(name) = &act.name {
                    style.push(GraphvizNodeStyleItem::Label(name.clone()));
                }
                let node = GraphVizNode::new(
                    act_id.id.clone(), 
                    style
                    );
                cluster.add_node(node);
            }
        }
    }
    for gt_id in &proc_content.direct_child_gateways {
        let gate = bpmn.gateways.get(gt_id).unwrap();
        let mut style = vec![GraphvizNodeStyleItem::Shape(GvNodeShape::Diamond)];
        match gate.gateway_type {
            GatewayType::Exclusive => {
                style.push(GraphvizNodeStyleItem::Label("X".to_string()));
            },
            GatewayType::Inclusive => {
                style.push(GraphvizNodeStyleItem::Label("O".to_string()));
            },
            GatewayType::Parallel => {
                style.push(GraphvizNodeStyleItem::Label("+".to_string()));
            }
        }
        let node = GraphVizNode::new(
            gt_id.id.clone(), 
            style
            );
        cluster.add_node(node);
    }
    cluster
}


pub fn bpmn_repr(
    bpmn : &Diagram
) -> GraphVizDiGraph {
    // Create a new graph:
    let mut digraph = GraphVizDiGraph::new(vec![]);
    // processes
    for (process_id,process) in &bpmn.top_level_processes {
        let cluster = bpmn_subproc_repr(bpmn,process_id,&process.content);
        digraph.add_cluster(cluster);
    }
    // edges
    for flow in bpmn.flows.values() {
        let edge = GraphVizEdge::new(
            flow.source_ref.id.clone(),
            None,
            flow.target_ref.id.clone(),
            None,
            vec![]
        );
        digraph.add_edge(edge);
    }
    digraph
}




