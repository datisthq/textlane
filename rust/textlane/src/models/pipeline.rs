use std::sync::Arc;

use serde_json::{Map, Value};
use textlane_core::{Allocator, Error, File, Lines, Tree, TreeKind};

use crate::actions::allocator::reuse::with_allocator;
use crate::actions::plugin::find::find_plugin;
use crate::models::host::Host;
use crate::models::processor::{FrozenProcessor, Processor};
use crate::models::run::{Progress, Run};

/// Whether a call may finish asynchronous stages, as unified's `process` and `run` do, or
/// must fail on them, as `processSync` and `runSync` do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Api {
    Sync,
    Async,
}

/// A processor built from plugin names and JSON options, with unified's `use`, `data` and
/// `freeze` semantics: what the bindings drive.
#[derive(Default)]
pub struct Pipeline {
    // Each plugin once, in first-use order, with its latest options; `false` turns it off
    entries: Vec<(Key, Option<Value>)>,
    data: Map<String, Value>,
    frozen: Option<Arc<FrozenProcessor>>,
}

// A host plugin is known by the id its binding gave it; its options stay in the binding
#[derive(Clone, Debug, PartialEq, Eq)]
enum Key {
    Name(String),
    Host(u32),
}

/// The data plugins set while they attach, shared through the attach slots.
#[derive(Default)]
pub(crate) struct PipelineData(pub(crate) Map<String, Value>);

const FROZEN_HINT: &str = "Create a new processor first, by calling it: use `processor()` instead of `processor`.";

