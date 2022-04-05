use std::collections::HashMap;

use regex::Regex;

use super::definitions;
use super::{ALL_MATCH, CLASS_MATCH, DEF_MATCH, INDENT_BLOCK};

pub struct ContextProcessor {
    in_context: bool,
    context_name: String,
    context_type: definitions::ContextType,
    line_counter: usize,
    current_indent: String,
    context_start_line: usize,
    max_height: usize,
    file_lines: Vec<String>,
    filename: String,
    patterns: HashMap<definitions::ContextType, Regex>,
}

impl ContextProcessor {
    pub fn load(file_lines: Vec<String>, filename: String) -> Self {
        let mut patterns: HashMap<definitions::ContextType, Regex> = HashMap::new();
        patterns.insert(definitions::ContextType::METHOD, Regex::new(DEF_MATCH).unwrap());
        patterns.insert(definitions::ContextType::CLASS, Regex::new(CLASS_MATCH).unwrap());
        patterns.insert(definitions::ContextType::ALL, Regex::new(ALL_MATCH).unwrap());

        Self {
            in_context: false,
            context_name: String::new(),
            context_type: definitions::ContextType::NONE,
            line_counter: 0,
            current_indent: INDENT_BLOCK.to_owned(),
            context_start_line: 0,
            max_height: file_lines.len(),
            file_lines,
            filename,
            patterns,
        }
    }

    /// Checks for context entry point.
    ///
    /// If a line matches a python definition such as a class/method/all it
    /// returns the specific context.
    pub fn check_context_entry(&mut self, current_line: &String) -> Option<definitions::ContextType> {
        if current_line.starts_with(INDENT_BLOCK) {
            return None;
        }
        if self.in_context {
            todo!("Flush context and reset first");
        }

        for (context_type, pattern) in &self.patterns {
            if pattern.is_match(current_line) {
                return Some(context_type.clone());
            }
        }

        return None;
    }

    /// Check if the current line represents an exit point from the
    /// current context.
    pub fn check_context_exit(&self, current_line: &String) -> bool {
        if self.in_context && current_line.is_empty() {
            if self.file_lines[self.line_counter + 1].is_empty() {
                return true;
            }
        }
        false
    }

    /// Changes the state of the processor so that it reflects being inside a
    /// context.
    pub fn start_context(
        &mut self,
        context_type: definitions::ContextType,
        current_line: &String,
    ) -> () {
        self.in_context = true;
        self.context_type = context_type;
        self.context_start_line = self.line_counter;
        self.context_name = self.get_context_name(current_line);
    }

    /// Changes the state of the processor so that it reflects being outside
    /// current context.
    pub fn exit_context(&mut self) -> () {
        self.in_context = false;
        self.context_type = definitions::ContextType::NONE;
        self.current_indent = INDENT_BLOCK.to_owned();
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

    pub fn flush(&self) -> definitions::Definition {
        definitions::Definition::Origin {
            context_type: self.context_type.clone(),
            name: self.context_name.clone(),
            location: (self.context_start_line, self.line_counter),
            docstring: definitions::Docstring::new(String::new(), 0, 0),
            is_public: self.context_name.starts_with("_"),
            children: Box::new(Vec::new())
        }
    }


    pub fn parse_module(&mut self) -> Vec<definitions::Definition> {
        let mut structures: Vec<definitions::Definition> = Vec::new();
        loop {

            // Grab the current line
            let current_line = self.file_lines[self.line_counter].to_owned();

            // Check for entry or exit of context
            if self.check_context_exit(&current_line) {
                structures.push(self.flush());
                self.exit_context();
            }
            if let Some(context) = self.check_context_entry(&current_line) {
                self.start_context(context, &current_line);
            }

            // Check if end of file
            self.line_counter += 1;
            if self.line_counter >= self.max_height {
                if self.in_context {
                    structures.push(self.flush());
                }
                break
            }
        }

        structures
    }

}
