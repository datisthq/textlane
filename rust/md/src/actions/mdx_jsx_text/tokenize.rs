use crate::actions::mdx_jsx::tokenize::tokenize_mdx_jsx;
use crate::models::construct::State;
use crate::models::mdx::JSX_TEXT;
use crate::utils::tokenizer::driver::Tokenizer;

/// A JSX tag in text, `<a>`, which may span lazy lines.
pub(crate) fn tokenize_mdx_jsx_text(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.parse_state.mdx.is_none() || tokenizer.current != Some(b'<') {
        return State::Nok;
    }
    let state = &mut tokenizer.tokenize_state.mdx;
    state.jsx = &JSX_TEXT;
    state.allow_lazy = true;
    State::Retry(tokenize_mdx_jsx)
}
