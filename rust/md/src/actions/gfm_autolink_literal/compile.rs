use textlane_core::ArenaVec;
use textlane_core::mdast::{Link, Node};

use crate::actions::text::compile::{enter_text, exit_text};
use crate::models::event::Kind;
use crate::models::token::Name;
use crate::utils::compiler::stack::Compiler;

/// A GFM literal autolink becomes a link whose text is the literal: a `www.` one points
/// to `http://` and an email one to `mailto:`.
pub(crate) fn compile_gfm_autolink_literal(
    compiler: &mut Compiler,
    name: Name,
    kind: Kind,
) {
    match (name, kind) {
        (Name::LITERAL_AUTOLINK, Kind::Enter) => compiler.tail_push(Node::Link(Link {
            url: "",
            title: None,
            children: ArenaVec::new_in(compiler.alloc),
            position: None,
        })),
        (Name::LITERAL_AUTOLINK, Kind::Exit) => compiler.tail_pop(),
        (_, Kind::Enter) => enter_text(compiler),
        (_, Kind::Exit) => {
            let raw = compiler.slice();
            exit_text(compiler, raw);
            let url = match name {
                Name::LITERAL_AUTOLINK_EMAIL => {
                    compiler.alloc_str(&format!("mailto:{raw}"))
                }
                Name::LITERAL_AUTOLINK_WWW => {
                    compiler.alloc_str(&format!("http://{raw}"))
                }
                _ => raw,
            };
            if let Node::Link(link) = compiler.tail_mut() {
                link.url = url;
            }
        }
    }
}
