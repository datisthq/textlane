/// A container whose lines get a prefix, as mdast-util-to-markdown's `indentLines` maps them.
#[derive(Clone, Debug)]
pub(crate) enum Layer {
    Blockquote,
    CodeIndented,
    DefListDescription,
    FootnoteDefinition { indent_first: bool },
    ListItem { bullet: String, size: usize },
}

impl Layer {
    fn write_prefix(&self, line: usize, blank: bool, output: &mut String) {
        match self {
            Layer::Blockquote => output.push_str(if blank { ">" } else { "> " }),
            Layer::CodeIndented => {
                if !blank {
                    output.push_str("    ");
                }
            }
            Layer::DefListDescription => {
                if line == 0 {
                    output.push_str(":   ");
                } else if !blank {
                    output.push_str("    ");
                }
            }
            Layer::FootnoteDefinition { indent_first } => {
                if !blank && (line > 0 || *indent_first) {
                    output.push_str("    ");
                }
            }
            Layer::ListItem { bullet, size } => {
                if line == 0 {
                    output.push_str(bullet);
                }
                if !blank {
                    let width = if line == 0 {
                        size - bullet.len()
                    } else {
                        *size
                    };
                    output.extend(std::iter::repeat_n(' ', width));
                }
            }
        }
    }
}

/// Output whose container prefixes are applied once, when it is finished: re-indenting each
/// container's text, as upstream does, is quadratic in the nesting depth.
#[derive(Debug, Default)]
pub(crate) struct Sink {
    text: String,
    marks: Vec<(usize, Option<Layer>)>,
}

struct Open {
    layer: Layer,
    line: usize,
}

impl Sink {
    pub(crate) fn write(&mut self, value: &str) {
        self.text.push_str(value);
    }

    pub(crate) fn open(&mut self, layer: Layer) {
        self.marks.push((self.text.len(), Some(layer)));
    }

    pub(crate) fn close(&mut self) {
        self.marks.push((self.text.len(), None));
    }

    /// Apply the prefixes, where a line of a container is blank when nothing, not even an
    /// inner container's prefix, is on it.
    pub(crate) fn finish(self) -> String {
        if self.marks.is_empty() {
            return self.text;
        }
        let mut output = String::with_capacity(self.text.len());
        let mut open: Vec<Open> = Vec::new();
        // Containers from `pending` on have not written their prefix on the current line
        let mut pending = 0;
        let mut start = 0;
        for (offset, mark) in self.marks {
            write_lines(
                &self.text[start..offset],
                &mut open,
                &mut pending,
                &mut output,
            );
            start = offset;
            match mark {
                Some(layer) => open.push(Open { layer, line: 0 }),
                None => close_layer(&mut open, &mut pending, &mut output),
            }
        }
        write_lines(&self.text[start..], &mut open, &mut pending, &mut output);
        output
    }
}

fn write_lines(text: &str, open: &mut [Open], pending: &mut usize, output: &mut String) {
    let bytes = text.as_bytes();
    let mut start = 0;
    while start < bytes.len() {
        let end = bytes[start..]
            .iter()
            .position(|byte| matches!(byte, b'\n' | b'\r'))
            .map_or(bytes.len(), |index| start + index);
        if end > start {
            write_prefixes(&mut open[*pending..], false, output);
            *pending = open.len();
            output.push_str(&text[start..end]);
        }
        if end == bytes.len() {
            break;
        }
        // `\r\n` is one line ending
        if bytes[end] == b'\n' && output.ends_with('\r') && *pending == 0 {
            output.push('\n');
        } else {
            write_blank_prefixes(&mut open[*pending..], output);
            output.push(bytes[end] as char);
            *pending = 0;
        }
        start = end + 1;
    }
}

fn close_layer(open: &mut Vec<Open>, pending: &mut usize, output: &mut String) {
    let Some(last) = open.len().checked_sub(1) else {
        return;
    };
    if *pending <= last {
        let mut prefix = String::new();
        open[last]
            .layer
            .write_prefix(open[last].line, true, &mut prefix);
        open[last].line += 1;
        if !prefix.is_empty() {
            write_prefixes(&mut open[*pending..last], false, output);
            output.push_str(&prefix);
            *pending = open.len();
        }
    }
    open.pop();
    *pending = (*pending).min(open.len());
}

fn write_prefixes(open: &mut [Open], blank: bool, output: &mut String) {
    for container in open {
        container.layer.write_prefix(container.line, blank, output);
        container.line += 1;
    }
}

// The innermost prefix decides whether the line is still blank for the containers around it
fn write_blank_prefixes(open: &mut [Open], output: &mut String) {
    let mut prefixes = vec![String::new(); open.len()];
    let mut blank = true;
    for (container, prefix) in open.iter_mut().zip(prefixes.iter_mut()).rev() {
        container.layer.write_prefix(container.line, blank, prefix);
        container.line += 1;
        blank = blank && prefix.is_empty();
    }
    for prefix in prefixes {
        output.push_str(&prefix);
    }
}
