use textlane_core::hast::{Element, PropertyValue};

use crate::helpers::value::{find_value, is_truthy, parse_int, write_value};
use crate::utils::converter::state::Converter;
use crate::utils::converter::store::{Converted, Kind};

// Browsers clamp spans to these; past them, upstream would add cells without end
const MAX_ROW_SPAN: usize = 65_534;
const MAX_COLUMN_SPAN: usize = 1000;

/// `<td>` and `<th>`: a table cell, with the rows and columns it spans.
pub(crate) fn convert_table_cell<'a, 'n>(
    state: &mut Converter<'a, 'n, '_>,
    element: &'n Element<'a>,
) -> Converted {
    let children = state.all(&element.children);
    let row_span = find_value(element, "rowSpan");
    let column_span = find_value(element, "colSpan");
    let spans = (is_truthy(row_span) || is_truthy(column_span)).then(|| {
        (
            read_span(row_span, MAX_ROW_SPAN),
            read_span(column_span, MAX_COLUMN_SPAN),
        )
    });
    Converted::One(
        state
            .store
            .add(Kind::TableCell(spans), children, element.position),
    )
}

// `Number.parseInt(String(span), 10) || 1`
fn read_span(value: Option<&PropertyValue<'_>>, max: usize) -> usize {
    let span = value
        .filter(|value| is_truthy(Some(value)))
        .and_then(|value| parse_int(&write_value(value)))
        .unwrap_or(1.0);
    if span < 1.0 {
        1
    } else {
        (span as usize).min(max)
    }
}
