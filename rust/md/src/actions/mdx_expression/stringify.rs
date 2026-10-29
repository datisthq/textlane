use textlane_core::mdast::MdxExpression;

use crate::utils::serializer::state::Serializer;

/// Write an expression, its lines after the first indented by two spaces, as
/// mdast-util-mdx-expression does.
pub(crate) fn stringify_mdx_expression(
    state: &mut Serializer,
    expression: &MdxExpression,
) {
    let mut value = String::from("{");
    let mut rest = expression.value;
    let mut first = true;
    loop {
        let end = rest.find(['\n', '\r']).unwrap_or(rest.len());
        let line = &rest[..end];
        if !first && !line.is_empty() {
            value.push_str("  ");
        }
        value.push_str(line);
        first = false;
        if end == rest.len() {
            break;
        }
        let ending = if rest[end..].starts_with("\r\n") {
            2
        } else {
            1
        };
        value.push_str(&rest[end..end + ending]);
        rest = &rest[end + ending..];
    }
    value.push('}');
    state.write(&value);
}
