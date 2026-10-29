use crate::helpers::theme_table::ThemeColors;

// The themes a block is colored with, the default color mode first
pub(crate) struct Themes {
    pub(crate) list: Vec<Theme>,
    // `theme` rather than `themes`: plain colors, no CSS variables
    pub(crate) single: bool,
}

pub(crate) struct Theme {
    // The color mode it serves, such as `light`, which names its CSS variables
    pub(crate) mode: String,
    pub(crate) colors: &'static ThemeColors,
}

// One span of a highlighted line, its style empty for none
pub(crate) struct CodeToken {
    pub(crate) text: String,
    pub(crate) style: String,
}
