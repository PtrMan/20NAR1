#![allow(non_snake_case)]
#![allow(dead_code)]

use std::cell::{Cell};

mod Edt;

// experiment to compute action with expected utility maximization
pub fn expExpUtilityMax0() {
    // build structure

    let mut nodes:Vec<Box<Edt::EnumNode>> = vec![];

    let mut ops = vec!["^a".to_string(), "^b".to_string(), "^c".to_string()];

    // iterate over ops in this level
    for iOp in &ops {
        let aLeaf = Edt::LeafStruct {desirability:1.0, resProb:0.0};
        let y = vec![Box::new(Edt::EnumNode::Leaf(Cell::new(aLeaf)))];
    
        let mut node0 = Edt::NodeStruct{
            children:y,
            prob:1.0 / (ops.len() as f64),
            act:iOp.clone(),
        };
    
        nodes.push(Box::new(Edt::EnumNode::Node(node0)));
    }

    let mut root = Edt::EnumNode::Node(Edt::NodeStruct{
        children:nodes,
        prob:1.0,
        act:"^aa".to_string(),
    });

    // compute rating
    Edt::calcUtility(&mut root, 1.0);
    
    // select best rating
    let mut best:Box<Option<Edt::Sel>> = Box::new(None);
    let mut bestPath:Box<Vec<String>> = Box::new(vec![]);
    Edt::selBestOption(&mut bestPath, &mut best, &root);

    match *best {
        Some(sel) => {
            println!("sel w/ score = {}", sel.score);
        }
        None => {
            // not handled
        }
    }
}

pub fn main() {
    expExpUtilityMax0();
}