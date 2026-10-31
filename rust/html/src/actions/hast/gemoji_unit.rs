use textlane_core::{Allocator, Error, File};

use super::replace_shortcodes;
use crate::actions::hast::stringify::stringify_hast;
use crate::actions::html::parse::parse_html;
use crate::models::options::{Gemoji, Parse, Stringify};

fn replace(input: &str, options: &Gemoji) -> Result<String, Error> {
    let alloc = Allocator::new();
    let parse = Parse {
        fragment: true,
        ..Default::default()
    };
    let mut node = parse_html(input, &parse, &mut File::new(), &alloc);
    replace_shortcodes(&mut node, options, &alloc);
    stringify_hast(&node, &Stringify::default())
}

#[test]
fn replaces_known_shortcodes_only() -> Result<(), Error> {
    assert_eq!(
        replace(
            "<p>Go :rocket: at 10:30:00 :nope:</p><code>:rocket:</code>",
            &Gemoji::default()
        )?,
        "<p>Go \u{1F680} at 10:30:00 :nope:</p><code>:rocket:</code>"
    );
    Ok(())
}

#[test]
fn labels_each_emoji_when_accessible() -> Result<(), Error> {
    let options = Gemoji {
        accessible: true,
        ..Gemoji::default()
    };
    assert_eq!(
        replace("<p>a :+1: :white_check_mark:</p>", &options)?,
        "<p>a <span role=\"img\" aria-label=\"+1 emoji\">\u{1F44D}</span> \
         <span role=\"img\" aria-label=\"white check mark emoji\">\u{2705}</span></p>"
    );
    Ok(())
}
