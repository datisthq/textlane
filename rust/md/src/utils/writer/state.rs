use std::cell::OnceCell;
use std::collections::HashMap;

use textlane_core::mdast::{self, AlignKind, Node};

use crate::actions::list_item::convert::{is_item_loose, is_list_loose};
use crate::actions::mdast::convert::find_definitions;
use crate::helpers::lines::trim_lines;
use crate::helpers::uri::normalize_uri;
use crate::models::options::ToHtml;
use crate::utils::writer::escape::{push_attribute, push_text};

// Deeper trees take the hast path, which walks without recursing
const MAX_DEPTH: usize = 256;

/// Writes HTML from mdast as `md.toHtml` and then `html.stringify` would, for the nodes
/// it knows; anything else makes it give up so the caller takes the hast path.
pub(crate) struct Writer<'o, 'a> {
    pub(crate) options: &'o ToHtml,
    /// Whether raw HTML is written as it is, `html.stringify`'s `allowDangerousHtml`.
    pub(crate) dangerous: bool,
    pub(crate) root: &'o Node<'a>,
    pub(crate) definitions: OnceCell<HashMap<String, (&'a str, Option<&'a str>)>>,
    pub(crate) output: String,
    pub(crate) depth: usize,
}

/// The writer met something it does not write.
pub(crate) struct GiveUp;

type Written = Result<(), GiveUp>;

// What a node turns into in hast, which separators and list items count
#[derive(Clone, Copy, PartialEq)]
enum Produce {
    Nothing,
    Paragraph,
    Other,
    // `br` and the line ending after it
    Break,
}