impl Pipeline {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a plugin by name or descriptor (`{name, options?}`, or `{host}` for a binding's
    /// function), a list of plugins, or a preset
    /// (`{plugins, settings}`); a plugin used again is reconfigured, its object options merged
    /// into the earlier ones.
    pub fn use_value(
        &mut self,
        value: &Value,
        options: Option<Value>,
    ) -> Result<(), Error> {
        self.check_unfrozen("use")?;
        match value {
            Value::Null => Ok(()),
            Value::String(name) => self.add_entry(Key::Name(name.clone()), options),
            Value::Array(list) => self.add_list(list),
            Value::Object(object) => match read_descriptor(object) {
                Some((key, own)) => self.add_entry(key, options.or(own)),
                None => self.add_preset(object),
            },
            other => Err(Error::new(
                "invalid-plugin",
                format!("Expected usable value, not `{other}`"),
            )),
        }
    }

    /// Set one data value; fails once the pipeline is frozen.
    pub fn set_data(&mut self, key: &str, value: Value) -> Result<(), Error> {
        self.check_unfrozen("data")?;
        self.data.insert(key.to_string(), value);
        Ok(())
    }

    /// Replace all data with an object, as unified's `data(dataset)`; fails once the
    /// pipeline is frozen.
    pub fn replace_data(&mut self, data: Value) -> Result<(), Error> {
        self.check_unfrozen("data")?;
        let Value::Object(data) = data else {
            return Err(Error::new(
                "invalid-data",
                format!("Expected an object as data, not `{data}`"),
            ));
        };
        self.data = data;
        Ok(())
    }

    /// The data plugins and callers set: all of it, or the value under `key`.
    pub fn data(&self, key: Option<&str>) -> Value {
        match key {
            Some(key) => self.data.get(key).cloned().unwrap_or(Value::Null),
            None => Value::Object(self.data.clone()),
        }
    }

    /// The ids of the enabled host plugins, in the order their stages run.
    pub fn hosts(&self) -> Vec<u32> {
        let enabled = self.entries.iter().filter_map(|(key, options)| match key {
            Key::Host(id) if options != &Some(Value::Bool(false)) => Some(*id),
            _ => None,
        });
        enabled.collect()
    }

    /// The latest options a host plugin was used with, merged as a named plugin's are.
    #[cfg(feature = "testing")]
    pub(crate) fn host_options(&self, id: u32) -> Option<Value> {
        let entry = self.entries.iter().find(|(key, _)| key == &Key::Host(id));
        entry.and_then(|(_, options)| options.clone())
    }

    /// Attach every enabled plugin once; later calls return the same processor.
    pub fn freeze(&mut self) -> Result<&FrozenProcessor, Error> {
        self.freeze_shared().map(|_| ())?;
        match &self.frozen {
            Some(frozen) => Ok(frozen),
            None => unreachable!("the pipeline froze above"),
        }
    }

    fn freeze_shared(&mut self) -> Result<Arc<FrozenProcessor>, Error> {
        if self.frozen.is_none() {
            let mut processor = Processor::new();
            for (key, options) in &self.entries {
                let options = match options {
                    Some(Value::Bool(false)) => continue,
                    Some(Value::Bool(true)) | None => Value::Null,
                    Some(options) => options.clone(),
                };
                let name = match key {
                    Key::Name(name) => name,
                    Key::Host(id) => {
                        processor = processor.with_boxed(Box::new(Host { id: *id }));
                        continue;
                    }
                };
                let build = find_plugin(name).ok_or_else(|| unknown_plugin(name))?;
                processor = processor.with_boxed(build(name, options)?);
            }
            let seed = std::mem::take(&mut self.data);
            let mut attached = Map::new();
            let frozen = processor.freeze_with(
                |context| context.slots_mut().get_or_default::<PipelineData>().0 = seed,
                |context| {
                    attached = std::mem::take(
                        &mut context.slots_mut().get_or_default::<PipelineData>().0,
                    );
                },
            )?;
            self.data = attached;
            self.frozen = Some(Arc::new(frozen));
        }
        match &self.frozen {
            Some(frozen) => Ok(Arc::clone(frozen)),
            None => unreachable!("the pipeline froze above"),
        }
    }

    /// Parse a document into a tree, as JSON.
    pub fn parse(&mut self, input: &str, file: &mut File) -> Result<Value, Error> {
        let processor = self.freeze()?;
        check_parser(processor, "parse")?;
        let alloc = Allocator::new();
        let tree = processor.parse(input, file, &alloc)?;
        write_tree(&tree, Some(&Lines::new(processor.source(input))))
    }

    /// Run the transforms over a tree given as JSON. A `file` holding the document's text
    /// (`File::value`) lets bridges read it and places the result's nodes in it.
    pub fn run(
        &mut self,
        tree: &Value,
        file: &mut File,
        api: Api,
    ) -> Result<Value, Error> {
        let processor = self.freeze()?;
        if let Some(value) = &file.value {
            file.value = Some(processor.source(value).to_string());
        }
        let alloc = Allocator::new();
        let tree = read_tree(tree, processor.input_kind(), &alloc)?;
        let tree = run_tree(processor, tree, file, &alloc, api, "run")?;
        let lines = file.value.as_deref().map(Lines::new);
        write_tree(&tree, lines.as_ref())
    }

    /// Compile a tree given as JSON, running no transform.
    pub fn stringify(&mut self, tree: &Value, file: &mut File) -> Result<String, Error> {
        let processor = self.freeze()?;
        check_compiler(processor, "stringify")?;
        let alloc = Allocator::new();
        let tree = read_tree(tree, processor.compiler_kind(), &alloc)?;
        processor.stringify(&tree, file)
    }

    /// Parse, run and compile a document.
    pub fn process(
        &mut self,
        input: &str,
        file: &mut File,
        api: Api,
    ) -> Result<String, Error> {
        let processor = self.freeze()?;
        let call = match api {
            Api::Sync => "processSync",
            Api::Async => "process",
        };
        check_parser(processor, call)?;
        check_compiler(processor, call)?;
        with_allocator(|alloc| {
            let tree = processor.parse(input, file, alloc)?;
            let tree = run_tree(processor, tree, file, alloc, api, "process")?;
            processor.stringify(&tree, file)
        })
    }

    /// Run the transforms over a tree given as JSON, as `run` does, pausing before the
    /// stages of host plugins for the binding to run them.
    pub fn start_run(
        &mut self,
        tree: &Value,
        mut file: File,
        api: Api,
    ) -> Result<Progress, Error> {
        let processor = self.freeze_shared()?;
        if let Some(value) = &file.value {
            file.value = Some(processor.source(value).to_string());
        }
        let run = Run {
            processor: &processor,
            api,
            compile: false,
        };
        let alloc = Allocator::new();
        let tree = read_tree(tree, processor.input_kind(), &alloc)?;
        let step = run.advance(tree, &mut file, &alloc, 0)?;
        Ok(run.progress(step, file))
    }

    /// Parse, run and compile a document, as `process` does, pausing before the stages of
    /// host plugins for the binding to run them.
    pub fn start_process(
        &mut self,
        input: &str,
        mut file: File,
        api: Api,
    ) -> Result<Progress, Error> {
        let processor = self.freeze_shared()?;
        let call = match api {
            Api::Sync => "processSync",
            Api::Async => "process",
        };
        check_parser(&processor, call)?;
        check_compiler(&processor, call)?;
        let run = Run {
            processor: &processor,
            api,
            compile: true,
        };
        let step = with_allocator(|alloc| {
            let tree = processor.parse(input, &mut file, alloc)?;
            run.advance(tree, &mut file, alloc, 0)
        })?;
        Ok(run.progress(step, file))
    }

    fn check_unfrozen(&self, call: &str) -> Result<(), Error> {
        match self.frozen {
            Some(_) => Err(Error::new(
                "frozen-processor",
                format!("Cannot call `{call}` on a frozen processor.\n{FROZEN_HINT}"),
            )),
            None => Ok(()),
        }
    }

    fn add_entry(&mut self, key: Key, options: Option<Value>) -> Result<(), Error> {
        if let Key::Name(name) = &key {
            find_plugin(name).ok_or_else(|| unknown_plugin(name))?;
        }
        let Some(index) = self.entries.iter().position(|(known, _)| known == &key) else {
            self.entries.push((key, options));
            return Ok(());
        };
        let Some(mut options) = options else {
            return Ok(());
        };
        if let (Some(current @ Value::Object(_)), Value::Object(_)) =
            (&self.entries[index].1, &options)
        {
            let mut merged = current.clone();
            merge_value(&mut merged, &options);
            options = merged;
        }
        self.entries[index].1 = Some(options);
        Ok(())
    }

    fn add_list(&mut self, list: &[Value]) -> Result<(), Error> {
        for entry in list {
            match entry {
                Value::String(name) => self.add_entry(Key::Name(name.clone()), None)?,
                Value::Array(tuple) => match tuple.split_first() {
                    Some((Value::String(name), rest)) => {
                        self.add_entry(Key::Name(name.clone()), rest.first().cloned())?;
                    }
                    Some((Value::Object(object), rest)) => {
                        match read_descriptor(object) {
                            Some((key, own)) => {
                                self.add_entry(key, rest.first().cloned().or(own))?;
                            }
                            None => return Err(invalid_entry(entry)),
                        }
                    }
                    _ => return Err(invalid_entry(entry)),
                },
                Value::Object(object) => match read_descriptor(object) {
                    Some((key, own)) => self.add_entry(key, own)?,
                    None => self.add_preset(object)?,
                },
                _ => return Err(invalid_entry(entry)),
            }
        }
        Ok(())
    }

    fn add_preset(&mut self, preset: &Map<String, Value>) -> Result<(), Error> {
        let plugins = preset.get("plugins");
        let settings = preset.get("settings");
        if plugins.is_none() && settings.is_none() {
            return Err(Error::new(
                "invalid-plugin",
                "Expected usable value but received an empty preset, which is probably a mistake: presets typically come with `plugins` and sometimes with `settings`, but this has neither",
            ));
        }
        match plugins {
            Some(Value::Array(list)) => self.add_list(list)?,
            Some(Value::Null) | None => {}
            Some(other) => return Err(invalid_entry(other)),
        }
        if let Some(settings) = settings {
            let current = self
                .data
                .entry("settings")
                .or_insert_with(|| Value::Object(Map::new()));
            merge_value(current, settings);
        }
        Ok(())
    }
}

fn run_tree<'a>(
    processor: &FrozenProcessor,
    tree: Tree<'a>,
    file: &mut File,
    alloc: &'a Allocator,
    api: Api,
    call: &str,
) -> Result<Tree<'a>, Error> {
    match api {
        Api::Async => processor.run(tree, file, alloc),
        Api::Sync => processor.run_sync(tree, file, alloc).map_err(|error| {
            if error.code != "async-in-sync" {
                return error;
            }
            let message = format!("`{call}Sync` finished async. Use `{call}` instead");
            Error::new(error.code, message)
        }),
    }
}

