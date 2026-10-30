use std::sync::LazyLock;

use regex::Regex;

use crate::models::highlight::{CodeBlock, CodeToken, add_classes};
use crate::models::options::{
    Classes, ConfiguredTransformer, HighlightTransformer, TransformerName,
};
use crate::utils::notation::apply::{Notation, apply_notation};
use crate::utils::notation::parse::{Comment, parse_comments};

static META_LINES: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"\{([\d,-]+)\}").ok());

/// Run the transformers over a block in Shiki's hook order: every `line` hook per line, then
/// every `code` hook, the notation ones sharing one parse of the block's comments.
pub(crate) fn transform_code(
    block: &mut CodeBlock,
    transformers: &[HighlightTransformer],
) {
    let transformers: Vec<ConfiguredTransformer> =
        transformers.iter().map(configure_transformer).collect();
    let highlighted = read_meta_lines(&block.meta);
    for (index, line) in block.lines.iter_mut().enumerate() {
        let number = index + 1;
        for transformer in &transformers {
            match transformer {
                ConfiguredTransformer::MetaHighlight {
                    class_name,
                    zero_indexed,
                } => {
                    if block.meta.is_empty() {
                        continue;
                    }
                    let effective = if *zero_indexed { number - 1 } else { number };
                    let effective = effective as i64;
                    if highlighted
                        .iter()
                        .any(|&(start, end)| start <= effective && effective <= end)
                    {
                        let classes = read_classes(class_name, "highlighted");
                        add_classes(&mut line.classes, &classes);
                    }
                }
                ConfiguredTransformer::RenderLineNumber {
                    class_line_number,
                    start,
                } => {
                    let token = CodeToken {
                        text: (start.unwrap_or(1) + number - 1).to_string(),
                        style: String::new(),
                        classes: read_classes(class_line_number, "line-number"),
                    };
                    line.tokens.insert(0, Some(token));
                }
                _ => {}
            }
        }
    }
    let jsx = matches!(block.language.as_str(), "jsx" | "tsx");
    let mut comments: Option<Vec<Comment>> = None;
    for transformer in &transformers {
        let Some(notation) = read_notation(transformer) else {
            continue;
        };
        let comments = comments.get_or_insert_with(|| parse_comments(block, jsx));
        apply_notation(block, comments, &notation);
    }
}

// A transformer named alone takes the options @shikijs/transformers defaults to
fn configure_transformer(transformer: &HighlightTransformer) -> ConfiguredTransformer {
    let name = match transformer {
        HighlightTransformer::Configured(configured) => return configured.clone(),
        HighlightTransformer::Name(name) => name,
    };
    match name {
        TransformerName::NotationDiff => ConfiguredTransformer::NotationDiff {
            class_line_add: None,
            class_line_remove: None,
            class_active_pre: None,
            class_active_code: None,
        },
        TransformerName::NotationFocus => ConfiguredTransformer::NotationFocus {
            class_active_line: None,
            class_active_pre: None,
            class_active_code: None,
        },
        TransformerName::NotationHighlight => ConfiguredTransformer::NotationHighlight {
            class_active_line: None,
            class_active_pre: None,
            class_active_code: None,
        },
        TransformerName::NotationErrorLevel => {
            ConfiguredTransformer::NotationErrorLevel {
                class_map: None,
                class_active_pre: None,
                class_active_code: None,
            }
        }
        TransformerName::MetaHighlight => ConfiguredTransformer::MetaHighlight {
            class_name: None,
            zero_indexed: false,
        },
        TransformerName::RenderLineNumber => ConfiguredTransformer::RenderLineNumber {
            class_line_number: None,
            start: None,
        },
    }
}

fn read_notation(transformer: &ConfiguredTransformer) -> Option<Notation> {
    let classes = read_classes;
    Some(match transformer {
        ConfiguredTransformer::NotationDiff {
            class_line_add,
            class_line_remove,
            class_active_pre,
            class_active_code,
        } => Notation {
            class_map: vec![
                ("++".into(), classes(class_line_add, "diff add")),
                ("--".into(), classes(class_line_remove, "diff remove")),
            ],
            active_pre: classes(class_active_pre, "has-diff"),
            active_code: classes(class_active_code, ""),
        },
        ConfiguredTransformer::NotationFocus {
            class_active_line,
            class_active_pre,
            class_active_code,
        } => Notation {
            class_map: vec![("focus".into(), classes(class_active_line, "focused"))],
            active_pre: classes(class_active_pre, "has-focused"),
            active_code: classes(class_active_code, ""),
        },
        ConfiguredTransformer::NotationHighlight {
            class_active_line,
            class_active_pre,
            class_active_code,
        } => {
            let line = classes(class_active_line, "highlighted");
            Notation {
                class_map: vec![("highlight".into(), line.clone()), ("hl".into(), line)],
                active_pre: classes(class_active_pre, "has-highlighted"),
                active_code: classes(class_active_code, ""),
            }
        }
        ConfiguredTransformer::NotationErrorLevel {
            class_map,
            class_active_pre,
            class_active_code,
        } => Notation {
            class_map: match class_map {
                Some(map) => map
                    .iter()
                    .map(|(key, value)| (key.clone(), split_classes(value)))
                    .collect(),
                None => ["error", "warning", "info"]
                    .into_iter()
                    .map(|level| (level.into(), vec!["highlighted".into(), level.into()]))
                    .collect(),
            },
            active_pre: classes(class_active_pre, "has-highlighted"),
            active_code: classes(class_active_code, ""),
        },
        _ => return None,
    })
}

fn read_classes(classes: &Option<Classes>, default: &str) -> Vec<String> {
    match classes {
        Some(classes) => split_classes(classes),
        None => split_classes(&Classes::One(default.into())),
    }
}

// `addClassToHast` splits a string on white space and takes a list as it is
fn split_classes(classes: &Classes) -> Vec<String> {
    match classes {
        Classes::One(text) => text.split_whitespace().map(str::to_owned).collect(),
        Classes::Many(list) => list.clone(),
    }
}

// `parseMetaHighlightString`'s lines and ranges of the first `{…}`, `NaN` never matching;
// kept as ranges, since expanding `{1-99999999999}` as upstream does exhausts memory
fn read_meta_lines(meta: &str) -> Vec<(i64, i64)> {
    let Some(captures) = META_LINES.as_ref().and_then(|lines| lines.captures(meta))
    else {
        return Vec::new();
    };
    let list = captures.get(1).map_or("", |list| list.as_str());
    list.split(',')
        .filter_map(|part| {
            let range: Vec<Option<i64>> = part.split('-').map(read_integer).collect();
            match range[..] {
                [Some(single)] => Some((single, single)),
                [Some(start), Some(end), ..] => Some((start, end)),
                _ => None,
            }
        })
        .collect()
}

// `Number.parseInt(text, 10)`: the leading digits, none being `NaN`
fn read_integer(text: &str) -> Option<i64> {
    let digits: String = text.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}
