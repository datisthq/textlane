use textlane_core::mdast::Node;
use textlane_core::{
    Allocator, ArenaVec, Attach, Data, Error, File, Plugin, Transform, Tree, TreeKind,
};

use crate::models::options::Gemoji;
use crate::plugins::gemoji::actions::text::replace::{
    Replaced, replace_emoji, replace_shortcodes,
};

impl Plugin for Gemoji {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(self.clone()));
        Ok(())
    }
}

impl Transform for Gemoji {
    fn input(&self) -> TreeKind {
        TreeKind::Mdast
    }

    fn output(&self) -> TreeKind {
        TreeKind::Mdast
    }

    fn run<'a>(
        &self,
        tree: Tree<'a>,
        _file: &mut File,
        alloc: &'a Allocator,
    ) -> Result<Tree<'a>, Error> {
        let Tree::Mdast(mut root) = tree else {
            return Err(Error::new("invalid-stage", "`md.gemoji` takes mdast"));
        };
        let plain = !self.emoticon && !self.accessible && !self.pad_space_after;
        // Like remark-gemoji, text that is the whole tree has no parent to be replaced in
        let mut stack = root.children_mut().into_iter().collect::<Vec<_>>();
        while let Some(children) = stack.pop() {
            for child in children.iter_mut() {
                let Node::Text(text) = child else {
                    if let Some(children) = child.children_mut() {
                        stack.push(children);
                    }
                    continue;
                };
                let replaced = match plain {
                    true => replace_shortcodes(text.value)
                        .map(|value| Replaced { value, lead: None }),
                    false => replace_emoji(text.value, self),
                };
                if let Some(Replaced { value, lead }) = replaced {
                    text.value = alloc.alloc_str(&value);
                    // upstream replaces the node with a new one, which has no position and
                    // only the `data` of a `<span>` that opens it
                    text.position = None;
                    text.data = lead.and_then(|lead| {
                        let label = lead.label?;
                        Some(&*alloc.alloc(span_data(
                            alloc.alloc_str(&lead.text),
                            &label,
                            alloc,
                        )))
                    });
                }
            }
        }
        Ok(Tree::Mdast(root))
    }
}

fn span_data<'a>(value: &'a str, label: &str, alloc: &'a Allocator) -> Data<'a> {
    let field = |name: &'a str, value: Data<'a>| (name, value);
    let mut properties = ArenaVec::with_capacity_in(2, alloc);
    properties.push(field("role", Data::String("img")));
    properties.push(field("ariaLabel", Data::String(alloc.alloc_str(label))));
    let mut text = ArenaVec::with_capacity_in(2, alloc);
    text.push(field("type", Data::String("text")));
    text.push(field("value", Data::String(value)));
    let mut children = ArenaVec::with_capacity_in(1, alloc);
    children.push(Data::Object(text));
    let mut data = ArenaVec::with_capacity_in(3, alloc);
    data.push(field("hName", Data::String("span")));
    data.push(field("hProperties", Data::Object(properties)));
    data.push(field("hChildren", Data::Array(children)));
    Data::Object(data)
}
