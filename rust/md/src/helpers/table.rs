use textlane_core::mdast::AlignKind;

/// Rows of cells as a Markdown table, as markdown-table writes them: the delimiter row
/// after the first, and columns padded to line up when `align_delimiters`.
pub(crate) fn format_table(
    rows: &[Vec<String>],
    align: &[AlignKind],
    padding: bool,
    align_delimiters: bool,
) -> String {
    let columns = rows.iter().map(Vec::len).max().unwrap_or(0);
    let mut longest = vec![0; columns];
    let mut sizes: Vec<Vec<usize>> = rows
        .iter()
        .map(|row| {
            if !align_delimiters {
                return vec![];
            }
            row.iter()
                .enumerate()
                .map(|(column, cell)| {
                    // markdown-table measures `String#length`: UTF-16 code units
                    let size = cell.encode_utf16().count();
                    longest[column] = longest[column].max(size);
                    size
                })
                .collect()
        })
        .collect();
    let alignments: Vec<AlignKind> = (0..columns)
        .map(|column| align.get(column).copied().unwrap_or(AlignKind::None))
        .collect();
    let mut delimiters = Vec::with_capacity(columns);
    let mut delimiter_sizes = vec![];
    for (column, alignment) in alignments.iter().enumerate() {
        let (before, after) = match alignment {
            AlignKind::Center => (":", ":"),
            AlignKind::Left => (":", ""),
            AlignKind::Right => ("", ":"),
            AlignKind::None => ("", ""),
        };
        let marks = before.len() + after.len();
        let size = if align_delimiters {
            longest[column].saturating_sub(marks).max(1)
        } else {
            1
        };
        delimiters.push(format!("{before}{}{after}", "-".repeat(size)));
        if align_delimiters {
            longest[column] = longest[column].max(size + marks);
            delimiter_sizes.push(size + marks);
        }
    }
    let at = rows.len().min(1);
    sizes.insert(at, delimiter_sizes);
    let mut lines = Vec::with_capacity(rows.len() + 1);
    for (index, sizes) in sizes.iter().enumerate() {
        let row = match index.cmp(&at) {
            std::cmp::Ordering::Less => &rows[index],
            std::cmp::Ordering::Equal => &delimiters,
            std::cmp::Ordering::Greater => &rows[index - 1],
        };
        let mut line = String::new();
        for (column, alignment) in alignments.iter().enumerate() {
            let cell = row.get(column).map_or("", String::as_str);
            let (mut before, mut after) = (0, 0);
            if align_delimiters {
                let size = longest[column] - sizes.get(column).copied().unwrap_or(0);
                match alignment {
                    AlignKind::Right => before = size,
                    AlignKind::Center => {
                        before = size.div_ceil(2);
                        after = size / 2;
                    }
                    _ => after = size,
                }
            }
            if column == 0 {
                line.push('|');
            }
            if padding && !(!align_delimiters && cell.is_empty()) {
                line.push(' ');
            }
            line.extend(std::iter::repeat_n(' ', before));
            line.push_str(cell);
            line.extend(std::iter::repeat_n(' ', after));
            if padding {
                line.push(' ');
            }
            line.push('|');
        }
        lines.push(line);
    }
    lines.join("\n")
}
