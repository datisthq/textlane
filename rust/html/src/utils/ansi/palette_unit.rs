use super::{AnsiPalette, dim_color};
use crate::helpers::theme_table::THEMES;
use crate::utils::ansi::parse::AnsiColor;

fn palette(name: &str) -> Option<AnsiPalette> {
    THEMES
        .iter()
        .find(|theme| theme.name == name)
        .map(AnsiPalette::new)
}

#[test]
fn takes_named_colors_from_the_theme_or_vs_code() {
    let colors = |name| {
        palette(name).map(|palette| {
            [AnsiColor::Named(1), AnsiColor::Table(Some(15))]
                .map(|color| palette.value(&color))
        })
    };
    assert_eq!(
        colors("catppuccin-latte"),
        Some([Some("#d20f39".to_owned()), Some("#bcc0cc".to_owned())])
    );
    assert_eq!(
        colors("dark-plus"),
        Some([Some("#cd3131".to_owned()), Some("#FFFFFF".to_owned())])
    );
}

#[test]
fn builds_the_256_color_table() {
    let Some(palette) = palette("dark-plus") else {
        panic!("dark-plus is bundled");
    };
    let table = |index| palette.value(&AnsiColor::Table(Some(index)));
    assert_eq!(table(16).as_deref(), Some("#000000"));
    assert_eq!(table(100).as_deref(), Some("#878700"));
    assert_eq!(table(231).as_deref(), Some("#ffffff"));
    assert_eq!(table(232).as_deref(), Some("#080808"));
    assert_eq!(table(255).as_deref(), Some("#eeeeee"));
    assert_eq!(palette.value(&AnsiColor::Table(None)), None);
    assert_eq!(
        palette.value(&AnsiColor::Rgb([1, 171, 255])).as_deref(),
        Some("#01abff")
    );
}

#[test]
fn dims_colors_as_shiki_does() {
    for (color, dimmed) in [
        ("#ABCDEF", "#ABCDEF80"),
        ("#abc", "#aabbcc80"),
        ("#abcf", "#aabbcc80"),
        ("#abcdefee", "#abcdef77"),
        ("#abcdef01", "#abcdef01"),
        ("#12345", "#12345"),
        ("red", "red"),
        ("x#zz#123", "#11223380"),
        ("var(--shiki-ansi-red)", "var(--shiki-ansi-red-dim)"),
        ("var(--ansi-red)", "var(--ansi-red)"),
    ] {
        assert_eq!(dim_color(color), dimmed, "{color}");
    }
}
