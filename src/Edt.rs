// https://en.wikipedia.org/wiki/Evidential_decision_theory

use std::cell::{Cell};

// decision tree node
pub struct NodeStruct {
    pub children:Vec<Box<Edge>>,
}

pub struct Edge {
    pub target:Box<EnumNode>,
    pub prob:f64, // probability
    pub act:String, // action for this node
}

#[derive(Copy, Clone)]
pub struct LeafStruct {
    pub desirability:f64, // desirability of this outcome

    pub resProb:f64, // computed result probability
}

pub enum EnumNode {
    Node(NodeStruct),
    Leaf(Cell<LeafStruct>),
}

// we need a wrapper
//pub struct Node {
//    pub v:EnumNode,
//}

// recursivly calculate utility
pub fn calcUtility(n2:&mut EnumNode, p:f64) {
    match n2 {
        EnumNode::Node(n) => {
            for iEdge in &mut n.children {
                calcUtility(&mut *iEdge.target, p*iEdge.prob);
            }
        },
        EnumNode::Leaf(l) => {
            l.get_mut().resProb = p; // store probability
        }
    }
}

//#[derive(Copy, Clone)]
pub struct Sel {
    pub actPath:Vec<String>, // path of all actions
    pub score:f64, // computed score
}

// select option with the highest expected reward
// should get called after calcUtility() was called for the root
pub fn selBestOption(currentPath: &Vec<String>, current:&mut Box<Option<Sel>>, n2:&EnumNode) {
    match &n2 {
        EnumNode::Node(n) => {
            for iEdge in &n.children {
                let mut path = currentPath.to_vec();
                path.push(iEdge.act.clone());
                selBestOption(&path, current, &iEdge.target);
            }
        },
        EnumNode::Leaf(l) => {
            let thisScore = l.get().resProb * l.get().desirability;

            let isThisBetter = match &**current {
                Some(c2) => {
                    c2.score < thisScore
                }
                None => {
                    true
                }
            };

            if isThisBetter {
                *current = Box::new(Some(Sel{actPath:vec![], score:thisScore}));
            }
        }
    }
}
