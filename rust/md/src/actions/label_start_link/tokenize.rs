use crate::models::construct::{Resolver, State};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::{LabelKind, LabelStart, Tokenizer};

/// `[`: the start of a link's label, closed later by a label end.
pub(crate) fn tokenize_label_start_link(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current != Some(b'[') {
        return State::Nok;
    }
    let start = tokenizer.events.len();
    tokenizer.enter(Name::LABEL_LINK);
    tokenizer.enter(Name::LABEL_MARKER);
    tokenizer.consume();
    tokenizer.exit(Name::LABEL_MARKER);
    tokenizer.exit(Name::LABEL_LINK);
    tokenizer.tokenize_state.label_starts.push(LabelStart {
        kind: LabelKind::Link,
        start: (start, tokenizer.events.len() - 1),
        inactive: false,
    });
    tokenizer.register_resolver_before(Resolver::Label);
    State::Ok
}
