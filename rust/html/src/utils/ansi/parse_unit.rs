use super::{AnsiColor, AnsiParser, AnsiSegment, BOLD, DIM, REVERSE, UNDERLINE};

fn segment(text: &str, foreground: Option<AnsiColor>, decorations: u8) -> AnsiSegment {
    AnsiSegment {
        text: text.to_owned(),
        foreground,
        background: None,
        decorations,
    }
}

#[test]
fn splits_a_line_at_its_sequences() {
    let segments = AnsiParser::default().parse("a\x1b[31mb\x1b[1;4mc\x1b[0md");
    assert_eq!(
        segments,
        vec![
            segment("a", None, 0),
            segment("b", Some(AnsiColor::Named(1)), 0),
            segment("c", Some(AnsiColor::Named(1)), BOLD | UNDERLINE),
            segment("d", None, 0),
        ]
    );
}

#[test]
fn keeps_its_state_from_one_line_to_the_next() {
    let mut parser = AnsiParser::default();
    parser.parse("\x1b[7;92m");
    assert_eq!(
        parser.parse("a"),
        vec![segment("a", Some(AnsiColor::Named(10)), REVERSE)]
    );
}

#[test]
fn resets_before_it_sets_within_a_sequence() {
    let segments = AnsiParser::default().parse("\x1b[31;0ma\x1b[2;1;22mb");
    assert_eq!(
        segments,
        vec![
            segment("a", Some(AnsiColor::Named(1)), 0),
            segment("b", Some(AnsiColor::Named(1)), BOLD | DIM),
        ]
    );
}

#[test]
fn reads_256_and_rgb_colors_as_javascript_numbers() {
    let mut parser = AnsiParser::default();
    let colors: Vec<Option<AnsiColor>> = [
        "\x1b[38;5;196ma",
        "\x1b[38;5;0x10ma",
        "\x1b[38;5; 7 ma",
        "\x1b[38;5;256ma",
        "\x1b[38;5;1.5ma",
        "\x1b[38;2;300;-4;0x1fma",
    ]
    .iter()
    .map(|line| {
        parser
            .parse(line)
            .first()
            .and_then(|segment| segment.foreground)
    })
    .collect();
    assert_eq!(
        colors,
        vec![
            Some(AnsiColor::Table(Some(196))),
            Some(AnsiColor::Table(Some(16))),
            Some(AnsiColor::Table(Some(7))),
            Some(AnsiColor::Table(None)),
            Some(AnsiColor::Table(None)),
            Some(AnsiColor::Rgb([255, 0, 31])),
        ]
    );
}

#[test]
fn takes_an_unfinished_sequence_as_text() {
    let segments = AnsiParser::default().parse("a\x1bb\x1b[31mc");
    assert_eq!(segments, vec![segment("a\x1bb\x1b[31mc", None, 0)]);
    let segments = AnsiParser::default().parse("\x1b[31ma\x1b[1 b");
    assert_eq!(
        segments,
        vec![segment("a\x1b[1 b", Some(AnsiColor::Named(1)), 0)]
    );
}

#[test]
fn parses_many_sequences_in_linear_time() {
    let line = "\x1b[1;38;2;1;2;3;48;5;100mab".repeat(200_000);
    let started = std::time::Instant::now();
    let segments = AnsiParser::default().parse(&line);
    assert_eq!(segments.len(), 200_000);
    let parameters = format!("\x1b[{}m", "1;".repeat(1_000_000));
    assert!(AnsiParser::default().parse(&parameters).is_empty());
    let unclosed = "\x1b[31".repeat(500_000);
    assert_eq!(AnsiParser::default().parse(&unclosed).len(), 1);
    assert!(
        started.elapsed().as_secs() < 20,
        "took {:?}",
        started.elapsed()
    );
}
