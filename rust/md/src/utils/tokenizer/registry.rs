use crate::actions::attention::resolve::resolve_attention;
use crate::actions::data::resolve::resolve_data;
use crate::actions::heading_atx::resolve::resolve_heading_atx;
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
        Resolver::HeadingAtx => {
            resolve_heading_atx(tokenizer);
            None
        }
        Resolver::String => {
            resolve_whitespace(tokenizer, false, false);
            None
        }
        Resolver::Text => {
            resolve_whitespace(tokenizer, true, true);
            None
        }
    }
}
