use crate::utils::tokenizer::driver::Tokenizer;

/// One state of a construct's state machine: it looks at `tokenizer.current` and says where
/// to go next.
pub(crate) type StateFn = for<'a, 'b> fn(&'b mut Tokenizer<'a>) -> State;

/// Where a state machine goes after one step.
#[derive(Clone, Copy, Debug)]
pub(crate) enum State {
    /// Consumed the current byte; feed the next one to this state.
    Next(StateFn),
    /// Did not consume; feed the same byte to this state.
    Retry(StateFn),
    Ok,
    Nok,
}

/// A pass over the events once a content type is tokenized.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Resolver {
    Attention,
    Content,
    Data,
    DefList,
    DirectiveContainer,
    GfmTable,
    GfmStrikethrough,
    HeadingAtx,
    HeadingSetext,
    Label,
    ListItem,
    String,
    Text,
}
