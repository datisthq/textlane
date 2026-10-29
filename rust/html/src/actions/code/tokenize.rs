use giallo::{FontStyle, HighlightOptions, Style, ThemeVariant};
use textlane_core::Error;

use crate::helpers::registry::load_registry;
use crate::models::highlight::{CodeToken, Themes};

/// Split code into lines of styled tokens as Shiki's `codeToTokens` and its whitespace
/// merging do, for a language giallo or Shiki's plain text knows.
pub(crate) fn tokenize_code(
    code: &str,
    language: &str,
    themes: &Themes,
) -> Result<Vec<Vec<CodeToken>>, Error> {
    if is_plain_language(language) {
        let lines = code
            .split('\n')
            .map(|line| line.strip_suffix('\r').unwrap_or(line));
        return Ok(lines
            .map(|text| {
                vec![CodeToken {
                    text: text.to_owned(),
                    style: String::new(),
                }]
            })
            .collect());
    }
    let mut lines = highlight_lines(code, language, themes)?;
    // giallo gives empty code no line, where Shiki gives it one empty line
    if lines.is_empty() {
        lines.push(vec![]);
    }
    Ok(lines
        .into_iter()
        .map(|line| merge_whitespace(merge_styles(line), themes))
        .collect())
}

/// Whether Shiki takes a language as plain text, which it leaves uncolored.
pub(crate) fn is_plain_language(language: &str) -> bool {
    // Shiki colors `ansi` from its escapes, which giallo cannot
    matches!(language, "plaintext" | "txt" | "text" | "plain" | "ansi")
}

type Line = Vec<(String, Vec<Style>)>;

// Each token as giallo's tokenizer cut it, with its style in every theme
fn highlight_lines(
    code: &str,
    language: &str,
    themes: &Themes,
) -> Result<Vec<Line>, Error> {
    let registry = load_registry().ok_or_else(|| {
        Error::new("highlight-unavailable", "giallo's registry failed to load")
    })?;
    let failed = |error: giallo::Error| Error::new("highlight-failed", error.to_string());
    let options = |theme| {
        HighlightOptions::new(language, theme)
            .merge_whitespace(false)
            .merge_same_style_tokens(false)
    };
    if let [light, dark] = &themes.list[..] {
        let theme = ThemeVariant::Dual {
            light: light.colors.name,
            dark: dark.colors.name,
        };
        let highlighted = registry.highlight(code, &options(theme)).map_err(failed)?;
        return Ok(highlighted
            .tokens
            .into_iter()
            .map(|line| {
                line.into_iter()
                    .map(|token| match token.style {
                        ThemeVariant::Dual { light, dark } => {
                            (token.text, vec![light, dark])
                        }
                        ThemeVariant::Single(style) => (token.text, vec![style]),
                    })
                    .collect()
            })
            .collect());
    }
    let mut lines: Vec<Line> = vec![];
    for (index, theme) in themes.list.iter().enumerate() {
        let options = options(ThemeVariant::Single(theme.colors.name));
        let highlighted = registry.highlight(code, &options).map_err(failed)?;
        for (number, line) in highlighted.tokens.into_iter().enumerate() {
            if index == 0 {
                lines.push(
                    line.iter()
                        .map(|token| (token.text.clone(), vec![]))
                        .collect(),
                );
            }
            let Some(target) = lines.get_mut(number) else {
                continue;
            };
            // One tokenizer cuts the same tokens whatever the theme
            for (slot, token) in target.iter_mut().zip(line) {
                if let ThemeVariant::Single(style) = token.style {
                    slot.1.push(style);
                }
            }
        }
    }
    Ok(lines)
}

// vscode-textmate joins neighbours one theme styles alike, and Shiki splits each theme's
// tokens where another's end, so tokens stay apart where any theme tells them apart
fn merge_styles(line: Line) -> Line {
    let mut merged: Line = Vec::with_capacity(line.len());
    for (text, styles) in line {
        match merged.last_mut() {
            Some(last) if last.1 == styles => last.0.push_str(&text),
            _ => merged.push((text, styles)),
        }
    }
    merged
}

