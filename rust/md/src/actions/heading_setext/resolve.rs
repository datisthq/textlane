use crate::helpers::skip::{skip_opt_back, skip_to, skip_to_back};
use crate::models::event::{ContentType, Event, Kind, Link};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

/// Turn each paragraph with an underline after it into a setext heading; an underline with
/// no paragraph above it is a paragraph itself.
pub(crate) fn resolve_heading_setext(tokenizer: &mut Tokenizer) {
    let underline = [Name::HEADING_SETEXT_UNDERLINE];
    let mut enter = skip_to(&tokenizer.events, 0, &underline);
    while enter < tokenizer.events.len() {
        let exit = skip_to(&tokenizer.events, enter + 1, &underline);
        let paragraph_exit = skip_opt_back(
            &tokenizer.events,
            enter - 1,
            &[
                Name::SPACE_OR_TAB,
                Name::LINE_ENDING,
                Name::BLOCK_QUOTE_PREFIX,
            ],
        );
        if tokenizer.events[paragraph_exit].name == Name::PARAGRAPH {
            let paragraph_enter =
                skip_to_back(&tokenizer.events, paragraph_exit - 1, &[Name::PARAGRAPH]);
            tokenizer.events[paragraph_enter].name = Name::HEADING_SETEXT_TEXT;
            tokenizer.events[paragraph_exit].name = Name::HEADING_SETEXT_TEXT;
            let mut heading_enter = tokenizer.events[paragraph_enter].clone();
            heading_enter.name = Name::HEADING_SETEXT;
            // Definitions before it in the same content: micromark starts the heading
            // where that content does
            let mut before = paragraph_enter;
            while before > 0 {
                let event = &tokenizer.events[before - 1];
                if event.name == Name::LINE_ENDING {
                    before -= 1;
                } else if event.name == Name::DEFINITION && event.kind == Kind::Exit {
                    before =
                        skip_to_back(&tokenizer.events, before - 2, &[Name::DEFINITION]);
                    heading_enter.point = tokenizer.events[before].point.clone();
                } else {
                    break;
                }
            }
            tokenizer.map.add(paragraph_enter, 0, vec![heading_enter]);
            let mut heading_exit = tokenizer.events[exit].clone();
            heading_exit.name = Name::HEADING_SETEXT;
            tokenizer.map.add(exit + 1, 0, vec![heading_exit]);
        } else if exit + 4 < tokenizer.events.len()
            && tokenizer.events[exit + 1].name == Name::LINE_ENDING
            && tokenizer.events[exit + 3].name == Name::PARAGRAPH
        {
            // The underline starts the paragraph after it: its text joins that paragraph's
            tokenizer.events[enter].name = Name::PARAGRAPH;
            tokenizer.events[exit + 1].name = Name::DATA;
            tokenizer.events[exit + 2].name = Name::DATA;
            tokenizer.events[exit + 1].point = tokenizer.events[enter].point.clone();
            tokenizer.events[exit + 1].link = Some(Link {
                previous: None,
                next: Some(exit + 4),
                content: ContentType::Text,
            });
            if let Some(link) = tokenizer.events[exit + 4].link.as_mut() {
                link.previous = Some(exit + 1);
            }
            tokenizer.map.add(enter + 1, exit - enter, vec![]);
            tokenizer.map.add(exit + 3, 1, vec![]);
        } else {
            tokenizer.events[enter].name = Name::PARAGRAPH;
            tokenizer.events[exit].name = Name::PARAGRAPH;
            let data = |kind, index: usize, link| Event {
                name: Name::DATA,
                kind,
                point: tokenizer.events[index].point.clone(),
                link,
            };
            let text = Link {
                previous: None,
                next: None,
                content: ContentType::Text,
            };
            let events = vec![
                data(Kind::Enter, enter, Some(text)),
                data(Kind::Exit, exit, None),
            ];
            tokenizer.map.add(enter + 1, exit - enter - 1, events);
        }
        enter = skip_to(&tokenizer.events, exit + 1, &underline);
    }
    tokenizer.map.consume(&mut tokenizer.events);
}
