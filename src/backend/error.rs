use crate::frontend::ast::DataType;

pub enum SemanticError {
    MultipleDefinition {
        ident: String,
        begin: usize,
        end: usize,
    },
    UndefinedIdentifier {
        ident: String,
        begin: usize,
        end: usize,
    },
    TypeMismatch {
        expected_type: DataType,
        begin: usize,
        end: usize,
    },
}