// Shiki's `mergeWhitespaceTokens`: whitespace joins the token after it, unless that one
// is underlined or struck, which only a single theme's tokens still know
fn merge_whitespace(line: Line, themes: &Themes) -> Vec<CodeToken> {
    let mut tokens = Vec::with_capacity(line.len());
    let mut carry = String::new();
    let count = line.len();
    for (index, (text, styles)) in line.into_iter().enumerate() {
        let could_merge = !themes.single || !styles.iter().any(is_decorated);
        let style = stringify_style(&styles, themes);
        if could_merge && is_whitespace(&text) && index + 1 < count {
            carry.push_str(&text);
        } else if !carry.is_empty() {
            if could_merge {
                carry.push_str(&text);
                tokens.push(CodeToken {
                    text: std::mem::take(&mut carry),
                    style,
                });
            } else {
                tokens.push(CodeToken {
                    text: std::mem::take(&mut carry),
                    style: String::new(),
                });
                tokens.push(CodeToken { text, style });
            }
        } else {
            tokens.push(CodeToken { text, style });
        }
    }
    tokens
}

fn is_decorated(style: &Style) -> bool {
    style.font_style.contains(FontStyle::UNDERLINE)
        || style.font_style.contains(FontStyle::STRIKETHROUGH)
}

// JavaScript's `/^\s+$/`
fn is_whitespace(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|c| (c.is_whitespace() && c != '\u{85}') || c == '\u{FEFF}')
}

// Shiki's `getTokenStyleObject` per theme, then `flatTokenVariants`: the default theme's
// color is plain, everything else a `--shiki-<mode>…` variable, `inherit` where unset
fn stringify_style(styles: &[Style], themes: &Themes) -> String {
    let objects: Vec<Vec<(&str, String)>> = (styles.iter().zip(&themes.list))
        .map(|(style, theme)| {
            let color = style.foreground.as_hex();
            let spelled = (theme.colors.spellings.iter())
                .find(|(written, _)| *written == color)
                .map_or(color.clone(), |(_, spelled)| (*spelled).to_owned());
            let mut object = vec![("color", spelled)];
            if style.font_style.contains(FontStyle::ITALIC) {
                object.push(("font-style", "italic".to_owned()));
            }
            if style.font_style.contains(FontStyle::BOLD) {
                object.push(("font-weight", "bold".to_owned()));
            }
            let decorations: Vec<&str> = [
                (FontStyle::UNDERLINE, "underline"),
                (FontStyle::STRIKETHROUGH, "line-through"),
            ]
            .into_iter()
            .filter(|(flag, _)| style.font_style.contains(*flag))
            .map(|(_, name)| name)
            .collect();
            if !decorations.is_empty() {
                object.push(("text-decoration", decorations.join(" ")));
            }
            object
        })
        .collect();
    if themes.single {
        return (objects.iter().flatten())
            .map(|(key, value)| format!("{key}:{value}"))
            .collect::<Vec<_>>()
            .join(";");
    }
    let mut keys: Vec<&str> = vec![];
    for (key, _) in objects.iter().flatten() {
        if !keys.contains(key) {
            keys.push(key);
        }
    }
    let mut declarations = vec![];
    for (index, (object, theme)) in objects.iter().zip(&themes.list).enumerate() {
        for key in &keys {
            let value = (object.iter())
                .find(|(name, _)| name == key)
                .map_or("inherit", |(_, value)| value.as_str());
            if index == 0 && matches!(*key, "color" | "background-color") {
                declarations.push(format!("{key}:{value}"));
                continue;
            }
            let suffix = match *key {
                "color" => String::new(),
                "background-color" => "-bg".to_owned(),
                key => format!("-{key}"),
            };
            declarations.push(format!("--shiki-{}{suffix}:{value}", theme.mode));
        }
    }
    declarations.join(";")
}
