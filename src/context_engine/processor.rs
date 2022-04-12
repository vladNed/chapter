use regex::Regex;
use std::collections::HashMap;
use std::rc::Rc;

use super::controller::ContextState;
use super::definitions;
use super::{ALL_MATCH, CLASS_MATCH, DEF_MATCH, DOCSTRING_END, DOCSTRING_START};

pub struct ContextProcessor {
    pub context_state: ContextState,
    pub line_counter: usize,
    pub(super) max_height: usize,
    pub file_lines: Vec<String>,
    pub(super) patterns: HashMap<definitions::ContextType, Regex>,
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
            context_state: ContextState::new(),
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
    pub(super) fn check_context_entry(
        &self,
        current_line: &String,
    ) -> Option<definitions::ContextType> {
        for (context_type, pattern) in &self.patterns {
            if pattern.is_match(current_line) {
                // TODO: Big ugly fix for docstrings. Needs to change ASAP
                if self.context_state.context_type == definitions::ContextType::DOCSTRING
                    && *context_type == definitions::ContextType::DOCSTRING
                {
                    continue;
                }
                return Some(context_type.to_owned());
            }
        }

        None
    }

    /// Changes the state of the processor so that it reflects being inside a
    /// context.
    pub(super) fn start_context(
        &mut self,
        context_type: definitions::ContextType,
        current_line: &String,
    ) {
        match self.context_state.context_type {
            definitions::ContextType::ROOT => {
                self.context_state.context_type = context_type.clone();
                let context_name = self.get_context_name(current_line);
                let is_public = !context_name.starts_with("_");
                let parent_node = definitions::ContextNode::new(
                    context_name,
                    context_type.clone(),
                    self.line_counter,
                    is_public,
                );
                self.context_state.context_node = parent_node;
                self.context_state.context_type = context_type;
            }
            _ => {
                self.context_state.context_type = context_type.clone();
                let context_name = self.get_context_name(current_line);
                let is_public = !context_name.starts_with("_");
                let child_node = definitions::ContextNode::new(
                    context_name,
                    context_type.clone(),
                    self.line_counter,
                    is_public,
                );
                self.context_state.descend(child_node);
            }
        }
    }

    /// Extracts context name based on context type
    pub(super) fn get_context_name(&self, current_line: &String) -> String {
        match self.context_state.context_type {
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
                let c = pattern.captures(current_line).unwrap();
                c.get(0).unwrap().as_str().to_string()
            }
            _ => String::from("__empty__"),
        }
    }

    /// Check if the current line represents an exit point from the
    /// current context.
    pub(super) fn check_context_exit(&self, current_line: &String) -> bool {
        let line_is_empty = |line: &String| line.is_empty() || !line.starts_with("    ");

        match self.context_state.context_type {
            definitions::ContextType::ROOT => false,
            definitions::ContextType::DOCSTRING => {
                let r = Regex::new(DOCSTRING_END).unwrap();
                r.is_match(current_line)
            }
            definitions::ContextType::CLASS => {
                if current_line.is_empty() {
                    if self.line_counter + 1 > self.max_height {
                        return true;
                    }
                    if line_is_empty(&self.file_lines[self.line_counter + 1]) {
                        return true;
                    }
                }
                false
            }
            _ => {
                if current_line.is_empty() {
                    if self.line_counter + 1 > self.max_height {
                        return true;
                    }
                    let second_line = self.file_lines[self.line_counter + 1].to_owned();
                    if let Some(_) = self.check_context_entry(&second_line) {
                        return true;
                    } else if line_is_empty(&second_line) {
                        return true;
                    }
                }
                false
            }
        }
    }

    /// Changes the state of the processor so that it reflects being outside
    /// current context.
    fn exit_context(&mut self) -> () {
        self.context_state.ascend()
    }

    /// Extracting lines that are used inside unique context types
    ///
    /// For example, docstring context would need the string values.
    /// Each new context can be safely added as a match arm to this method.
    fn extract_context_data(&mut self, current_line: &String) {
        match self.context_state.context_type {
            definitions::ContextType::DOCSTRING => {
                self.context_state
                    .context_node
                    .borrow_mut()
                    .append_value(current_line);
            }
            _ => (),
        }
    }

    // TODO: More tests for this
    pub fn parse_module(&mut self) {
        let mut lines = self
            .file_lines
            .to_owned()
            .into_iter()
            .take(self.max_height - 1);

        loop {
            // Fetch current line
            let current_line = match lines.next() {
                Some(line) => line,
                None => {
                    // TODO: Flush objects...constantly
                    break;
                }
            };

            // Check context entry
            if let Some(c) = self.check_context_entry(&current_line) {
                self.start_context(c, &current_line);
            }

            // Process any kind of context for values
            self.extract_context_data(&current_line);

            // Check exit
            if self.check_context_exit(&current_line) {
                self.exit_context();
            }

            // Increment line counter
            self.line_counter += 1;
        }
    }
}
