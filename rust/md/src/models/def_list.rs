/// A definition description that opened, for the resolver that builds definition lists.
#[derive(Clone, Debug, Default)]
pub(crate) struct DefListStart {
    /// Where its terms start: lines of the paragraph before it at or after this byte.
    pub(crate) terms: Option<usize>,
    /// Whether it follows a description of the same list, so it has no terms.
    pub(crate) sibling: bool,
    /// Whether it closed a block quote whose lazy lines are its terms.
    pub(crate) block_quote: bool,
}

/// What the document tokenizer knows of its lines for definition lists, as micromark's
/// extension reads them from the document's events.
#[derive(Debug, Default)]
pub(crate) struct DefListTrack {
    /// The descriptions that opened, in order.
    pub(crate) starts: Vec<DefListStart>,
    /// The description being opened.
    pub(crate) pending: DefListStart,
    /// Whether the description being opened is spread.
    pub(crate) pending_spread: bool,
    /// Where the flow of the first line after the last container boundary starts: a term
    /// paragraph starts at or after it.
    pub(crate) run_start: usize,
    /// Where the flow of the last line that micromark's check closed the flow on starts:
    /// content does not reach above it.
    pub(crate) flow_start: usize,
    /// Where the flow of the last line with container events before it starts: terms do
    /// not reach above it.
    pub(crate) pure_start: usize,
    /// Where the flow of the trailing lazy lines starts, if the last line was lazy.
    pub(crate) lazy_start: Option<usize>,
    /// Whether the line before the last one was lazy.
    pub(crate) lazy_before: bool,
    /// Where the flow of this line starts.
    pub(crate) line_start: usize,
    /// Whether a container opened, closed, or took indent of a list item or footnote on
    /// this line.
    pub(crate) line_boundary: bool,
    /// Whether containers closed before this line.
    pub(crate) line_exits: bool,
    /// The number of events before the continuation being tried.
    pub(crate) events_before: usize,
}
