use tree_sitter::Parser;

pub struct ParserManager {
    pub go_parser: Parser,
    pub ts_parser: Parser,
    pub python_parser: Parser,
}

impl ParserManager {
    pub fn new() -> Self {
        Self {
            go_parser: Parser::new(),
            ts_parser: Parser::new(),
            python_parser: Parser::new(),
        }
    }

    pub fn init_parsers(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let go_lang: tree_sitter::Language = tree_sitter_go::LANGUAGE.into();
        let ts_lang: tree_sitter::Language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into();
        let py_lang: tree_sitter::Language = tree_sitter_python::LANGUAGE.into();

        self.go_parser.set_language(&go_lang)?;
        self.ts_parser.set_language(&ts_lang)?;
        self.python_parser.set_language(&py_lang)?;

        Ok(())
    }

    pub fn get_parser_for_language(&mut self, language: &str) -> Option<&mut Parser> {
        match language {
            "go" => Some(&mut self.go_parser),
            "typescript" | "javascript" => Some(&mut self.ts_parser),
            "python" => Some(&mut self.python_parser),
            _ => None,
        }
    }
}
