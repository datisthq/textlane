use crate::actions::mdx_expression_flow::tokenize::tokenize_mdx_expression_flow;
use crate::actions::mdx_jsx::tokenize::tokenize_mdx_jsx;
use crate::actions::space_or_tab::tokenize::tokenize_space_or_tab;
use crate::models::construct::State;
use crate::models::mdx::JSX_FLOW;
use crate::utils::tokenizer::driver::Tokenizer;

/// JSX in flow: tags alone on their lines, with expressions between them and nothing else.
pub(crate) fn tokenize_mdx_jsx_flow(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.parse_state.mdx.is_none() {
        return State::Nok;
    }
    tokenizer.concrete = true;
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        let max = tokenizer.parse_state.indent_max();
        tokenizer.attempt(State::Next(before), State::Next(nok));
        return State::Retry(tokenize_space_or_tab(tokenizer, 0, max));
    }
    State::Retry(before)
}

fn before(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current != Some(b'<') {
        return State::Retry(nok);
    }
    start_tag(tokenizer)
}

fn start_tag(tokenizer: &mut Tokenizer) -> State {
    let state = &mut tokenizer.tokenize_state.mdx;
    state.jsx = &JSX_FLOW;
    state.allow_lazy = false;
    tokenizer.attempt(State::Next(after), State::Next(nok));
    State::Retry(tokenize_mdx_jsx)
}

fn after(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(end), State::Next(nok));
        return State::Retry(tokenize_space_or_tab(tokenizer, 1, usize::MAX));
    }
    State::Retry(end)
}

fn end(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.concrete = false;
            State::Ok
        }
        Some(b'<') => start_tag(tokenizer),
        Some(b'{') => {
            tokenizer.attempt(State::Next(end), State::Next(nok));
            State::Retry(tokenize_mdx_expression_flow)
        }
        _ => State::Retry(nok),
    }
}

fn nok(tokenizer: &mut Tokenizer) -> State {
    tokenizer.concrete = false;
    State::Nok
}
