use bitflags::bitflags;

bitflags! {
    pub struct MutationType: u64 {
        const OTHER = 1 << 0;
        const REPLACE_WITH_TRUE = 1 << 1;
        const REPLACE_WITH_FALSE = 1 << 2;
        const ADD_ONE_TO_LITERAL = 1 << 3;
        const SUB_ONE_TO_LITERAL = 1 << 4;
        const REMOVE_RIGHT = 1 << 5;
        const NEGATE_LEFT = 1 << 6;
        const NEGATE_RIGHT = 1 << 7;
        const NEGATE_EXPRESSION = 1 << 8;
        const COMPARISON = 1 << 9;
        const OPORTUNISTIC_BINARY = 1 << 10;
        const OPORTUNISTIC_UNARY = 1 << 11;
        const ITERATOR_EMPTY = 1 << 12;
        const ITERATOR_SKIP_FIRST = 1 << 13;
        const ITERATOR_SKIP_LAST = 1 << 14;
        const ITERATOR_SKIP_BOUNDS = 1 << 15;
        const RETURN_DEFAULT = 1 << 16;
        const RETURN_ARGUMENT = 1 << 17;
        const EXCHANGE_ARGUMENT = 1 << 18;
        const CLONE_MUTABLE = 1 << 19;
    }
}

impl MutationType {
    pub fn as_str(&self) -> &'static str {
        match *self {
            MutationType::OTHER => "OTHER",
            MutationType::REPLACE_WITH_TRUE => "REPLACE_WITH_TRUE",
            MutationType::REPLACE_WITH_FALSE => "REPLACE_WITH_FALSE",
            MutationType::ADD_ONE_TO_LITERAL => "ADD_ONE_TO_LITERAL",
            MutationType::SUB_ONE_TO_LITERAL => "SUB_ONE_TO_LITERAL",
            MutationType::REMOVE_RIGHT => "REMOVE_RIGHT",
            MutationType::NEGATE_LEFT => "NEGATE_LEFT",
            MutationType::NEGATE_RIGHT => "NEGATE_RIGHT",
            MutationType::NEGATE_EXPRESSION => "NEGATE_EXPRESSION",
            MutationType::COMPARISON => "COMPARISION",
            MutationType::OPORTUNISTIC_BINARY => "OPORTUNISTIC_BINARY",
            MutationType::OPORTUNISTIC_UNARY => "OPORTUNISTIC_UNARY",
            MutationType::ITERATOR_EMPTY => "ITERATOR_EMPTY",
            MutationType::ITERATOR_SKIP_FIRST => "ITERATOR_SKIP_FIRST",
            MutationType::ITERATOR_SKIP_LAST => "ITERATOR_SKIP_LAST",
            MutationType::ITERATOR_SKIP_BOUNDS => "ITERATOR_SKIP_BOUNDS",
            MutationType::RETURN_DEFAULT=> "RETURN_DEFAULT",
            MutationType::RETURN_ARGUMENT => "RETURN_ARGUMENT",
            MutationType::EXCHANGE_ARGUMENT => "EXCHANGE_ARGUMENT",
            MutationType::CLONE_MUTABLE => "CLONE_MUTABLE",
            _ => "UNKNOWN",
        }
    }
}
