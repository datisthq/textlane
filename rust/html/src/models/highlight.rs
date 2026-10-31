use std::collections::BTreeMap;

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
    // Lowercase colors to what replaces them, `colorReplacements` resolved for this theme
    pub(crate) replacements: BTreeMap<String, String>,
}

// A token's colors and font style in one theme, as Shiki's `getTokenStyleObject` reads them
pub(crate) struct TokenStyle {
    pub(crate) color: Option<String>,
    pub(crate) background: Option<String>,
    pub(crate) italic: bool,
    pub(crate) bold: bool,
    pub(crate) underline: bool,
    pub(crate) strikethrough: bool,
}

// A line's tokens before whitespace merging, each with its style in every theme
pub(crate) type StyledLine = Vec<(String, Vec<TokenStyle>)>;

// One span of a highlighted line, its style empty for none
pub(crate) struct CodeToken {
    pub(crate) text: String,
    pub(crate) style: String,
    pub(crate) classes: Vec<String>,
}

// A line as transformers see it; a token or the line itself is `None` or `removed` once a
// notation took it out, so the indexes parsed comments hold stay valid
pub(crate) struct CodeLine {
    pub(crate) classes: Vec<String>,
    pub(crate) tokens: Vec<Option<CodeToken>>,
    pub(crate) removed: bool,
}

// A block between tokenizing and hast: Shiki's transformer hooks run over it
pub(crate) struct CodeBlock {
    pub(crate) lines: Vec<CodeLine>,
    pub(crate) language: String,
    pub(crate) meta: String,
    pub(crate) pre_classes: Vec<String>,
    pub(crate) code_classes: Vec<String>,
}

impl CodeBlock {
    // The indexes of the lines still in the block, as `code.children` holds them
    pub(crate) fn visible_lines(&self) -> Vec<usize> {
        (0..self.lines.len())
            .filter(|&index| !self.lines[index].removed)
            .collect()
    }
}

/// Add class names to a list as Shiki's `addClassToHast` does, each once.
pub(crate) fn add_classes(list: &mut Vec<String>, names: &[String]) {
    for name in names {
        if !name.is_empty() && !list.contains(name) {
            list.push(name.clone());
        }
    }
}

// What a block's own attributes ask for, beside its meta: `lineAttributes`,
// `wordAttribute` and `lineNumberAttribute`
#[derive(Default)]
pub(crate) struct BlockMarks {
    pub(crate) lines: Vec<LineMarks>,
    pub(crate) pre: Vec<String>,
    pub(crate) words: Vec<String>,
    pub(crate) numbers: Option<usize>,
}

// Inclusive line ranges, 1-based, and the classes those lines get
pub(crate) type LineMarks = (Vec<(usize, usize)>, Vec<String>);
