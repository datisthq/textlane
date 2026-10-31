/// What `visit_elements` does after its visitor sees an element.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visit {
    /// Go on into the element's children.
    Continue,
    /// Leave the element's children unvisited.
    Skip,
    /// Visit nothing more.
    Stop,
}
