use crate::models::construct::{Resolver, State};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::{LabelKind, LabelStart, Tokenizer};

/// `![`: the start of an image's label, closed later by a label end.
pub(crate) fn tokenize_label_start_image(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current != Some(b'!') {
        return State::Nok;
    }
    tokenizer.enter(Name::LABEL_IMAGE);
    tokenizer.enter(Name::LABEL_IMAGE_MARKER);
    tokenizer.consume();
    tokenizer.exit(Name::LABEL_IMAGE_MARKER);
    State::Next(open)
}

fn open(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current != Some(b'[') {
        return State::Nok;
    }
    tokenizer.enter(Name::LABEL_MARKER);
    tokenizer.consume();
    tokenizer.exit(Name::LABEL_MARKER);
    tokenizer.exit(Name::LABEL_IMAGE);
    let end = tokenizer.events.len() - 1;
    tokenizer.tokenize_state.label_starts.push(LabelStart {
        kind: LabelKind::Image,
        start: (end - 5, end),
        inactive: false,
    });
    tokenizer.register_resolver_before(Resolver::Label);
    State::Ok
}
