#![crate_name = "xpath"]

extern crate document;

use document::{Any,ToAny};
use document::{Document,Nodeset};
use std::collections::HashMap;

#[deriving(PartialEq,Show,Clone)]
pub enum XPathValue<'n> {
    Boolean(bool),
    Number(f64),
    String(String),
    Nodes(Nodeset), // rename as Nodeset
}

impl<'n> XPathValue<'n> {
    fn boolean(&self) -> bool {
        match *self {
            Boolean(val) => val,
            Number(n) => n != 0.0 && ! n.is_nan(),
            String(ref s) => ! s.is_empty(),
            Nodes(ref nodeset) => nodeset.size() > 0,
        }
    }

    fn number(&self) -> f64 {
        match *self {
            Number(val) => val,
            _ => -42.0
        }
    }

    fn string(&self) -> String {
        match *self {
            String(ref val) => val.clone(),
            _ => "Unimplemented".to_string(),
        }
    }

    fn nodeset(&self) -> Nodeset {
        match *self {
            Nodes(ref ns) => ns.clone(),
            _ => fail!("Did not evaluate to a nodeset!"),
        }
    }
}

pub trait XPathFunction<'n> {
    fn evaluate(&self,
                context: &XPathEvaluationContext,
                args: Vec<XPathValue>) -> XPathValue<'n>;
}

pub struct XPathEvaluationContext<'a> {
    document: & 'a Document,
    node: Any,
    functions: & 'a HashMap<String, Box<XPathFunction<'a>>>,
    position: uint,
}

impl<'a> XPathEvaluationContext<'a> {
    pub fn new<A: ToAny>(document: & 'a Document,
                        node: A,
                        functions: & 'a HashMap<String, Box<XPathFunction<'a>>>) -> XPathEvaluationContext<'a>
    {
        XPathEvaluationContext {
            document: document,
            node: node.to_any(),
            functions: functions,
            position: 0,
        }
    }

    fn node(&self) -> Any {
        self.node
    }

    fn new_context_for(& self, size: uint) -> XPathEvaluationContext<'a> {
        XPathEvaluationContext {
            document: self.document,
            node: self.node,
            functions: self.functions,
            position: 0,
        }
    }

    fn next<A: ToAny>(& mut self, node: A) {
        self.position += 1;
    }

    fn position(&self) -> uint {
        self.position
    }

    fn function_for_name(&self, name: &str) -> Option<& 'a Box<XPathFunction<'a>>> {
        self.functions.find(&name.to_string())
    }
}


pub struct XPathNodeTest;

impl XPathNodeTest {
    fn test(&self, context: &XPathEvaluationContext, result: &mut Nodeset) {
    }
}

struct EmptyIterator<T>;

impl<T> Iterator<T> for EmptyIterator<T> {
    fn next(&mut self) -> Option<T> { None }
}

pub mod token;
pub mod tokenizer;
pub mod deabbreviator;
pub mod disambiguator;
pub mod axis;
pub mod expression;
