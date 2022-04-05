pub mod definitions;
pub mod processor;

const INDENT_BLOCK: &str = "    ";
const DEF_MATCH: &str = r"def\s\w+";
const CLASS_MATCH: &str = r"class\s\w+";
const ALL_MATCH: &str = r"__all__\s=\s(\[|\()";