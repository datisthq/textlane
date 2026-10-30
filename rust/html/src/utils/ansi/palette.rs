use crate::helpers::theme_table::ThemeColors;
use crate::utils::ansi::parse::AnsiColor;

// Shiki's `defaultAnsiColors`, for a theme without `terminal.ansi*` colors
const DEFAULT_COLORS: [&str; 16] = [
    "#000000", "#cd3131", "#0DBC79", "#E5E510", "#2472C8", "#BC3FBC", "#11A8CD",
    "#E5E5E5", "#666666", "#F14C4C", "#23D18B", "#F5F543", "#3B8EEA", "#D670D6",
    "#29B8DB", "#FFFFFF",
];
const LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];

/// ansi-sequence-parser's `createColorPalette` over a theme's terminal colors.
pub(crate) struct AnsiPalette {
    named: [&'static str; 16],
}

impl AnsiPalette {
    pub(crate) fn new(theme: &ThemeColors) -> Self {
        let mut named = DEFAULT_COLORS;
        for (slot, color) in named.iter_mut().zip(theme.terminal) {
            if let Some(color) = color {
                *slot = color;
            }
        }
        Self { named }
    }

    /// The CSS color an ANSI color stands for, `None` past the 256-color table.
    pub(crate) fn value(&self, color: &AnsiColor) -> Option<String> {
        match *color {
            AnsiColor::Named(index) => {
                self.named.get(index).map(|color| (*color).to_owned())
            }
            AnsiColor::Rgb(rgb) => Some(format_rgb(rgb)),
            AnsiColor::Table(index) => {
                let index = usize::from(index?);
                if let Some(color) = self.named.get(index) {
                    return Some((*color).to_owned());
                }
                Some(match index - 16 {
                    cube @ ..216 => format_rgb([
                        LEVELS[cube / 36],
                        LEVELS[cube / 6 % 6],
                        LEVELS[cube % 6],
                    ]),
                    gray => {
                        let level =
                            u8::try_from(8 + 10 * (gray - 216)).unwrap_or(u8::MAX);
                        format_rgb([level; 3])
                    }
                })
            }
        }
    }
}

fn format_rgb(rgb: [u8; 3]) -> String {
    format!("#{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2])
}

/// Shiki's `dimColor`: half the alpha of the first hex color, or a `-dim` ANSI variable.
pub(crate) fn dim_color(color: &str) -> String {
    let bytes = color.as_bytes();
    // `/#([0-9a-f]{3,8})/i`, the whole color replaced by what it finds
    let found = (0..bytes.len())
        .filter(|&index| bytes[index] == b'#')
        .find_map(|index| {
            let digits = (bytes[index + 1..].iter())
                .take(8)
                .take_while(|byte| byte.is_ascii_hexdigit())
                .count();
            (digits >= 3).then(|| &color[index + 1..index + 1 + digits])
        });
    let half = |alpha: &str| {
        let alpha = u16::from_str_radix(alpha, 16).unwrap_or(0);
        format!("{:02x}", alpha.div_ceil(2))
    };
    if let Some(hex) = found {
        let double = |index: usize| hex[index..=index].repeat(2);
        match hex.len() {
            8 => return format!("#{}{}", &hex[..6], half(&hex[6..])),
            6 => return format!("#{hex}80"),
            4 => {
                let alpha = half(&double(3));
                return format!("#{}{}{}{alpha}", double(0), double(1), double(2));
            }
            3 => return format!("#{}{}{}80", double(0), double(1), double(2)),
            _ => {}
        }
    }
    match find_ansi_variable(color) {
        Some(name) => format!("var({name}-dim)"),
        None => color.to_owned(),
    }
}

// `/var\((--[\w-]+-ansi-[\w-]+)\)/`
fn find_ansi_variable(color: &str) -> Option<&str> {
    let is_word = |byte: &u8| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-');
    color.match_indices("var(").find_map(|(index, _)| {
        let start = index + 4;
        let length = color.as_bytes()[start..]
            .iter()
            .take_while(|b| is_word(b))
            .count();
        let name = &color[start..start + length];
        let closed = color.as_bytes().get(start + length) == Some(&b')');
        let inner = name.strip_prefix("--")?;
        let infix = (1..inner.len()).any(|at| {
            inner[at..].starts_with("-ansi-") && at + "-ansi-".len() < inner.len()
        });
        (closed && infix).then_some(name)
    })
}

#[cfg(test)]
#[path = "palette_unit.rs"]
mod unit;
