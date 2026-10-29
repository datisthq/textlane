use textlane_core::{ScriptError, ScriptExpression, ScriptParser, ScriptStatement};

use crate::actions::expression::parse::parse_expression;
use crate::actions::module::parse::parse_module;

/// The script parser `md.mdx` checks JavaScript with, on oxc: the textlane processor seeds
/// it into `ScriptParserSlot`.
#[derive(Clone, Copy, Debug, Default)]
pub struct OxcScriptParser;

impl ScriptParser for OxcScriptParser {
    fn parse_expression(&self, source: &str) -> Result<ScriptExpression, ScriptError> {
        parse_expression(source)
    }

    fn parse_module(&self, source: &str) -> Result<Vec<ScriptStatement>, ScriptError> {
        parse_module(source)
    }
}
