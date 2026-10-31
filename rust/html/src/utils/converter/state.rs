use std::borrow::Cow;
use std::collections::HashMap;

use textlane_core::Allocator;
use textlane_core::hast::{Element, Node};
use url::Url;

use crate::actions::a::convert::convert_a;
use crate::actions::attribute::read::read_attribute;
use crate::actions::base::convert::convert_base;
use crate::actions::blockquote::convert::convert_blockquote;
use crate::actions::br::convert::convert_br;
use crate::actions::code::convert::convert_code;
use crate::actions::comment::convert::convert_comment;
use crate::actions::del::convert::convert_del;
use crate::actions::dl::convert::convert_dl;
use crate::actions::em::convert::convert_em;
use crate::actions::heading::convert::convert_heading;
use crate::actions::hr::convert::convert_hr;
use crate::actions::iframe::convert::convert_iframe;
use crate::actions::img::convert::convert_img;
use crate::actions::inline_code::convert::convert_inline_code;
use crate::actions::input::convert::convert_input;
use crate::actions::li::convert::convert_li;
use crate::actions::list::convert::convert_list;
use crate::actions::math::convert::convert_math;
use crate::actions::media::convert::convert_media;
use crate::actions::p::convert::convert_p;
use crate::actions::q::convert::convert_q;
use crate::actions::root::convert::convert_root;
use crate::actions::select::convert::convert_select;
use crate::actions::strong::convert::convert_strong;
use crate::actions::table::convert::convert_table;
use crate::actions::table_cell::convert::convert_table_cell;
use crate::actions::table_row::convert::convert_table_row;
use crate::actions::text::convert::convert_text;
use crate::actions::textarea::convert::convert_textarea;
use crate::actions::unknown::convert::convert_unknown;
use crate::actions::wbr::convert::convert_wbr;
use crate::helpers::selected::OptionValues;
use crate::helpers::value::{find_string, find_value, is_truthy, read_value};
use crate::models::options::ToMarkdown;
use crate::utils::converter::store::{Converted, Id, Kind, Store};

// Handlers recurse once per level of the tree; past the red zone the stack grows on the heap
const STACK_RED_ZONE: usize = 64 * 1024;
const STACK_GROWTH: usize = 1024 * 1024;

/// Turns hast into mdast, as hast-util-to-mdast's state does.
pub(crate) struct Converter<'a, 'n, 'o> {
    pub(crate) alloc: &'a Allocator,
    pub(crate) options: &'o ToMarkdown,
    pub(crate) store: Store<'a>,
    /// The first element with each `id`, for an `input`'s `list`.
    pub(crate) element_by_id: HashMap<String, &'n Element<'a>>,
    pub(crate) base_found: bool,
    /// The first `<base>`'s `href`, which later URLs resolve against.
    pub(crate) base: Option<Url>,
    pub(crate) in_table: bool,
    pub(crate) q_nesting: usize,
    /// Whether each list item's content is spread out, by element address, so nested items
    /// are not searched again for each one around them.
    pub(crate) spread: HashMap<usize, bool>,
    /// The options each `<datalist>` shows, by its address and how many may show, so
    /// inputs that share one do not each search all its options.
    pub(crate) datalists: HashMap<(usize, usize), OptionValues>,
}

