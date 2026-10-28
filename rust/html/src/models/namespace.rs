/// A namespace an element or attribute is in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Namespace {
    Html,
    MathMl,
    Svg,
    Xlink,
    Xml,
    Xmlns,
}
