use regex::Regex;
use std::collections::HashMap;

use super::definitions;
use super::{ALL_MATCH, CLASS_MATCH, DEF_MATCH, DOCSTRING_START};

pub struct ContextProcessor {
    context_type: definitions::ContextType,
    current_context: definitions::ContextNode,
    line_counter: usize,
    max_height: usize,
    file_lines: Vec<String>,
    patterns: HashMap<definitions::ContextType, Regex>,
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
            current_context: definitions::ContextNode::Root,
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
    fn check_context_entry(&mut self, current_line: &String) -> Option<definitions::ContextType> {
        for (context_type, pattern) in &self.patterns {
            if pattern.is_match(current_line) {
                return Some(context_type.clone())
            }
        }

        None
    }

    /// Changes the state of the processor so that it reflects being inside a
    /// context.
    fn start_context(&mut self, context_type: definitions::ContextType, current_line: &String) {
        match self.current_context {
            definitions::ContextNode::Root => {
                let context_name = self.get_context_name(current_line);
                let is_public = context_name.starts_with("_");
                let root_parent = definitions::ContextNode::Parent {
                    name: context_name,
                    context_type: context_type.to_owned(),
                    start: self.line_counter,
                    end: 0,
                    is_public,
                    value: String::new(),
                    children: Vec::new(),
                };
                self.current_context = root_parent;
                self.context_type = context_type;
            }
            _ => {
                let context_name = self.get_context_name(current_line);
                let is_public = context_name.starts_with("_");
                let new_child = definitions::ContextNode::Child {
                    name: context_name,
                    context_type: context_type.to_owned(),
                    start: self.line_counter,
                    end: 0,
                    is_public,
                    value: String::new(),
                    parent: Box::new(self.current_context.clone()),
                    children: Vec::new(),
                };
                self.current_context.add_node(&new_child);
                self.current_context = new_child;
                self.context_type = context_type;
            }
        }
    }

    /// Check if the current line represents an exit point from the
    /// current context.
    fn check_context_exit(&self, current_line: &String) -> bool {
        if self.context_type != definitions::ContextType::ROOT && current_line.is_empty(){
            if self.file_lines[self.line_counter + 1].is_empty() {
                return true
            }
        }
        false
    }

    /// Changes the state of the processor so that it reflects being outside
    /// current context.
    fn exit_context(&mut self) -> () {
        match self.current_context.to_owned() {
            definitions::ContextNode::Child {parent, ..} => {
                self.current_context.update(self.line_counter, String::new());
                self.current_context = *parent.clone();
                // TODO: Should flush somewhere to memory or smth
            }
            _ => {
                self.current_context.update(self.line_counter, String::new());
                self.current_context = definitions::ContextNode::Root;
                // TODO: Should flush somewhere to memory or smth
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
                self.current_context.append_value(current_line)
            },
            _ => ()
        }
    }

    fn get_context_name(&self, current_line: &String) -> String {
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
                let c = self
                    .patterns
                    .get(&definitions::ContextType::CLASS)
                    .unwrap()
                    .captures(current_line)
                    .unwrap();
                c.get(0).unwrap().as_str().to_string()
            }
            _ => String::from("__empty__"),
        }
    }

    pub fn parse_module(&mut self) {
        loop {
            let current_line = self.file_lines[self.line_counter].to_owned();

            if self.check_context_exit(&current_line) {
                self.exit_context();
            }
            if let Some(c) = self.check_context_entry(&current_line) {
                self.start_context(c, &current_line);
            }

            self.extract_context_data(&current_line);

            self.line_counter += 1;
            if self.line_counter >= self.max_height {
                break
            }
        }
    }
}
