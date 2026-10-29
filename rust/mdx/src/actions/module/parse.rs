use oxc_allocator::Allocator;
use oxc_ast::ast::{ImportDeclarationSpecifier, Statement};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{GetSpan, SourceType};
use textlane_core::{ScriptError, ScriptStatement};

use crate::helpers::diagnostic::{check_nesting, read_diagnostic, with_stack};

/// Parse `source` as a JavaScript module with JSX and list its top-level statements by
/// ESTree type, with the names each import declares. Early errors acorn reports and oxc
/// leaves to semantic analysis (a name declared twice, an export of nothing) fail it too.
pub(crate) fn parse_module(source: &str) -> Result<Vec<ScriptStatement>, ScriptError> {
    let depth = check_nesting(source)?;
    let allocator = Allocator::default();
    let source_type = SourceType::mjs().with_jsx(true);
    with_stack(depth, || {
        let parsed = Parser::new(&allocator, source, source_type).parse();
        if parsed.fatal_error || !parsed.diagnostics.is_empty() {
            return Err(read_diagnostic(
                &parsed.diagnostics,
                parsed.fatal_error,
                0,
                source.len(),
            ));
        }
        let program = &parsed.program;
        let semantic = SemanticBuilder::new()
            .with_check_syntax_error(true)
            .build(program);
        if !semantic.diagnostics.is_empty() {
            return Err(read_diagnostic(
                &semantic.diagnostics,
                true,
                0,
                source.len(),
            ));
        }
        let mut statements: Vec<ScriptStatement> = program
            .directives
            .iter()
            .map(|directive| ScriptStatement {
                kind: "ExpressionStatement".into(),
                offset: directive.span.start as usize,
                imports: vec![],
            })
            .collect();
        for statement in &program.body {
            let imports = match statement {
                Statement::ImportDeclaration(import) => import
                    .specifiers
                    .iter()
                    .flatten()
                    .map(|specifier| match specifier {
                        ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                            specifier.local.name.to_string()
                        }
                        ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                            specifier.local.name.to_string()
                        }
                        ImportDeclarationSpecifier::ImportNamespaceSpecifier(
                            specifier,
                        ) => specifier.local.name.to_string(),
                    })
                    .collect(),
                _ => vec![],
            };
            statements.push(ScriptStatement {
                kind: read_type(statement).into(),
                offset: statement.span().start as usize,
                imports,
            });
        }
        Ok(statements)
    })
}

// The ESTree type of a statement, as acorn names it
fn read_type(statement: &Statement) -> &'static str {
    match statement {
        Statement::BlockStatement(_) => "BlockStatement",
        Statement::BreakStatement(_) => "BreakStatement",
        Statement::ContinueStatement(_) => "ContinueStatement",
        Statement::DebuggerStatement(_) => "DebuggerStatement",
        Statement::DoWhileStatement(_) => "DoWhileStatement",
        Statement::EmptyStatement(_) => "EmptyStatement",
        Statement::ExpressionStatement(_) => "ExpressionStatement",
        Statement::ForInStatement(_) => "ForInStatement",
        Statement::ForOfStatement(_) => "ForOfStatement",
        Statement::ForStatement(_) => "ForStatement",
        Statement::IfStatement(_) => "IfStatement",
        Statement::LabeledStatement(_) => "LabeledStatement",
        Statement::ReturnStatement(_) => "ReturnStatement",
        Statement::SwitchStatement(_) => "SwitchStatement",
        Statement::ThrowStatement(_) => "ThrowStatement",
        Statement::TryStatement(_) => "TryStatement",
        Statement::WhileStatement(_) => "WhileStatement",
        Statement::WithStatement(_) => "WithStatement",
        Statement::VariableDeclaration(_) => "VariableDeclaration",
        Statement::FunctionDeclaration(_) => "FunctionDeclaration",
        Statement::ClassDeclaration(_) => "ClassDeclaration",
        Statement::ImportDeclaration(_) => "ImportDeclaration",
        Statement::ExportAllDeclaration(_) => "ExportAllDeclaration",
        Statement::ExportDefaultDeclaration(_) => "ExportDefaultDeclaration",
        Statement::ExportDeclaration(_)
        | Statement::ExportNamedDeclaration(_)
        | Statement::ExportFromDeclaration(_) => "ExportNamedDeclaration",
        _ => "Statement",
    }
}

#[cfg(test)]
#[path = "parse_unit.rs"]
mod unit;
