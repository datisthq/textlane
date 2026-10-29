use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, ObjectPropertyKind, Statement};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use textlane_core::{ScriptError, ScriptExpression};

use crate::helpers::diagnostic::{check_nesting, read_diagnostic, with_stack};

/// Parse `source` as one JavaScript expression with JSX, as acorn's `parseExpressionAt`
/// does for micromark: where it ends, and the properties of an object literal.
pub(crate) fn parse_expression(source: &str) -> Result<ScriptExpression, ScriptError> {
    let depth = check_nesting(source)?;
    // In parentheses, a program of one statement is an expression, never a block; the
    // line ending keeps a trailing line comment from swallowing the `)`
    let wrapped = format!("({source}\n)");
    let allocator = Allocator::default();
    let source_type = SourceType::mjs().with_jsx(true);
    with_stack(depth, || {
        let parsed = Parser::new(&allocator, &wrapped, source_type).parse();
        if parsed.fatal_error || !parsed.diagnostics.is_empty() {
            return Err(read_diagnostic(
                &parsed.diagnostics,
                parsed.fatal_error,
                1,
                source.len(),
            ));
        }
        let program = &parsed.program;
        let expression = match (program.directives.is_empty(), program.body.as_slice()) {
            (true, [Statement::ExpressionStatement(statement)]) => &statement.expression,
            _ => return Err(whole_error(source)),
        };
        // The parentheses must be the wrapper's, around everything
        let Expression::ParenthesizedExpression(outer) = expression else {
            return Err(whole_error(source));
        };
        if outer.span.start != 0 || outer.span.end as usize != wrapped.len() {
            return Err(whole_error(source));
        }
        let inner = &outer.expression;
        let end = (inner.span().end as usize)
            .saturating_sub(1)
            .min(source.len());
        let mut unwrapped = inner;
        while let Expression::ParenthesizedExpression(parenthesized) = unwrapped {
            unwrapped = &parenthesized.expression;
        }
        let object = match unwrapped {
            Expression::ObjectExpression(object) => Some(
                object
                    .properties
                    .iter()
                    .map(|property| {
                        let kind = match property {
                            ObjectPropertyKind::ObjectProperty(_) => "Property",
                            ObjectPropertyKind::SpreadProperty(_) => "SpreadElement",
                        };
                        let offset = (property.span().start as usize).saturating_sub(1);
                        (kind.to_string(), offset)
                    })
                    .collect(),
            ),
            _ => None,
        };
        Ok(ScriptExpression { end, object })
    })
}

fn whole_error(source: &str) -> ScriptError {
    ScriptError {
        start: 0,
        end: source.len(),
        reason: "Unexpected content after expression".into(),
    }
}

#[cfg(test)]
#[path = "parse_unit.rs"]
mod unit;
