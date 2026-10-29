use crate::models::estree::{Id, Store, Value};

const STACK_RED_ZONE: usize = 64 * 1024;
const STACK_GROWTH: usize = 1024 * 1024;
const INDENT: &str = "  ";

/// astring 1.9.0's `State` with estree-util-to-js's JSX handlers and comments on.
pub(crate) struct Printer<'s> {
    pub(crate) store: &'s Store,
    pub(crate) output: String,
    pub(crate) level: usize,
}

/// Print a program as astring does.
pub(crate) fn print_program(store: &Store, program: Id) -> String {
    let mut printer = Printer {
        store,
        output: String::new(),
        level: 0,
    };
    printer.node(program);
    printer.output
}

impl Printer<'_> {
    pub(crate) fn write(&mut self, value: &str) {
        self.output.push_str(value);
    }

    pub(crate) fn indent(&self, level: usize) -> String {
        INDENT.repeat(level)
    }

    /// astring's `generator[node.type](node, state)`.
    pub(crate) fn node(&mut self, id: Id) {
        stacker::maybe_grow(STACK_RED_ZONE, STACK_GROWTH, || {
            if !self.statement(id) && !self.expression(id) {
                self.jsx(id);
            }
        });
    }

    pub(crate) fn field(&mut self, id: Id, key: &str) {
        if let Some(child) = self.store.child(id, key) {
            self.node(child);
        }
    }

    pub(crate) fn kind(&self, id: Id) -> &str {
        self.store.kind(id)
    }

    pub(crate) fn kind_of(&self, id: Id, key: &str) -> &str {
        self.store
            .child(id, key)
            .map_or("", |child| self.store.kind(child))
    }

    /// A field's name, as JS reads `node.x.name`: `undefined` when it has none.
    pub(crate) fn name_of(&self, id: Id, key: &str) -> String {
        self.store
            .child(id, key)
            .and_then(|child| self.store.str(child, "name"))
            .unwrap_or("undefined")
            .to_string()
    }

    /// `formatSequence`: `(a, b)`.
    pub(crate) fn sequence(&mut self, nodes: &[Value]) {
        self.write("(");
        for (index, node) in nodes.iter().enumerate() {
            if index > 0 {
                self.write(", ");
            }
            if let Value::Node(id) = node {
                self.node(*id);
            }
        }
        self.write(")");
    }

    pub(crate) fn comments_of(&mut self, id: Id, key: &str, indent: &str) -> bool {
        let comments = self.store.children(id, key);
        if !self.store.has(id, key) {
            return false;
        }
        self.format_comments(&comments, indent);
        true
    }

    /// `formatComments`: each comment on its own line at `indent`.
    pub(crate) fn format_comments(&mut self, comments: &[Id], indent: &str) {
        for comment in comments {
            self.write(indent);
            let value = self.store.str(*comment, "value").unwrap_or_default();
            if self.store.kind(*comment).starts_with('L') {
                let line = format!("// {}\n", value.trim_matches(is_js_whitespace));
                self.write(&line);
            } else {
                self.write("/*");
                self.reindent(value, indent);
                self.write("*/\n");
            }
        }
    }

    fn reindent(&mut self, text: &str, indent: &str) {
        let lines: Vec<&str> = text.split('\n').collect();
        let end = lines.len() - 1;
        self.write(lines[0].trim_matches(is_js_whitespace));
        if end > 0 {
            self.write("\n");
            for line in &lines[1..end] {
                let line = format!("{indent}{}\n", line.trim_matches(is_js_whitespace));
                self.write(&line);
            }
            let last = format!("{indent}{}", lines[end].trim_matches(is_js_whitespace));
            self.write(&last);
        }
    }
}

/// What JS's `String.prototype.trim` strips.
pub(crate) fn is_js_whitespace(character: char) -> bool {
    matches!(
        character,
        '\u{9}'..='\u{d}' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}'
            | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}'
    )
}

#[cfg(test)]
#[path = "state_unit.rs"]
mod unit;
