use crate::actions::blank_line::tokenize::tokenize_blank_line;
use crate::actions::code_fenced::tokenize::tokenize_code_fenced;
use crate::actions::code_indented::tokenize::tokenize_code_indented;
use crate::actions::gfm_table::tokenize::tokenize_gfm_table;
use crate::actions::heading_atx::tokenize::tokenize_heading_atx;
use crate::actions::heading_setext::tokenize::tokenize_heading_setext;
use crate::actions::html_flow::tokenize::tokenize_html_flow;
use crate::actions::thematic_break::tokenize::tokenize_thematic_break;
use crate::models::construct::State;
use crate::models::token::Name;
use crate::utils::tokenizer::content::tokenize_content_chunk;
use crate::utils::tokenizer::driver::Tokenizer;

/// The flow content type: one flow construct per line, falling back to content.
pub(crate) fn tokenize_flow(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'#') => {
            tokenizer.attempt(State::Next(after), State::Next(before_gfm_table));
            State::Retry(tokenize_heading_atx)
        }
        Some(b'`' | b'~') => {
            tokenizer.attempt(State::Next(after), State::Next(before_gfm_table));
            State::Retry(tokenize_code_fenced)
        }
        Some(b'<') => {
            tokenizer.attempt(State::Next(after), State::Next(before_gfm_table));
            State::Retry(tokenize_html_flow)
        }
        Some(b'*' | b'_') => {
            tokenizer.attempt(State::Next(after), State::Next(before_gfm_table));
            State::Retry(tokenize_thematic_break)
        }
        _ => State::Retry(blank_line_before),
    }
}

fn blank_line_before(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        State::Next(blank_line_after),
        State::Next(before_code_indented),
    );
    State::Retry(tokenize_blank_line)
}

fn before_code_indented(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(after), State::Next(before_code_fenced));
    State::Retry(tokenize_code_indented)
}

fn before_code_fenced(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(after), State::Next(before_html_flow));
    State::Retry(tokenize_code_fenced)
}

fn before_html_flow(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(after), State::Next(before_heading_atx));
    State::Retry(tokenize_html_flow)
}

fn before_heading_atx(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(after), State::Next(before_heading_setext));
    State::Retry(tokenize_heading_atx)
}

fn before_heading_setext(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(after), State::Next(before_thematic_break));
    State::Retry(tokenize_heading_setext)
}

fn before_thematic_break(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(after), State::Next(before_gfm_table));
    State::Retry(tokenize_thematic_break)
}

fn before_gfm_table(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(after), State::Next(before_content));
    State::Retry(tokenize_gfm_table)
}

fn before_content(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(after), State::Nok);
    State::Retry(tokenize_content_chunk)
}

fn blank_line_after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\n') => {
            tokenizer.enter(Name::BLANK_LINE_ENDING);
            tokenizer.consume();
            tokenizer.exit(Name::BLANK_LINE_ENDING);
            // A blank line ends a paragraph, so the next line may not interrupt anything
            tokenizer.interrupt = false;
            State::Next(tokenize_flow)
        }
        _ => State::Ok,
    }
}

fn after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\n') => {
            tokenizer.enter(Name::LINE_ENDING);
            tokenizer.consume();
            tokenizer.exit(Name::LINE_ENDING);
            State::Next(tokenize_flow)
        }
        _ => State::Ok,
    }
}
