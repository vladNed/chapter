pub type CodeLocation = (usize, usize);

/// NamespaceType refers to a code block context type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ContextType {
    /// Default namespace
    ROOT,

    /// Represents a python namespace context
    METHOD,

    /// Represents a python namespace context
    CLASS,

    /// Represents a __all__ namespace context
    ALL,

    /// Represents a docstring context
    DOCSTRING,
}

#[derive(Clone)]
pub enum ContextNode {
    Root,
    Parent {
        name: String,
        context_type: ContextType,
        start: usize,
        end: usize,
        is_public: bool,
        value: String,
        children: Vec<ContextNode>,
    },
    Child {
        name: String,
        context_type: ContextType,
        start: usize,
        end: usize,
        is_public: bool,
        value: String,
        parent: Box<ContextNode>,
        children: Vec<ContextNode>,
    },
}

impl ContextNode {
    pub fn value(&self) -> Option<&String> {
        match self {
            Self::Child { value, .. } => {
                Some(value)
            },
            Self::Parent { value, .. } => {
                Some(value)
            },
            _ => None,
        }
    }
    pub fn add_node(&mut self, node: &ContextNode) {
        match self {
            Self::Child { children, .. } => {
                children.push(node.to_owned());
            }
            Self::Parent { children, .. } => {
                children.push(node.to_owned());
            }
            _ => (),
        }
    }

    pub fn update(&mut self, update_end: usize, update_value: String) {
        match self {
            Self::Child { end, value, .. } => {
                *end = update_end;
                *value = update_value;
            }
            Self::Parent { end, value, .. } => {
                *end = update_end;
                *value = update_value;
            }
            _ => (),
        }
    }

    pub fn children_len(&self) -> usize {
        match self {
            Self::Child { children, .. } => {
                children.len()
            }
            Self::Parent { children, .. } => {
                children.len()
            }
            _ => 0,
        }
    }

    pub fn append_value(&mut self, new_value: &String) -> () {
        match self {
            Self::Child { value, .. } => {
                value.push_str(new_value)
            }
            Self::Parent { value, .. } => {
                value.push_str(new_value)
            }
            _ => (),
        }
    }
}
