use textlane_core::{Allocator, ArenaVec, Data};

/// The `data` mdast-util-math gives math (flow): a `pre` around a `code` holding `value`.
pub(crate) fn create_math_data<'a>(alloc: &'a Allocator, value: &'a str) -> &'a Data<'a> {
    let code = create_object(
        alloc,
        [
            ("type", Data::String("element")),
            ("tagName", Data::String("code")),
            ("properties", create_class_name(alloc, "math-display")),
            ("children", create_text_children(alloc, value)),
        ],
    );
    let mut children = ArenaVec::with_capacity_in(1, alloc);
    children.push(code);
    alloc.alloc(create_object(
        alloc,
        [
            ("hName", Data::String("pre")),
            ("hChildren", Data::Array(children)),
        ],
    ))
}

/// The `data` mdast-util-math gives math (text): a `code` holding `value`.
pub(crate) fn create_inline_math_data<'a>(
    alloc: &'a Allocator,
    value: &'a str,
) -> &'a Data<'a> {
    alloc.alloc(create_object(
        alloc,
        [
            ("hName", Data::String("code")),
            ("hProperties", create_class_name(alloc, "math-inline")),
            ("hChildren", create_text_children(alloc, value)),
        ],
    ))
}

fn create_class_name<'a>(alloc: &'a Allocator, kind: &'a str) -> Data<'a> {
    let mut names = ArenaVec::with_capacity_in(2, alloc);
    names.push(Data::String("language-math"));
    names.push(Data::String(kind));
    create_object(alloc, [("className", Data::Array(names))])
}

fn create_text_children<'a>(alloc: &'a Allocator, value: &'a str) -> Data<'a> {
    let text = create_object(
        alloc,
        [
            ("type", Data::String("text")),
            ("value", Data::String(value)),
        ],
    );
    let mut children = ArenaVec::with_capacity_in(1, alloc);
    children.push(text);
    Data::Array(children)
}

fn create_object<'a, const N: usize>(
    alloc: &'a Allocator,
    entries: [(&'a str, Data<'a>); N],
) -> Data<'a> {
    let mut object = ArenaVec::with_capacity_in(N, alloc);
    object.extend(entries);
    Data::Object(object)
}
