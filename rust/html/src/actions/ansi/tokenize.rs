use textlane_core::Error;

use crate::actions::code::tokenize::replace_color;
use crate::models::highlight::{StyledLine, Themes, TokenStyle};
use crate::utils::ansi::palette::{AnsiPalette, dim_color};
use crate::utils::ansi::parse::{
    AnsiParser, BOLD, DIM, ITALIC, REVERSE, STRIKETHROUGH, UNDERLINE,
};

/// Split `ansi` code into lines of tokens colored by their escapes, as Shiki's
/// `tokenizeAnsiWithTheme` does for every theme.
pub(crate) fn tokenize_ansi(
    code: &str,
    themes: &Themes,
) -> Result<Vec<StyledLine>, Error> {
    let palettes: Vec<AnsiPalette> = (themes.list.iter())
        .map(|theme| AnsiPalette::new(theme.colors))
        .collect();
    let mut parser = AnsiParser::default();
    let mut lines = vec![];
    for line in code.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        let mut tokens = vec![];
        for segment in parser.parse(line) {
            let mut styles = Vec::with_capacity(themes.list.len());
            for (theme, palette) in themes.list.iter().zip(&palettes) {
                let value = |color| palette.value(&color);
                let (fg, bg) = (theme.colors.fg.to_owned(), theme.colors.bg.to_owned());
                let (color, background) = match segment.decorations & REVERSE {
                    0 => (segment.foreground.map_or(Some(fg), value), {
                        segment.background.and_then(value)
                    }),
                    _ => (
                        segment.background.map_or(Some(bg), value),
                        segment.foreground.map_or(Some(fg), value),
                    ),
                };
                let mut color = color.map(|color| replace_color(color, theme));
                let background = background.map(|color| replace_color(color, theme));
                if segment.decorations & DIM != 0 {
                    // Shiki fails reading a color the 256-color table lacks
                    let Some(dimmed) = color.as_deref().map(dim_color) else {
                        return Err(Error::new(
                            "highlight-failed",
                            "a dim ANSI color past the 256-color table has no color to dim",
                        ));
                    };
                    color = Some(dimmed);
                }
                let has = |flag: u8| segment.decorations & flag != 0;
                styles.push(TokenStyle {
                    color,
                    background,
                    italic: has(ITALIC),
                    bold: has(BOLD),
                    underline: has(UNDERLINE),
                    strikethrough: has(STRIKETHROUGH),
                });
            }
            tokens.push((segment.text, styles));
        }
        lines.push(tokens);
    }
    Ok(lines)
}
