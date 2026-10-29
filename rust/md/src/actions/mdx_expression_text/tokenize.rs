use crate::actions::mdx_expression::tokenize::tokenize_mdx_expression;
use crate::models::construct::State;
use crate::models::mdx::EXPRESSION_TEXT;
use crate::utils::tokenizer::driver::Tokenizer;

/// An expression in text, `{a}`, which may span lazy lines.
pub(crate) fn tokenize_mdx_expression_text(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.parse_state.mdx.is_none() || tokenizer.current != Some(b'{') {
        return State::Nok;
    }
    let state = &mut tokenizer.tokenize_state.mdx;
    state.expression = &EXPRESSION_TEXT;
    state.spread = false;
    state.allow_empty = true;
    state.allow_lazy = true;
    State::Retry(tokenize_mdx_expression)
}
