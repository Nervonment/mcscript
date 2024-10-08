use crate::frontend::ast::{exp::ArrayMethodType, DataType};

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
        found_type: DataType,
        begin: usize,
        end: usize,
    },
    ExpectedVoid {
        found_type: DataType,
        begin: usize,
        end: usize,
    },
    ExpectedValue {
        expected_type: DataType,
        begin: usize,
        end: usize,
    },
    IndexIntoNonArray {
        found_type: DataType,
        begin: usize,
        end: usize,
    },
    NoLoopToBreak {
        begin: usize,
        end: usize,
    },
    NoLoopToContinue {
        begin: usize,
        end: usize,
    },
    CallArrayMethodOnNonArray {
        method: ArrayMethodType,
        found_type: DataType,
        begin: usize,
        end: usize,
    },
    FuncArgumentsCountMismatch {
        expected_count: usize,
        found_count: usize,
        begin: usize,
        end: usize,
    },
}
