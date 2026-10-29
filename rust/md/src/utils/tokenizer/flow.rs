use crate::actions::blank_line::tokenize::tokenize_blank_line;
use crate::actions::code_fenced::tokenize::tokenize_code_fenced;
use crate::actions::code_indented::tokenize::tokenize_code_indented;
use crate::actions::directive_container::tokenize::tokenize_directive_container;
use crate::actions::directive_leaf::tokenize::tokenize_directive_leaf;
use crate::actions::frontmatter::tokenize::tokenize_frontmatter;
use crate::actions::gfm_table::tokenize::tokenize_gfm_table;
use crate::actions::heading_atx::tokenize::tokenize_heading_atx;
use crate::actions::heading_setext::tokenize::tokenize_heading_setext;
use crate::actions::html_flow::tokenize::tokenize_html_flow;
use crate::actions::math_flow::tokenize::tokenize_math_flow;
use crate::actions::mdx_expression_flow::tokenize::tokenize_mdx_expression_flow;
use crate::actions::mdx_jsx_flow::tokenize::tokenize_mdx_jsx_flow;
use crate::actions::mdxjs_esm::tokenize::tokenize_mdxjs_esm;
use crate::actions::thematic_break::tokenize::tokenize_thematic_break;
use crate::models::construct::State;
use crate::models::token::Name;
use crate::utils::tokenizer::content::tokenize_content_chunk;
use crate::utils::tokenizer::driver::Tokenizer;

/// The flow content type: one flow construct per line, falling back to content.
pub(crate) fn tokenize_flow(tokenizer: &mut Tokenizer) -> State {
    // Frontmatter is an extension: micromark tries it before the built-in constructs
    if !tokenizer.parse_state.frontmatter.is_empty() {
        tokenizer.attempt(State::Next(after), State::Next(before_flow));
        return State::Retry(tokenize_frontmatter);
    }
    State::Retry(before_flow)
}

fn before_flow(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'#') => {
            tokenizer.attempt(State::Next(after), State::Next(before_gfm_table));
            State::Retry(tokenize_heading_atx)
        }
        Some(b'`' | b'~') => {
            tokenizer.attempt(State::Next(after), State::Next(before_gfm_table));
            State::Retry(tokenize_code_fenced)
        }
        Some(b'<') if tokenizer.parse_state.mdx.is_some() => {
            tokenizer.attempt(State::Next(after), State::Next(before_gfm_table));
            State::Retry(tokenize_mdx_jsx_flow)
        }
        Some(b'<') => {
            tokenizer.attempt(State::Next(after), State::Next(before_gfm_table));
            State::Retry(tokenize_html_flow)
        }
        Some(b'{') if tokenizer.parse_state.mdx.is_some() => {
            tokenizer.attempt(State::Next(after), State::Next(before_gfm_table));
            State::Retry(tokenize_mdx_expression_flow)
        }
        Some(b'e' | b'i') if tokenizer.parse_state.mdx.is_some() => {
            tokenizer.attempt(State::Next(after), State::Next(before_gfm_table));
            State::Retry(tokenize_mdxjs_esm)
        }
        Some(b'*' | b'_') => {
            tokenizer.attempt(State::Next(after), State::Next(before_gfm_table));
            State::Retry(tokenize_thematic_break)
        }
        Some(b'$') if tokenizer.parse_state.math.is_some() => {
            tokenizer.attempt(State::Next(after), State::Next(before_gfm_table));
            State::Retry(tokenize_math_flow)
        }
        Some(b':') if tokenizer.parse_state.directive => {
            State::Retry(before_directive_container)
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
    tokenizer.attempt(State::Next(after), State::Next(before_math_flow));
    State::Retry(tokenize_code_fenced)
}

fn before_math_flow(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(after), State::Next(before_html_flow));
    State::Retry(tokenize_math_flow)
}

fn before_html_flow(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(after), State::Next(before_mdx_jsx_flow));
    State::Retry(tokenize_html_flow)
}

fn before_mdx_jsx_flow(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(after), State::Next(before_heading_atx));
    State::Retry(tokenize_mdx_jsx_flow)
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
    let next = if tokenizer.parse_state.directive {
        before_directive_container
    } else {
        before_mdx_expression_flow
    };
    tokenizer.attempt(State::Next(after), State::Next(next));
    State::Retry(tokenize_thematic_break)
}

fn before_directive_container(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(after), State::Next(before_directive_leaf));
    State::Retry(tokenize_directive_container)
}

fn before_directive_leaf(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(after), State::Next(before_mdx_expression_flow));
    State::Retry(tokenize_directive_leaf)
}

fn before_mdx_expression_flow(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(after), State::Next(before_gfm_table));
    State::Retry(tokenize_mdx_expression_flow)
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