impl<'a> Writer<'_, 'a> {
    pub(crate) fn write_root(&mut self) -> Written {
        match self.root {
            Node::Root(root) if root.data.is_none() => {
                self.write_wrapped(&root.children, false)
            }
            _ => Err(GiveUp),
        }
    }

    fn produce(&self, node: &Node<'a>) -> Produce {
        match node {
            Node::Definition(_) | Node::FootnoteDefinition(_) => Produce::Nothing,
            Node::Html(_) if !self.options.allow_dangerous_html => Produce::Nothing,
            Node::Paragraph(_) => Produce::Paragraph,
            Node::Break(_) => Produce::Break,
            _ => Produce::Other,
        }
    }

    // `trim` is mdast-util-to-hast's trimMarkdownSpaceStart on the node's first hast node:
    // a text loses its leading spaces and tabs, an element its first child's
    fn write_node(&mut self, node: &Node<'a>, trim: bool) -> Written {
        if node.data().is_some() {
            return Err(GiveUp);
        }
        match node {
            Node::Paragraph(node) => self.write_element("p", &node.children, trim),
            Node::Heading(node) => {
                let tag = ["h1", "h2", "h3", "h4", "h5", "h6"]
                    [usize::from(node.depth.clamp(1, 6)) - 1];
                self.write_element(tag, &node.children, trim)
            }
            Node::Emphasis(node) => self.write_element("em", &node.children, trim),
            Node::Strong(node) => self.write_element("strong", &node.children, trim),
            Node::Delete(node) => self.write_element("del", &node.children, trim),
            Node::Text(node) => {
                let value = trim_lines(node.value);
                let value = if trim {
                    value.trim_start_matches([' ', '\t'])
                } else {
                    &value
                };
                push_text(&mut self.output, value);
                Ok(())
            }
            Node::InlineCode(node) => {
                let value = if node.value.contains(['\n', '\r']) {
                    node.value.replace("\r\n", " ").replace(['\n', '\r'], " ")
                } else {
                    node.value.to_string()
                };
                let value = if trim {
                    value.trim_start_matches([' ', '\t'])
                } else {
                    &value
                };
                self.output.push_str("<code>");
                push_text(&mut self.output, value);
                self.output.push_str("</code>");
                Ok(())
            }
            Node::Code(node) => {
                self.output.push_str("<pre><code");
                if let Some(language) =
                    node.lang.and_then(|lang| lang.split_whitespace().next())
                {
                    self.output.push_str(" class=\"language-");
                    push_attribute(&mut self.output, language);
                    self.output.push('"');
                }
                self.output.push('>');
                push_text(&mut self.output, node.value);
                if !node.value.is_empty() {
                    self.output.push('\n');
                }
                self.output.push_str("</code></pre>");
                Ok(())
            }
            Node::Blockquote(node) => {
                self.output.push_str("<blockquote>");
                self.write_wrapped(&node.children, true)?;
                self.output.push_str("</blockquote>");
                Ok(())
            }
            Node::List(node) => self.write_list(node),
            Node::ListItem(node) => self.write_list_item(node, None),
            Node::ThematicBreak(_) => {
                self.output.push_str("<hr>");
                Ok(())
            }
            Node::Break(_) => {
                self.output.push_str("<br>\n");
                Ok(())
            }
            Node::Link(node) => {
                self.write_anchor(node.url, node.title)?;
                self.write_children(&node.children, trim)?;
                self.output.push_str("</a>");
                Ok(())
            }
            Node::LinkReference(node) => {
                let (url, title) = self.find_definition(node.identifier).ok_or(GiveUp)?;
                self.write_anchor(url, title)?;
                self.write_children(&node.children, trim)?;
                self.output.push_str("</a>");
                Ok(())
            }
            Node::Image(node) => {
                self.write_image(node.url, node.alt, node.title);
                Ok(())
            }
            Node::ImageReference(node) => {
                let (url, title) = self.find_definition(node.identifier).ok_or(GiveUp)?;
                self.write_image(url, node.alt, title);
                Ok(())
            }
            Node::Html(node) => {
                if self.options.allow_dangerous_html {
                    if self.dangerous {
                        self.output.push_str(node.value);
                    } else {
                        push_text(&mut self.output, node.value);
                    }
                }
                Ok(())
            }
            Node::Table(node) => self.write_table(node),
            Node::Definition(_) | Node::FootnoteDefinition(_) => Ok(()),
            _ => Err(GiveUp),
        }
    }

    fn write_element(&mut self, tag: &str, children: &[Node<'a>], trim: bool) -> Written {
        self.output.push('<');
        self.output.push_str(tag);
        self.output.push('>');
        self.write_children(children, trim)?;
        self.output.push_str("</");
        self.output.push_str(tag);
        self.output.push('>');
        Ok(())
    }

    // Children one after another; a text right after a break loses its leading spaces, and
    // so does the first child when it is text and the parent was trimmed
    fn write_children(&mut self, children: &[Node<'a>], trim: bool) -> Written {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(GiveUp);
        }
        let first = children
            .iter()
            .position(|child| self.produce(child) != Produce::Nothing);
        for (index, child) in children.iter().enumerate() {
            let after_break = index > 0 && matches!(children[index - 1], Node::Break(_));
            let inherited =
                trim && Some(index) == first && matches!(child, Node::Text(_));
            self.write_node(child, after_break || inherited)?;
        }
        self.depth -= 1;
        Ok(())
    }

    // mdast-util-to-hast's `wrap`: a line ending between the hast nodes, and around them
    // when `loose`
    fn write_wrapped(&mut self, children: &[Node<'a>], loose: bool) -> Written {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(GiveUp);
        }
        if loose {
            self.output.push('\n');
        }
        let mut any = false;
        for (index, child) in children.iter().enumerate() {
            let produce = self.produce(child);
            if produce == Produce::Nothing {
                continue;
            }
            if any {
                self.output.push('\n');
            }
            any = true;
            let after_break = index > 0 && matches!(children[index - 1], Node::Break(_));
            if produce == Produce::Break {
                if child.data().is_some() {
                    return Err(GiveUp);
                }
                // `br`, then its line ending as a node of its own
                self.output.push_str("<br>\n\n");
                continue;
            }
            self.write_node(child, after_break)?;
        }
        if loose && any {
            self.output.push('\n');
        }
        self.depth -= 1;
        Ok(())
    }

    fn write_list(&mut self, list: &mdast::List<'a>) -> Written {
        let tag = if list.ordered { "ol" } else { "ul" };
        self.output.push('<');
        self.output.push_str(tag);
        if let Some(start) = list.start.filter(|start| *start != 1) {
            self.output.push_str(" start=\"");
            self.output.push_str(&start.to_string());
            self.output.push('"');
        }
        let tasks = list
            .children
            .iter()
            .any(|child| matches!(child, Node::ListItem(item) if item.checked.is_some()));
        if tasks {
            self.output.push_str(" class=\"contains-task-list\"");
        }
        self.output.push('>');
        let loose = is_list_loose(list);
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(GiveUp);
        }
        // Items are wrapped as any loose parent's children are
        self.output.push('\n');
        let mut any = false;
        for (index, child) in list.children.iter().enumerate() {
            if self.produce(child) == Produce::Nothing {
                continue;
            }
            if any {
                self.output.push('\n');
            }
            any = true;
            let after_break =
                index > 0 && matches!(list.children[index - 1], Node::Break(_));
            match child {
                // A trimmed item would lose the spaces of its first text
                Node::ListItem(_) if after_break => return Err(GiveUp),
                Node::ListItem(item) if child.data().is_none() => {
                    self.write_list_item(item, Some(loose))?;
                }
                Node::Break(_) if child.data().is_none() => {
                    self.output.push_str("<br>\n\n");
                }
                _ => self.write_node(child, after_break)?,
            }
        }
        if any {
            self.output.push('\n');
        }
        self.depth -= 1;
        self.output.push_str("</");
        self.output.push_str(tag);
        self.output.push('>');
        Ok(())
    }

    // `li`: a tight item's paragraphs unwrap into their content, a line ending goes before
    // each child but such a first paragraph and after the last unless it was one
    fn write_list_item(
        &mut self,
        item: &mdast::ListItem<'a>,
        loose: Option<bool>,
    ) -> Written {
        let loose = loose.unwrap_or_else(|| is_item_loose(item));
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(GiveUp);
        }
        if item.checked.is_some() {
            self.output.push_str("<li class=\"task-list-item\">");
        } else {
            self.output.push_str("<li>");
        }
        let units = self.list_units(&item.children);
        // A checkbox goes into the first paragraph, made empty when there is none
        let virtual_paragraph = item.checked.is_some()
            && !matches!(units.first(), Some((_, Produce::Paragraph)));
        let count = units.len() + usize::from(virtual_paragraph);
        let tail_is_paragraph = match units.last() {
            Some((_, produce)) => *produce == Produce::Paragraph,
            None => virtual_paragraph,
        };
        let mut position = 0;
        if virtual_paragraph {
            if loose {
                self.output.push_str("\n<p>");
            }
            self.write_checkbox(item.checked == Some(true), false);
            if loose {
                self.output.push_str("</p>");
            }
            position += 1;
        }
        for (unit, (index, produce)) in units.iter().enumerate() {
            let child = &item.children[*index];
            let paragraph = *produce == Produce::Paragraph;
            if loose || position != 0 || !paragraph {
                self.output.push('\n');
            }
            let after_break =
                *index > 0 && matches!(item.children[*index - 1], Node::Break(_));
            match child {
                Node::Paragraph(node) if paragraph => {
                    if child.data().is_some() {
                        return Err(GiveUp);
                    }
                    if loose {
                        self.output.push_str("<p>");
                    }
                    if position == 0 && item.checked.is_some() {
                        let empty = node
                            .children
                            .iter()
                            .all(|child| self.produce(child) == Produce::Nothing);
                        self.write_checkbox(item.checked == Some(true), !empty);
                    }
                    self.write_children(&node.children, after_break)?;
                    if loose {
                        self.output.push_str("</p>");
                    }
                }
                Node::Break(_) if child.data().is_some() => return Err(GiveUp),
                // The second unit of a break is its line ending
                Node::Break(_) if unit > 0 && units[unit - 1].0 == *index => {
                    self.output.push('\n');
                }
                Node::Break(_) => self.output.push_str("<br>"),
                _ => self.write_node(child, after_break)?,
            }
            position += 1;
        }
        if count > 0 && (loose || !tail_is_paragraph) {
            self.output.push('\n');
        }
        self.output.push_str("</li>");
        self.depth -= 1;
        Ok(())
    }

    // Each child that turns into hast with what it turns into, a break as its two nodes
    fn list_units(&self, children: &[Node<'a>]) -> Vec<(usize, Produce)> {
        let mut units = Vec::with_capacity(children.len());
        for (index, child) in children.iter().enumerate() {
            match self.produce(child) {
                Produce::Nothing => {}
                Produce::Break => {
                    units.push((index, Produce::Other));
                    units.push((index, Produce::Other));
                }
                produce => units.push((index, produce)),
            }
        }
        units
    }

    fn write_checkbox(&mut self, checked: bool, space: bool) {
        self.output.push_str(if checked {
            "<input type=\"checkbox\" checked disabled>"
        } else {
            "<input type=\"checkbox\" disabled>"
        });
        if space {
            self.output.push(' ');
        }
    }

    fn write_anchor(&mut self, url: &str, title: Option<&str>) -> Written {
        self.output.push_str("<a href=\"");
        push_attribute(&mut self.output, &normalize_uri(url));
        self.output.push('"');
        if let Some(title) = title {
            self.output.push_str(" title=\"");
            push_attribute(&mut self.output, title);
            self.output.push('"');
        }
        self.output.push('>');
        Ok(())
    }

    fn write_image(&mut self, url: &str, alt: Option<&str>, title: Option<&str>) {
        self.output.push_str("<img src=\"");
        push_attribute(&mut self.output, &normalize_uri(url));
        self.output.push('"');
        for (name, value) in [(" alt=\"", alt), (" title=\"", title)] {
            if let Some(value) = value {
                self.output.push_str(name);
                push_attribute(&mut self.output, value);
                self.output.push('"');
            }
        }
        self.output.push('>');
    }

    // `table`: the first row in `thead`, the rest in `tbody`, each wrapped loosely
    fn write_table(&mut self, table: &mdast::Table<'a>) -> Written {
        if table
            .children
            .iter()
            .any(|row| !matches!(row, Node::TableRow(_)) || row.data().is_some())
        {
            return Err(GiveUp);
        }
        self.output.push_str("<table>\n");
        let mut rows = table.children.iter();
        if let Some(first) = rows.next() {
            self.output.push_str("<thead>\n");
            self.write_row(table, first, true)?;
            self.output.push_str("\n</thead>\n");
        }
        if table.children.len() > 1 {
            self.output.push_str("<tbody>\n");
            for (index, row) in rows.enumerate() {
                if index > 0 {
                    self.output.push('\n');
                }
                self.write_row(table, row, false)?;
            }
            self.output.push_str("\n</tbody>\n");
        }
        self.output.push_str("</table>");
        Ok(())
    }

    fn write_row(
        &mut self,
        table: &mdast::Table<'a>,
        row: &Node<'a>,
        head: bool,
    ) -> Written {
        let Node::TableRow(row) = row else {
            return Err(GiveUp);
        };
        if row
            .children
            .iter()
            .any(|cell| !matches!(cell, Node::TableCell(_)) || cell.data().is_some())
        {
            return Err(GiveUp);
        }
        let tag = if head { "th" } else { "td" };
        let align = (!table.align.is_empty()).then_some(&table.align);
        let length = align.map_or(row.children.len(), |align| align.len());
        self.output.push_str("<tr>\n");
        for index in 0..length {
            self.output.push('<');
            self.output.push_str(tag);
            match align.and_then(|align| align.get(index)) {
                Some(AlignKind::Left) => self.output.push_str(" align=\"left\""),
                Some(AlignKind::Right) => self.output.push_str(" align=\"right\""),
                Some(AlignKind::Center) => self.output.push_str(" align=\"center\""),
                _ => {}
            }
            self.output.push('>');
            if let Some(Node::TableCell(cell)) = row.children.get(index) {
                self.write_children(&cell.children, false)?;
            }
            self.output.push_str("</");
            self.output.push_str(tag);
            self.output.push_str(">\n");
        }
        self.output.push_str("</tr>");
        Ok(())
    }

    fn find_definition(&self, identifier: &str) -> Option<(&'a str, Option<&'a str>)> {
        self.definitions
            .get_or_init(|| find_definitions(self.root))
            .get(&identifier.to_uppercase())
            .copied()
    }
}
