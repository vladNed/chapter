use regex::Regex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::definitions;
use super::{
    ALL_MATCH,
    CLASS_MATCH,
    DEF_MATCH,
    DOCSTRING_START,
    DOCSTRING_END
};

pub struct ContextProcessor {
    pub(super)context_type: definitions::ContextType,
    pub(super)current_context: Rc<RefCell<definitions::ContextNode>>,
    pub line_counter: usize,
    pub(super)max_height: usize,
    pub file_lines: Vec<String>,
    pub(super)patterns: HashMap<definitions::ContextType, Regex>,
}

impl ContextProcessor {
    pub fn load(file_lines: Vec<String>) -> Self {
        let mut patterns: HashMap<definitions::ContextType, Regex> = HashMap::new();
        patterns.insert(
            definitions::ContextType::METHOD,
            Regex::new(DEF_MATCH).unwrap(),
        );
        patterns.insert(
            definitions::ContextType::CLASS,
            Regex::new(CLASS_MATCH).unwrap(),
        );
        patterns.insert(
            definitions::ContextType::ALL,
            Regex::new(ALL_MATCH).unwrap(),
        );
        patterns.insert(
            definitions::ContextType::DOCSTRING,
            Regex::new(DOCSTRING_START).unwrap(),
        );

        Self {
            context_type: definitions::ContextType::ROOT,
            current_context: definitions::ContextNode::root(),
            line_counter: 0,
            max_height: file_lines.len(),
            file_lines,
            patterns,
        }
    }

    /// Checks for context entry point.
    ///
    /// If a line matches a python definition such as a class/method/all it
    /// returns the specific context.
    pub(super) fn check_context_entry(&self, current_line: &String) -> Option<definitions::ContextType> {
        for (context_type, pattern) in &self.patterns {
            if pattern.is_match(current_line) {
                return Some(context_type.to_owned())
            }
        }

        None
    }

    /// Changes the state of the processor so that it reflects being inside a
    /// context.
    pub(super) fn start_context(&mut self, context_type: definitions::ContextType, current_line: &String) {
        self.context_type = context_type;
        let context_name = self.get_context_name(current_line);
        let is_public = !context_name.starts_with("_");

        let new_context = match self.current_context.borrow().context_type {
            definitions::ContextType::ROOT => {
                definitions::ContextNode::new(
                    context_name,
                    self.context_type.clone(),
                    self.line_counter,
                    is_public
                )
            }
            _ => {
                let child_node = definitions::ContextNode::new(
                    context_name,
                    self.context_type.clone(),
                    self.line_counter,
                    is_public
                );

                child_node.borrow_mut().set_parent(Rc::clone(&self.current_context));
                child_node
            }
        };

        match new_context.borrow().parent {
            Some(_) => {
                self.current_context.borrow_mut().add_node(Rc::clone(&new_context));
            },
            None => ()
        }

        self.current_context = new_context;
    }

    /// Extracts context name based on context type
    pub(super) fn get_context_name(&self, current_line: &String) -> String {
        match self.context_type {
            definitions::ContextType::METHOD => {
                let c = self
                    .patterns
                    .get(&definitions::ContextType::METHOD)
                    .unwrap()
                    .captures(current_line)
                    .unwrap();
                c.get(0).unwrap().as_str().to_string()
            }
            definitions::ContextType::CLASS => {
                let pattern = self.patterns.get(&definitions::ContextType::CLASS).unwrap();
                let c = pattern
                    .captures(current_line)
                    .unwrap();
                c.get(0).unwrap().as_str().to_string()
            }
            _ => String::from("__empty__"),
        }
    }

    /// Check if the current line represents an exit point from the
    /// current context.
    pub(super) fn check_context_exit(&self, current_line: &String) -> bool {
        match self.context_type {
            definitions::ContextType::ROOT => false,
            definitions::ContextType::DOCSTRING => {
                let r = Regex::new(DOCSTRING_END).unwrap();
                r.is_match(current_line)
            },
            definitions::ContextType::CLASS => {
                if current_line.is_empty() {
                    if self.line_counter + 1 > self.max_height {
                        return true
                    }
                    let second_line = self.file_lines[self.line_counter + 1].to_owned();
                    if second_line.is_empty() || !second_line.starts_with("    "){
                        return true
                    }
                }
                false
            },
            _ => {
                if current_line.is_empty() {
                    if self.line_counter + 1 > self.max_height {
                        return true
                    }
                    let second_line = self.file_lines[self.line_counter + 1].to_owned();
                    if let Some(_) = self.check_context_entry(&second_line){
                        return true
                    } else if second_line.is_empty() {
                        return true
                    }
                }
                false
            }
        }
    }

    /// Changes the state of the processor so that it reflects being outside
    /// current context.
    fn exit_context(&mut self) -> () {
        match self.current_context.to_owned().borrow().parent.to_owned() {
            Some(p) => {
                self.current_context = Rc::clone(&p);
                self.context_type = p.borrow().context_type.to_owned();
            },
            None => {
                self.current_context = definitions::ContextNode::root();
                self.context_type = definitions::ContextType::ROOT
            }
        }
    }
    /// Extracting lines that are used inside unique context types
    ///
    /// For example, docstring context would need the string values.
    /// Each new context can be safely added as a match arm to this method.
    fn extract_context_data(&mut self, current_line: &String) {
        match self.context_type.to_owned() {
            definitions::ContextType::DOCSTRING => {
                self.current_context.borrow_mut().append_value(current_line)
            },
            _ => ()
        }
    }

    // TODO: More tests for this
    pub fn parse_module(&mut self) {
        loop {
            let current_line = self.file_lines[self.line_counter].to_owned();

            if let Some(c) = self.check_context_entry(&current_line) {
                self.start_context(c, &current_line);
                println!("ENTERING CONTEXT -> {:?}", self.context_type);
            }
            self.extract_context_data(&current_line);
            if self.check_context_exit(&current_line) {
                println!("Exiting CONTEXT -> {:?}", self.context_type);
                self.exit_context();
            }

            self.line_counter += 1;
            if self.line_counter >= self.max_height -1 {
                // TODO: Clear and make pymodule were to flush all the classes, methods, all...everything
                break
            }
        }
    }
}







