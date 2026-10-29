use std::cell::RefCell;

use serde_json::Value;
use textlane_core::mdast::{Node, Text, deserialize_mdast, serialize_mdast};
use textlane_core::{
    Allocator, Attach, Compiler, Error, File, Message, Parser, Plugin, Step, Transform,
    Tree, TreeKind,
};

use crate::actions::plugin::find::BuildPlugin;
use crate::models::pipeline::PipelineData;

thread_local! {
    static TRACE: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// Take the labels `test.trace:<label>` stages recorded on this thread, in run order.
pub(crate) fn take_trace() -> Vec<String> {
    TRACE.with(|trace| std::mem::take(&mut *trace.borrow_mut()))
}

/// The builder of a `test.*` plugin, the stand-ins for unified's inline test plugins.
pub(crate) fn find_test_plugin(name: &str) -> Option<BuildPlugin> {
    let known = matches!(
        name,
        "test.parseText"
            | "test.compileValue"
            | "test.noop"
            | "test.setData"
            | "test.recordOptions"
            | "test.replaceTree"
            | "test.appendNode"
            | "test.fail"
            | "test.message"
    ) || name.starts_with("test.trace:");
    known.then_some(|name, options| {
        Ok(Box::new(TestPlugin {
            name: name.to_string(),
            options,
        }))
    })
}

#[derive(Clone)]
struct TestPlugin {
    name: String,
    options: Value,
}

impl Plugin for TestPlugin {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        match self.name.as_str() {
            "test.parseText" => context.set_parser(|_| Ok(Box::new(TextParser))),
            "test.compileValue" => context.set_compiler(|_| Ok(Box::new(ValueCompiler))),
            "test.setData" => {
                let key = self.options["key"].as_str().unwrap_or_default().to_string();
                let data = &mut context.slots_mut().get_or_default::<PipelineData>().0;
                data.insert(key, self.options["value"].clone());
            }
            "test.recordOptions" => {
                let data = &mut context.slots_mut().get_or_default::<PipelineData>().0;
                let recorded = data
                    .entry("recorded")
                    .or_insert_with(|| Value::Array(vec![]));
                if let Value::Array(recorded) = recorded {
                    recorded.push(self.options.clone());
                }
            }
            _ => context.add_transform(Box::new(self.clone())),
        }
        Ok(())
    }
}

impl Transform for TestPlugin {
    fn input(&self) -> TreeKind {
        TreeKind::Mdast
    }

    fn output(&self) -> TreeKind {
        TreeKind::Mdast
    }

    fn run<'a>(
        &self,
        tree: Tree<'a>,
        file: &mut File,
        alloc: &'a Allocator,
    ) -> Result<Tree<'a>, Error> {
        let option = |key: &str| self.options.get(key).cloned().unwrap_or(Value::Null);
        let text = |key: &str| option(key).as_str().map(str::to_string);
        match self.name.as_str() {
            "test.replaceTree" => {
                Ok(Tree::Mdast(deserialize_mdast(&option("tree"), alloc)?))
            }
            "test.appendNode" => {
                let node = deserialize_mdast(&option("node"), alloc)?;
                let mut tree = tree;
                if let Tree::Mdast(root) = &mut tree
                    && let Some(children) = root.children_mut()
                {
                    children.push(node);
                }
                Ok(tree)
            }
            "test.fail" => Err(Error::new(
                "plugin-failed",
                text("message").unwrap_or_default(),
            )),
            "test.message" => {
                file.messages.push(Message {
                    reason: text("reason").unwrap_or_default(),
                    source: text("source"),
                    rule: text("ruleId"),
                    place: None,
                });
                Ok(tree)
            }
            name => {
                if let Some(label) = name.strip_prefix("test.trace:") {
                    TRACE.with(|trace| trace.borrow_mut().push(label.to_string()));
                }
                Ok(tree)
            }
        }
    }

    // `mode: "async"` stands in for a JavaScript plugin returning a promise
    fn start<'a>(
        &self,
        tree: Tree<'a>,
        file: &mut File,
        alloc: &'a Allocator,
    ) -> Result<Step<'a>, Error> {
        if self.options.get("mode").and_then(Value::as_str) != Some("async") {
            return self.run(tree, file, alloc).map(Step::Done);
        }
        let this = self.clone();
        Ok(Step::Job(Box::new(move |file| this.run(tree, file, alloc))))
    }
}

struct TextParser;

impl Parser for TextParser {
    fn output(&self) -> TreeKind {
        TreeKind::Mdast
    }

    fn parse<'a>(
        &self,
        input: &'a str,
        _file: &mut File,
        _alloc: &'a Allocator,
    ) -> Result<Tree<'a>, Error> {
        Ok(Tree::Mdast(Node::Text(Text {
            value: input,
            position: None,
            data: None,
        })))
    }
}

struct ValueCompiler;

impl Compiler for ValueCompiler {
    fn input(&self) -> TreeKind {
        TreeKind::Mdast
    }

    fn extension(&self) -> &'static str {
        "txt"
    }

    fn compile(&self, tree: &Tree<'_>, _file: &mut File) -> Result<String, Error> {
        let Tree::Mdast(node) = tree else {
            return Ok(String::new());
        };
        let value = serialize_mdast(node, None)?;
        Ok(value["value"].as_str().unwrap_or_default().to_string())
    }
}
