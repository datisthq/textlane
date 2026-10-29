use crate::models::estree::Id;
use crate::models::name::{Key, Kind};
use crate::utils::printer::state::Printer;

/// astring's `NEEDS_PARENTHESES`: wrapped wherever it is an operand.
pub(crate) const NEEDS_PARENTHESES: u8 = 17;

/// astring's `EXPRESSIONS_PRECEDENCE`; JSX and patterns have none, as `undefined` there.
pub(crate) fn precedence_of(kind: Kind) -> Option<u8> {
    Some(match kind {
        Kind::ARRAY_EXPRESSION
        | Kind::TAGGED_TEMPLATE_EXPRESSION
        | Kind::THIS_EXPRESSION
        | Kind::IDENTIFIER
        | Kind::PRIVATE_IDENTIFIER
        | Kind::TEMPLATE_LITERAL
        | Kind::SUPER
        | Kind::SEQUENCE_EXPRESSION => 20,
        Kind::LITERAL => 18,
        Kind::MEMBER_EXPRESSION
        | Kind::CHAIN_EXPRESSION
        | Kind::CALL_EXPRESSION
        | Kind::NEW_EXPRESSION => 19,
        Kind::ARROW_FUNCTION_EXPRESSION
        | Kind::CLASS_EXPRESSION
        | Kind::FUNCTION_EXPRESSION
        | Kind::OBJECT_EXPRESSION => NEEDS_PARENTHESES,
        Kind::UPDATE_EXPRESSION => 16,
        Kind::UNARY_EXPRESSION | Kind::AWAIT_EXPRESSION => 15,
        Kind::BINARY_EXPRESSION => 14,
        Kind::LOGICAL_EXPRESSION => 13,
        Kind::CONDITIONAL_EXPRESSION => 4,
        Kind::ASSIGNMENT_EXPRESSION => 3,
        Kind::YIELD_EXPRESSION => 2,
        Kind::REST_ELEMENT => 1,
        _ => return None,
    })
}

fn operator_precedence(operator: &str) -> Option<u8> {
    Some(match operator {
        "||" => 2,
        "??" => 3,
        "&&" => 4,
        "|" => 5,
        "^" => 6,
        "&" => 7,
        "==" | "!=" | "===" | "!==" => 8,
        "<" | ">" | "<=" | ">=" | "in" | "instanceof" => 9,
        "<<" | ">>" | ">>>" => 10,
        "+" | "-" => 11,
        "*" | "%" | "/" => 12,
        "**" => 13,
        _ => return None,
    })
}

/// JS's `a < b` where `undefined` compares false either way.
pub(crate) fn below(left: Option<u8>, right: Option<u8>) -> bool {
    matches!((left, right), (Some(left), Some(right)) if left < right)
}

fn at_most(left: Option<u8>, right: Option<u8>) -> bool {
    matches!((left, right), (Some(left), Some(right)) if left <= right)
}

impl Printer<'_> {
    pub(crate) fn precedence(&self, id: Id) -> Option<u8> {
        precedence_of(self.kind(id))
    }

    fn operator(&self, id: Id) -> &str {
        self.store.str(id, Key::OPERATOR).unwrap_or_default()
    }

    /// `expressionNeedsParenthesis`.
    pub(crate) fn needs_parentheses(&self, node: Id, parent: Id, right: bool) -> bool {
        let precedence = self.precedence(node);
        if precedence == Some(NEEDS_PARENTHESES) {
            return true;
        }
        let parent_precedence = self.precedence(parent);
        if precedence != parent_precedence {
            return (!right
                && precedence == Some(15)
                && parent_precedence == Some(14)
                && self.operator(parent) == "**")
                || below(precedence, parent_precedence);
        }
        if precedence != Some(13) && precedence != Some(14) {
            return false;
        }
        let (operator, parent_operator) = (self.operator(node), self.operator(parent));
        if operator == "**" && parent_operator == "**" {
            return !right;
        }
        if precedence == Some(13)
            && parent_precedence == Some(13)
            && (operator == "??" || parent_operator == "??")
        {
            return true;
        }
        let (own, other) = (
            operator_precedence(operator),
            operator_precedence(parent_operator),
        );
        if right {
            at_most(own, other)
        } else {
            below(own, other)
        }
    }

    /// `formatExpression`: the operand, in parentheses when its parent needs them.
    pub(crate) fn operand(&mut self, node: Id, parent: Id, right: bool) {
        if self.needs_parentheses(node, parent, right) {
            self.write("(");
            self.node(node);
            self.write(")");
        } else {
            self.node(node);
        }
    }
}
