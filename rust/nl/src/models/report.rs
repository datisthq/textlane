use textlane_core::Position;

/// A message a natural-language check found, placed by `place_messages` once the check is done.
#[derive(Clone, Debug)]
pub(crate) struct Report<'a> {
    pub reason: String,
    pub rule: String,
    pub source: &'static str,
    pub position: Option<Position<'a>>,
}
