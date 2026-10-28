use crate::actions::attention::resolve::resolve_attention;
use crate::actions::data::resolve::resolve_data;
use crate::actions::heading_atx::resolve::resolve_heading_atx;
use crate::actions::heading_setext::resolve::resolve_heading_setext;
use crate::actions::label_end::resolve::resolve_label;
use crate::actions::list_item::resolve::resolve_list_item;
use crate::actions::whitespace::resolve::resolve_whitespace;
use crate::models::construct::Resolver;
use crate::utils::tokenizer::content::resolve_content;
use crate::utils::tokenizer::driver::Tokenizer;
use crate::utils::tokenizer::subtokenize::Subresult;

/// Run one resolver over a tokenizer's events.
pub(crate) fn call_resolver(
    tokenizer: &mut Tokenizer,
    resolver: Resolver,
) -> Option<Subresult> {
    match resolver {
        Resolver::Attention => {
            resolve_attention(tokenizer);
            None
        }
        Resolver::Content => Some(resolve_content(tokenizer)),
        Resolver::Data => {
            resolve_data(tokenizer);
            None
        }
        Resolver::ListItem => {
            resolve_list_item(tokenizer);
            None
        }
        Resolver::HeadingSetext => {
            resolve_heading_setext(tokenizer);
            None
        }
        Resolver::Label => {
            resolve_label(tokenizer);
            None
        }
        Resolver::HeadingAtx => {
            resolve_heading_atx(tokenizer);
            None
        }
        // micromark trims whitespace around line endings in text only, never in strings
        Resolver::String => None,
        Resolver::Text => {
            resolve_whitespace(tokenizer, true, true);
            None
        }
    }
}
