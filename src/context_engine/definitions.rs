pub type CodeLocation = (usize, usize);

/// NamespaceType refers to a code block context type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ContextType {
    /// Represents a default no namespace value
    NONE,

    /// Represents a python namespace context
    METHOD,

    /// Represents a python namespace context
    CLASS,

    /// Represents a __all__ namespace context
    ALL,
}


/// String like object but also contains additional attributes related to
/// code location and state
#[derive(Debug, Clone)]
pub struct Docstring {

    /// Value in string of the docstring lines concatenated
    pub value: String,

    /// Start line of the docstring
    pub start: usize,

    /// End line of the docstring
    pub end: usize,
}

impl Docstring {
    pub fn new(value: String, start: usize, end: usize) -> Self {
        Self {
            value,
            start,
            end
        }
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}
/// Represents a python structure such as:
/// - class
/// - method
///
/// To satisfy the cases where classes have method or nested classes
/// or methods have inner methods defined, this concept of a definition
/// holding multiple states resembles a tree.
pub enum Definition {

    /// Top level definition. Usually found in the root of the py module
    Origin {
        context_type: ContextType,
        name: String,
        location: CodeLocation,
        docstring: Docstring,
        is_public: bool,
        children: Box<Vec<Definition>>
    },

    /// Nested definition. Usually defined inside a class/method
    Nested {
        context_type: ContextType,
        name: String,
        location: CodeLocation,
        docstring: Docstring,
        is_public: bool,
        parent: Box<Vec<Definition>>,
        children: Box<Vec<Definition>>,
    },
}

/// Represents the `__all__` in python module defining which definitions
/// (classes/methods) are public.
pub struct AllDefinition {
    pub objects: Vec<Definition>
}

pub struct PyModule {

    /// Full path to the `.py` file
    pub filename: String,

    /// Docstring
    pub docstring: Docstring,

    /// Definitions
    pub defs: Vec<Definition>,

    /// `__all__` representation
    pub all: Option<AllDefinition>
}