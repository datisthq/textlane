use textlane_core::ArenaVec;
use textlane_core::mdast::{
    Image, ImageReference, Link, LinkReference, Node, ReferenceType,
};

use crate::actions::text::compile::{enter_text, exit_text};
use crate::helpers::decode::decode_string;
use crate::helpers::identifier::normalize_identifier;
use crate::models::event::Kind;
use crate::models::token::Name;
use crate::utils::compiler::stack::{Compiler, Media};

/// Links, images and autolinks; a label with no resource after it makes a reference, as
/// mdast-util-from-markdown decides when the media closes.
pub(crate) fn compile_link(compiler: &mut Compiler, name: Name, kind: Kind) {
    match (name, kind) {
        (Name::LINK | Name::AUTOLINK, Kind::Enter) => {
            compiler.tail_push(Node::Link(Link {
                url: "",
                title: None,
                children: ArenaVec::new_in(compiler.alloc),
                position: None,
            }));
            if name == Name::LINK {
                compiler.media.push(Media::default());
            }
        }
        (Name::IMAGE, Kind::Enter) => {
            compiler.tail_push(Node::Image(Image {
                url: "",
                title: None,
                alt: None,
                position: None,
            }));
            compiler.media.push(Media::default());
        }
        (Name::LABEL, Kind::Enter)
        | (
            Name::REFERENCE_STRING
            | Name::RESOURCE_DESTINATION_STRING
            | Name::RESOURCE_TITLE_STRING,
            Kind::Enter,
        ) => compiler.buffer(),
        (Name::LABEL_TEXT, Kind::Exit) => {
            let index = compiler.index;
            if let Some(media) = compiler.media.last_mut() {
                media.label_text = Some(index);
            }
        }
        (Name::LABEL, Kind::Exit) => {
            let Some(fragment) = compiler.resume_node() else {
                return;
            };
            let text = compiler.text_of(&fragment);
            if let Some(media) = compiler.media.last_mut() {
                media.in_reference = true;
            }
            match (compiler.tail_mut(), fragment) {
                (Node::Link(link), Node::Paragraph(fragment)) => {
                    link.children = fragment.children
                }
                (Node::Image(image), _) => image.alt = Some(text),
                _ => {}
            }
        }
        (Name::REFERENCE, Kind::Enter) => {
            if let Some(media) = compiler.media.last_mut() {
                media.reference_type = Some(ReferenceType::Collapsed);
            }
        }
        (Name::REFERENCE_STRING, Kind::Exit) => {
            let label = compiler.resume();
            let raw = compiler.slice();
            let identifier =
                compiler.alloc_str(&normalize_identifier(raw).to_lowercase());
            if let Some(media) = compiler.media.last_mut() {
                media.reference_type = Some(ReferenceType::Full);
                media.label = Some(label);
                media.identifier = identifier;
            }
        }
        (Name::RESOURCE, Kind::Enter) => {
            if let Some(media) = compiler.media.last_mut() {
                media.in_reference = false;
            }
        }
        (Name::RESOURCE_DESTINATION_STRING, Kind::Exit) => {
            let url = compiler.resume();
            match compiler.tail_mut() {
                Node::Link(link) => link.url = url,
                Node::Image(image) => image.url = url,
                _ => {}
            }
        }
        (Name::RESOURCE_TITLE_STRING, Kind::Exit) => {
            let title = Some(compiler.resume());
            match compiler.tail_mut() {
                Node::Link(link) => link.title = title,
                Node::Image(image) => image.title = title,
                _ => {}
            }
        }
        (Name::AUTOLINK_PROTOCOL | Name::AUTOLINK_EMAIL, Kind::Enter) => {
            enter_text(compiler)
        }
        (Name::AUTOLINK_PROTOCOL | Name::AUTOLINK_EMAIL, Kind::Exit) => {
            let raw = compiler.slice();
            exit_text(compiler, raw);
            let url = if name == Name::AUTOLINK_EMAIL {
                compiler.alloc_str(&format!("mailto:{raw}"))
            } else {
                raw
            };
            if let Node::Link(link) = compiler.tail_mut() {
                link.url = url;
            }
        }
        (Name::AUTOLINK, Kind::Exit) => compiler.tail_pop(),
        (Name::LINK | Name::IMAGE, Kind::Exit) => {
            let media = compiler.media.pop().unwrap_or_default();
            if media.in_reference {
                into_reference(compiler, media);
            }
            compiler.tail_pop();
        }
        _ => {}
    }
}

// A link or image whose label had no resource after it points through a definition
fn into_reference<'a>(compiler: &mut Compiler<'a, '_>, mut media: Media<'a>) {
    let reference_type = media.reference_type.unwrap_or(ReferenceType::Shortcut);
    // A shortcut or collapsed reference is named by its label text
    if media.label.is_none()
        && let Some(exit) = media.label_text
    {
        let raw = compiler.slice_at(exit);
        media.label = Some(compiler.alloc_str(&decode_string(raw)));
        media.identifier = compiler.alloc_str(&normalize_identifier(raw).to_lowercase());
    }
    let tail = compiler.tail_mut();
    let node = std::mem::replace(
        tail,
        Node::Text(textlane_core::mdast::Text {
            value: "",
            position: None,
        }),
    );
    *tail = match node {
        Node::Link(link) => Node::LinkReference(LinkReference {
            identifier: media.identifier,
            label: media.label,
            reference_type,
            children: link.children,
            position: link.position,
        }),
        Node::Image(image) => Node::ImageReference(ImageReference {
            identifier: media.identifier,
            label: media.label,
            reference_type,
            alt: image.alt,
            position: image.position,
        }),
        node => node,
    };
}