fn check_parser(processor: &FrozenProcessor, call: &str) -> Result<(), Error> {
    match processor.has_parser() {
        true => Ok(()),
        false => Err(Error::new(
            "missing-parser",
            format!("Cannot `{call}` without `parser`"),
        )),
    }
}

fn check_compiler(processor: &FrozenProcessor, call: &str) -> Result<(), Error> {
    match processor.has_compiler() {
        true => Ok(()),
        false => Err(Error::new(
            "missing-compiler",
            format!("Cannot `{call}` without `compiler`"),
        )),
    }
}

// An absent `options` is None, not null: `use(md.gfm())` must keep earlier options
fn read_descriptor(object: &Map<String, Value>) -> Option<(Key, Option<Value>)> {
    let key = match (object.get("name"), object.get("host")) {
        (Some(Value::String(name)), _) => Key::Name(name.clone()),
        (None, Some(id)) => Key::Host(u32::try_from(id.as_u64()?).ok()?),
        _ => return None,
    };
    Some((key, object.get("options").cloned()))
}

fn unknown_plugin(name: &str) -> Error {
    Error::new("unknown-plugin", format!("no plugin named `{name}`"))
}

fn invalid_entry(entry: &Value) -> Error {
    Error::new(
        "invalid-plugin",
        format!("Expected usable value, not `{entry}`"),
    )
}

