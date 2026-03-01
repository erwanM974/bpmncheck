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




use std::{collections::{BTreeMap, BTreeSet, HashSet}, rc::Rc};

use citreelo::ctl::{BinaryCTLOperator, CTLFormula, CTLFormulaLeaf, UnaryCTLOperator};
use petricheck::{model::label::PetriTransitionLabel, model_checking::props::BuiltinPetriAtomicProposition};

#[derive(Default)]
struct TrieNode {
    children: BTreeMap<String, TrieNode>,
    is_terminal: bool,
}

fn insert_trace(root: &mut TrieNode, trace: &[String]) {
    let mut node = root;
    for label in trace {
        node = node.children.entry(label.clone()).or_default();
    }
    node.is_terminal = true;
}

fn build_trie(traces: &HashSet<Vec<String>>) -> TrieNode {
    let mut root = TrieNode::default();
    for trace in traces {
        insert_trace(&mut root, trace);
    }
    root
}

fn complement(alphabet: &BTreeSet<String>, keep: &BTreeSet<String>) -> Option<CTLFormula<BuiltinPetriAtomicProposition>> {
    let excluded: Vec<_> = alphabet
        .difference(keep)
        .map(|x| 
            CTLFormula::Leaf(
                CTLFormulaLeaf::AtomicProp(
                    BuiltinPetriAtomicProposition::PreviousTransitionLabelMustBe(Rc::new(PetriTransitionLabel::new(x.to_string())))
                )
            )
        )
        .collect();
    if excluded.is_empty() {
        None
    } else {
        let ctl_or = excluded
        .into_iter()
        .reduce(|x, y| {
            CTLFormula::Binary(
                BinaryCTLOperator::Or,
                Box::new(x),
                Box::new(y),
            )
        }).unwrap();
        Some(CTLFormula::Unary(UnaryCTLOperator::Not, Box::new(ctl_or)))
    }
}

fn ctl_from_trie(
    previous : Option<String>,
    node: &TrieNode,
    alphabet: &BTreeSet<String>,
) -> CTLFormula<BuiltinPetriAtomicProposition> {
    // If no children, we’ve fully matched a trace
    if node.children.is_empty() {
        return CTLFormula::Leaf(CTLFormulaLeaf::True);
    }

    let mut branches = Vec::new();

    for (label, child) in &node.children {
        let mut keep = BTreeSet::new();
        keep.insert(label.clone());
        if let Some(prev_lab) = &previous {
            keep.insert(prev_lab.clone());
        }

        let wait = complement(alphabet, &keep);
        let next = ctl_from_trie(Some(label.clone()),child, alphabet);

        let reach = CTLFormula::Leaf(
            CTLFormulaLeaf::AtomicProp(
                BuiltinPetriAtomicProposition::PreviousTransitionLabelMustBe(Rc::new(PetriTransitionLabel::new(label.to_string())))
            )
        );
        let branch = if next == CTLFormula::Leaf(CTLFormulaLeaf::True) {
            match wait {
                None => {
                    CTLFormula::Unary(UnaryCTLOperator::EF, Box::new(reach))
                },
                Some(got_wait) => {
                    CTLFormula::Binary(BinaryCTLOperator::EU, Box::new(got_wait), Box::new(reach))
                }
            }
        } else {
            let reach_and_next = CTLFormula::Binary(
                BinaryCTLOperator::And, 
                Box::new(reach), 
                Box::new(next)
            );
            match wait {
                None => {
                    CTLFormula::Unary(UnaryCTLOperator::EF, Box::new(reach_and_next))
                },
                Some(got_wait) => {
                    CTLFormula::Binary(BinaryCTLOperator::EU, Box::new(got_wait), Box::new(reach_and_next))
                }
            }
        };

        branches.push(branch);
    }

    if branches.len() == 1 {
        branches[0].clone()
    } else {
        branches
        .into_iter()
        .reduce(|x, y| {
            CTLFormula::Binary(
                BinaryCTLOperator::Or,
                Box::new(x),
                Box::new(y),
            )
        }).unwrap()
    }
}

pub fn traces_to_ctl(
    traces: HashSet<Vec<String>>,
    alphabet: BTreeSet<String>,
) -> CTLFormula<BuiltinPetriAtomicProposition> {
    let trie = build_trie(&traces);
    ctl_from_trie(None,&trie, &alphabet)
}


