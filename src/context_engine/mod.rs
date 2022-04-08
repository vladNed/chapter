pub mod definitions;
pub mod processor;

const DOCSTRING_START: &str = r#"\s{4}\"{3}\w*|\s{4}\"{3}"#;
const DEF_MATCH: &str = r"def\s\w+";
const CLASS_MATCH: &str = r"class\s\w+";
const ALL_MATCH: &str = r"__all__\s=\s(\[|\()";


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contex_node_add() {
        let mut new_node = definitions::ContextNode::Parent {
            name: "Test".to_owned(),
            context_type: definitions::ContextType::CLASS,
            start: 0,
            end: 12,
            is_public: true,
            value: String::new(),
            children: Vec::new()
        };
        let second_node = definitions::ContextNode::Child {
            name: "__init__".to_owned(),
            context_type: definitions::ContextType::METHOD,
            start: 7,
            end: 10,
            is_public: true,
            value: String::new(),
            children: Vec::new(),
            parent: Box::new(new_node.to_owned())
        };
        new_node.add_node(&second_node);
        assert_eq!(new_node.children_len(), 1);
    }

    #[test]
    fn test_context_node_defaults() {
        let mut default_node = definitions::ContextNode::Root;
        let new_node = definitions::ContextNode::Parent {
            name: "Test".to_owned(),
            context_type: definitions::ContextType::CLASS,
            start: 0,
            end: 12,
            is_public: true,
            value: String::new(),
            children: Vec::new()
        };
        default_node.add_node(&new_node);

        assert_eq!(default_node.children_len(), 0);
    }

    #[test]
    fn test_context_append_value() {
        let mut second_node = definitions::ContextNode::Parent {
            name: "__init__".to_owned(),
            context_type: definitions::ContextType::DOCSTRING,
            start: 7,
            end: 10,
            is_public: true,
            value: String::new(),
            children: Vec::new(),
        };
        let new_string = String::from("New value");
        second_node.append_value(&new_string);

        assert_eq!(second_node.value(), Some(&new_string));
    }
}