// extend(true, …) as unified merges options and settings: objects key by key, arrays index
// by index, anything else replaced
fn merge_value(target: &mut Value, source: &Value) {
    match (target, source) {
        (Value::Object(target), Value::Object(source)) => {
            for (key, value) in source {
                match target.get_mut(key) {
                    Some(current) if value.is_object() || value.is_array() => {
                        merge_value(current, value);
                    }
                    _ => {
                        target.insert(key.clone(), value.clone());
                    }
                }
            }
        }
        (Value::Array(target), Value::Array(source)) => {
            for (index, value) in source.iter().enumerate() {
                match target.get_mut(index) {
                    Some(current) => merge_value(current, value),
                    None => target.push(value.clone()),
                }
            }
        }
        (target, source) => *target = source.clone(),
    }
}

/// Read a JSON tree as the kind of tree its first stage takes.
pub(crate) fn read_tree<'a>(
    tree: &Value,
    kind: Option<TreeKind>,
    alloc: &'a Allocator,
) -> Result<Tree<'a>, Error> {
    match kind {
        Some(TreeKind::Hast) => Ok(Tree::Hast(textlane_core::hast::deserialize_hast(
            tree, alloc,
        )?)),
        Some(TreeKind::Nlcst) => Ok(Tree::Nlcst(
            textlane_core::nlcst::deserialize_nlcst(tree, alloc)?,
        )),
        _ => Ok(Tree::Mdast(textlane_core::mdast::deserialize_mdast(
            tree, alloc,
        )?)),
    }
}

/// Write a tree as JSON, with unist positions when the source's lines are known.
pub(crate) fn write_tree(
    tree: &Tree<'_>,
    lines: Option<&Lines<'_>>,
) -> Result<Value, Error> {
    match tree {
        Tree::Mdast(node) => textlane_core::mdast::serialize_mdast(node, lines),
        Tree::Hast(node) => textlane_core::hast::serialize_hast(node, lines),
        Tree::Nlcst(node) => textlane_core::nlcst::serialize_nlcst(node, lines),
    }
}