impl<'a, 'n> Converter<'a, 'n, '_> {
    /// Convert one hast node.
    pub(crate) fn one(&mut self, node: &'n Node<'a>) -> Converted {
        stacker::maybe_grow(STACK_RED_ZONE, STACK_GROWTH, || match node {
            Node::Element(element) => {
                if find_string(element, "dataMdast") == Some("ignore") {
                    return Converted::Nothing;
                }
                self.element(element)
            }
            Node::Root(root) => convert_root(self, root),
            Node::Text(text) => convert_text(self, text),
            Node::Comment(comment) => convert_comment(self, comment),
            Node::Doctype(_) => Converted::Nothing,
            Node::Raw(raw) => {
                Converted::One(self.store.add_text(raw.value, raw.position))
            }
            Node::Unknown(unknown) => convert_unknown(self, unknown),
            // As unknown nodes: a value is text, an element its children
            Node::MdxJsxFlowElement(element) | Node::MdxJsxTextElement(element) => {
                Converted::Many(self.all(&element.children))
            }
            Node::MdxFlowExpression(expression) | Node::MdxTextExpression(expression) => {
                Converted::One(self.store.add_text(expression.value, expression.position))
            }
            Node::MdxjsEsm(esm) => {
                Converted::One(self.store.add_text(esm.value, esm.position))
            }
        })
    }

    /// Convert each of some hast nodes, in order.
    pub(crate) fn all(&mut self, children: &'n [Node<'a>]) -> Vec<Id> {
        let mut results = vec![];
        for child in children {
            let converted = self.one(child);
            push_converted(&mut results, converted);
        }
        results
    }

    /// Wrap runs of phrasing content in paragraphs.
    pub(crate) fn wrap_flow(&mut self, nodes: Vec<Id>) -> Vec<Id> {
        self.store.wrap(nodes)
    }

    /// Make every node a `reference` node, such as a list item: others join the next one,
    /// or the last one if none follows.
    pub(crate) fn wrap_specific_content(
        &mut self,
        nodes: Vec<Id>,
        reference: Kind<'a>,
    ) -> Vec<Id> {
        let nodes = self.store.expand(&nodes).into_owned();
        let expected = std::mem::discriminant(&reference);
        let mut results: Vec<Id> = vec![];
        let mut queue: Vec<Id> = vec![];
        for node in nodes {
            if std::mem::discriminant(&self.store.items[node].kind) == expected {
                if !queue.is_empty() {
                    let children = self.store.take_children(node);
                    queue.extend(children);
                    self.store.set_children(node, std::mem::take(&mut queue));
                }
                results.push(node);
            } else {
                queue.push(node);
            }
        }
        if !queue.is_empty() {
            let node = match results.last() {
                Some(node) => *node,
                None => {
                    let node = self.store.add(reference, vec![], None);
                    results.push(node);
                    node
                }
            };
            let mut children = self.store.take_children(node);
            children.extend(queue);
            self.store.set_children(node, children);
        }
        results
    }

    /// Resolve a URL against the document's `<base>`; no URL is an empty one.
    pub(crate) fn resolve(&self, url: Option<&str>) -> &'a str {
        let Some(url) = url else {
            return "";
        };
        match self.base.as_ref().map(|base| base.join(url)) {
            Some(Ok(resolved)) => self.alloc.alloc_str(resolved.as_str()),
            _ => self.alloc.alloc_str(url),
        }
    }

    /// A string that lives as long as the tree.
    pub(crate) fn intern(&self, value: Cow<'a, str>) -> &'a str {
        match value {
            Cow::Borrowed(value) => value,
            Cow::Owned(value) => self.alloc.alloc_str(&value),
        }
    }

    /// `properties[name] ? String(properties[name]) : null`.
    pub(crate) fn read_optional(
        &self,
        element: &Element<'a>,
        name: &str,
    ) -> Option<&'a str> {
        is_truthy(find_value(element, name))
            .then(|| self.intern(read_value(element, name)))
    }

    fn element(&mut self, element: &'n Element<'a>) -> Converted {
        if self.options.math {
            if element.tag_name == "math" {
                return convert_math(self, element);
            }
            let classes = read_attribute(element, "class").unwrap_or_default();
            if classes
                .split_ascii_whitespace()
                .any(|class| class == "katex-html")
            {
                return Converted::Nothing;
            }
        }
        match element.tag_name {
            "applet" | "area" | "basefont" | "bgsound" | "caption" | "col"
            | "colgroup" | "command" | "content" | "datalist" | "dialog" | "element"
            | "embed" | "frame" | "frameset" | "isindex" | "keygen" | "link" | "math"
            | "menu" | "menuitem" | "meta" | "nextid" | "noembed" | "noframes"
            | "optgroup" | "option" | "param" | "script" | "shadow" | "source"
            | "spacer" | "style" | "svg" | "template" | "title" | "track" => {
                Converted::Nothing
            }
            "address" | "article" | "aside" | "body" | "center" | "div" | "fieldset"
            | "figcaption" | "figure" | "form" | "footer" | "header" | "hgroup"
            | "html" | "legend" | "main" | "multicol" | "nav" | "picture" | "section" => {
                let children = self.all(&element.children);
                let mut children = self.wrap_flow(children);
                if children.len() > 1 {
                    children = vec![self.store.add(Kind::Fragment, children, None)];
                }
                Converted::Many(children)
            }
            "a" => convert_a(self, element),
            "audio" | "video" => convert_media(self, element),
            "b" | "strong" => convert_strong(self, element),
            "base" => convert_base(self, element),
            "blockquote" => convert_blockquote(self, element),
            "br" => convert_br(self, element),
            "code" | "kbd" | "samp" | "tt" | "var" => convert_inline_code(self, element),
            "dir" | "ol" | "ul" => convert_list(self, element),
            "dl" => convert_dl(self, element),
            "dt" | "dd" | "li" => convert_li(self, element),
            "del" | "s" | "strike" => convert_del(self, element),
            "em" | "i" | "mark" | "u" => convert_em(self, element),
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => convert_heading(self, element),
            "hr" => convert_hr(self, element),
            "iframe" => convert_iframe(self, element),
            "img" | "image" => convert_img(self, element),
            "input" => convert_input(self, element),
            "listing" | "plaintext" | "pre" | "xmp" => convert_code(self, element),
            "p" | "summary" => convert_p(self, element),
            "q" => convert_q(self, element),
            "select" => convert_select(self, element),
            "table" => convert_table(self, element),
            "td" | "th" => convert_table_cell(self, element),
            "textarea" => convert_textarea(self, element),
            "tr" => convert_table_row(self, element),
            "wbr" => convert_wbr(self, element),
            // `abbr`, `span`, `tbody` and the like, and unknown elements, are transparent
            _ => Converted::Many(self.all(&element.children)),
        }
    }
}

/// Add what a handler returned to a list of results.
pub(crate) fn push_converted(results: &mut Vec<Id>, converted: Converted) {
    match converted {
        Converted::Nothing => {}
        Converted::One(id) => results.push(id),
        Converted::Many(ids) => results.extend(ids),
    }
}
