use textlane_core::Error;

use crate::actions::program::wrap::specifiers_to_declarations;
use crate::helpers::identifier::is_identifier_name;
use crate::helpers::walk::{Enter, Visit, walk};
use crate::models::estree::{Id, Store, Value};
use crate::models::name::{Key, Kind};
use crate::models::options::Compile;

/// estree-util-build-jsx 3.0.1 as recma-build-jsx runs it: JSX into calls of the runtime
/// the program's `@jsx…` pragma comments pick, `_jsx`/`_jsxs` imported for the automatic
/// one; then recma-build-jsx-transform drops those comments and, as a function body, takes
/// the runtime from `arguments[0]`.
pub(crate) fn build_program(
    store: &mut Store,
    program: Id,
    options: &Compile,
) -> Result<(), Error> {
    let mut build = Build {
        automatic: false,
        annotations: Annotations::default(),
        imports: Imports::default(),
        pragma: options.pragma.clone().filter(|pragma| !pragma.is_empty()),
        pragma_frag: options.pragma_frag.clone().filter(|frag| !frag.is_empty()),
        error: None,
    };
    walk(store, program, &mut build);
    if let Some(error) = build.error {
        return Err(error);
    }
    let comments: Vec<Value> = store
        .list(program, Key::COMMENTS)
        .iter()
        .filter(|comment| {
            !comment
                .as_node()
                .is_some_and(|id| store.flag(id, Key::PRAGMA))
        })
        .cloned()
        .collect();
    if store.has(program, Key::COMMENTS) {
        store.set(program, Key::COMMENTS, Value::List(comments));
    }
    if options.output_format.as_deref() == Some("function-body") {
        let body = store.children(program, Key::BODY);
        let index = body
            .iter()
            .position(|statement| !store.has(*statement, Key::DIRECTIVE))
            .unwrap_or(body.len());
        if let Some(declaration) = body.get(index).copied()
            && store.is(declaration, Kind::IMPORT_DECLARATION)
            && store
                .child(declaration, Key::SOURCE)
                .and_then(|source| store.str(source, Key::VALUE))
                .is_some_and(|source| {
                    source.ends_with("/jsx-runtime")
                        || source.ends_with("/jsx-dev-runtime")
                })
        {
            let arguments = store.identifier("arguments");
            let zero = store.literal(Value::Number(0.0));
            let init = store.member(arguments, zero, true);
            let specifiers = store.children(declaration, Key::SPECIFIERS);
            let declarations = specifiers_to_declarations(store, &specifiers, init);
            let constant = store.declaration("const", declarations);
            store.list_mut(program, Key::BODY)[index] = constant.into();
        }
    }
    Ok(())
}

#[derive(Default)]
struct Annotations {
    jsx: Option<String>,
    jsx_frag: Option<String>,
    jsx_import_source: Option<String>,
    jsx_runtime: Option<String>,
}

#[derive(Default)]
struct Imports {
    fragment: bool,
    jsx: bool,
    jsxs: bool,
}

struct Build {
    automatic: bool,
    annotations: Annotations,
    imports: Imports,
    pragma: Option<String>,
    pragma_frag: Option<String>,
    error: Option<Error>,
}

fn failure(message: &str) -> Error {
    Error::new("mdx-failed", message)
}

impl Visit for Build {
    fn enter(&mut self, store: &mut Store, id: Id) -> Enter {
        if store.is(id, Kind::PROGRAM) {
            for comment in store.children(id, Key::COMMENTS) {
                let value = store.str(comment, Key::VALUE).unwrap_or_default();
                read_annotations(value, &mut self.annotations);
            }
            if let Some(runtime) = self.annotations.jsx_runtime.as_deref() {
                let error = match runtime {
                    "automatic" => {
                        self.automatic = true;
                        if self.annotations.jsx.is_some() {
                            Some("Unexpected `@jsx` pragma w/ automatic runtime")
                        } else if self.annotations.jsx_frag.is_some() {
                            Some("Unexpected `@jsxFrag` pragma w/ automatic runtime")
                        } else {
                            None
                        }
                    }
                    "classic" => {
                        self.automatic = false;
                        self.annotations
                            .jsx_import_source
                            .is_some()
                            .then_some("Unexpected `@jsxImportSource` w/ classic runtime")
                    }
                    _ => {
                        self.error = Some(failure(&format!(
                            "Unexpected `jsxRuntime` `{runtime}`, expected `automatic` or `classic`"
                        )));
                        None
                    }
                };
                if let Some(error) = error {
                    self.error = Some(failure(error));
                }
            }
        }
        Enter::default()
    }

    fn leave(&mut self, store: &mut Store, id: Id) -> Option<Id> {
        if store.is(id, Kind::PROGRAM) {
            self.inject_imports(store, id);
            return None;
        }
        if self.error.is_some()
            || !matches!(store.kind(id), Kind::JSX_ELEMENT | Kind::JSX_FRAGMENT)
        {
            return None;
        }
        match self.build(store, id) {
            Ok(call) => Some(call),
            Err(error) => {
                self.error = Some(error);
                None
            }
        }
    }
}

impl Build {
    fn inject_imports(&self, store: &mut Store, program: Id) {
        let mut specifiers = Vec::new();
        for (enabled, imported, local) in [
            (self.imports.fragment, "Fragment", "_Fragment"),
            (self.imports.jsx, "jsx", "_jsx"),
            (self.imports.jsxs, "jsxs", "_jsxs"),
        ] {
            if enabled {
                let imported = store.identifier(imported);
                let local = store.identifier(local);
                specifiers.push(store.add(
                    Kind::IMPORT_SPECIFIER,
                    [(Key::IMPORTED, imported.into()), (Key::LOCAL, local.into())],
                ));
            }
        }
        if specifiers.is_empty() {
            return;
        }
        let source = format!(
            "{}/jsx-runtime",
            self.annotations
                .jsx_import_source
                .as_deref()
                .unwrap_or("react")
        );
        let source = store.literal(source.as_str());
        let declaration = store.add(
            Kind::IMPORT_DECLARATION,
            [
                (Key::SPECIFIERS, Value::nodes(specifiers)),
                (Key::SOURCE, source.into()),
            ],
        );
        let body = store.children(program, Key::BODY);
        let index = body
            .iter()
            .position(|statement| !store.has(*statement, Key::DIRECTIVE))
            .unwrap_or(body.len());
        store
            .list_mut(program, Key::BODY)
            .insert(index, declaration.into());
    }

    fn build(&mut self, store: &mut Store, node: Id) -> Result<Id, Error> {
        let mut children = Vec::new();
        for child in store.children(node, Key::CHILDREN) {
            match store.kind(child) {
                Kind::JSX_EXPRESSION_CONTAINER => {
                    if let Some(expression) = store.child(child, Key::EXPRESSION)
                        && !store.is(expression, Kind::JSX_EMPTY_EXPRESSION)
                    {
                        children.push(expression);
                    }
                }
                Kind::JSX_TEXT => {
                    let value =
                        collapse_text(store.str(child, Key::VALUE).unwrap_or_default());
                    if !value.is_empty() {
                        let text = store.literal(value.as_str());
                        create(store, child, text);
                        children.push(text);
                    }
                }
                _ => children.push(child),
            }
        }
        let mut fields = Vec::new();
        let mut key = None;
        let name;
        if store.is(node, Kind::JSX_ELEMENT) {
            let opening = store.child(node, Key::OPENING_ELEMENT);
            let element_name =
                opening.and_then(|opening| store.child(opening, Key::NAME));
            let mut identifier = match element_name {
                Some(element_name) => to_identifier(store, element_name),
                None => store.identifier("undefined"),
            };
            if store.is(identifier, Kind::IDENTIFIER)
                && store.str(identifier, Key::NAME).is_some_and(|name| {
                    name.starts_with(|c: char| c.is_ascii_lowercase())
                })
            {
                let text = store
                    .str(identifier, Key::NAME)
                    .unwrap_or_default()
                    .to_string();
                let next = store.literal(text.as_str());
                create(store, identifier, next);
                identifier = next;
            }
            name = identifier;
            let mut spread = false;
            let attributes =
                opening.map(|opening| store.children(opening, Key::ATTRIBUTES));
            for attribute in attributes.unwrap_or_default() {
                if store.is(attribute, Kind::JSX_SPREAD_ATTRIBUTE) {
                    if let Some(argument) = store.child(attribute, Key::ARGUMENT) {
                        if store.is(argument, Kind::OBJECT_EXPRESSION) {
                            fields.extend(store.children(argument, Key::PROPERTIES));
                        } else {
                            fields.push(store.spread(argument));
                        }
                    }
                    spread = true;
                    continue;
                }
                let property = to_property(store, attribute);
                let is_key = store.child(property, Key::KEY).is_some_and(|key| {
                    store.is(key, Kind::IDENTIFIER)
                        && store.str(key, Key::NAME) == Some("key")
                });
                if self.automatic && is_key {
                    if spread {
                        return Err(failure(
                            "Expected `key` to come before any spread expressions",
                        ));
                    }
                    key = store.child(property, Key::VALUE);
                } else {
                    fields.push(property);
                }
            }
        } else if self.automatic {
            self.imports.fragment = true;
            name = store.identifier("_Fragment");
        } else {
            let fragment = self
                .annotations
                .jsx_frag
                .clone()
                .or_else(|| self.pragma_frag.clone())
                .unwrap_or_else(|| "React.Fragment".to_string());
            name = to_member_expression(store, &fragment);
        }
        let mut parameters;
        let callee;
        if self.automatic {
            parameters = Vec::new();
            if !children.is_empty() {
                let value = if children.len() > 1 {
                    store.add(
                        Kind::ARRAY_EXPRESSION,
                        [(Key::ELEMENTS, Value::nodes(children.clone()))],
                    )
                } else {
                    children[0]
                };
                let key = store.identifier("children");
                fields.push(store.property(key, value, false));
            }
            parameters.push(store.object(fields));
            if let Some(key) = key {
                parameters.push(key);
            }
            if children.len() > 1 {
                self.imports.jsxs = true;
                callee = store.identifier("_jsxs");
            } else {
                self.imports.jsx = true;
                callee = store.identifier("_jsx");
            }
        } else {
            parameters = children;
            if !fields.is_empty() {
                parameters.insert(0, store.object(fields));
            } else if !parameters.is_empty() {
                parameters.insert(0, store.literal(Value::Null));
            }
            let pragma = self
                .annotations
                .jsx
                .clone()
                .or_else(|| self.pragma.clone())
                .unwrap_or_else(|| "React.createElement".to_string());
            callee = to_member_expression(store, &pragma);
        }
        parameters.insert(0, name);
        let call = store.call(callee, parameters);
        create(store, node, call);
        Ok(call)
    }
}

// The annotations regex, `/@(jsx|jsxFrag|jsxImportSource|jsxRuntime)\s+(\S+)/g`
fn read_annotations(value: &str, annotations: &mut Annotations) {
    let mut rest = value;
    while let Some(at) = rest.find('@') {
        let after = &rest[at + 1..];
        let found = ["jsx", "jsxFrag", "jsxImportSource", "jsxRuntime"]
            .into_iter()
            .find_map(|name| {
                let tail = after.strip_prefix(name)?;
                let trimmed = tail.trim_start();
                if trimmed.len() == tail.len() {
                    return None;
                }
                let end = trimmed.find(char::is_whitespace).unwrap_or(trimmed.len());
                (end > 0).then(|| (name, &trimmed[..end], &trimmed[end..]))
            });
        match found {
            Some((name, word, tail)) => {
                let word = Some(word.to_string());
                match name {
                    "jsx" => annotations.jsx = word,
                    "jsxFrag" => annotations.jsx_frag = word,
                    "jsxImportSource" => annotations.jsx_import_source = word,
                    _ => annotations.jsx_runtime = word,
                }
                rest = tail;
            }
            None => rest = after,
        }
    }
}

// JSX text: tabs become spaces, white space around line endings goes, and the lines join
// with a space
fn collapse_text(value: &str) -> String {
    let value = value.replace('\t', " ");
    let mut lines: Vec<&str> = Vec::new();
    let mut rest = value.as_str();
    loop {
        match rest.find(['\r', '\n']) {
            Some(index) => {
                lines.push(&rest[..index]);
                let skip = if rest[index..].starts_with("\r\n") {
                    2
                } else {
                    1
                };
                rest = &rest[index + skip..];
            }
            None => {
                lines.push(rest);
                break;
            }
        }
    }
    let count = lines.len();
    let trimmed: Vec<&str> = lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            let mut line = *line;
            if index > 0 {
                line = line.trim_start_matches(' ');
            }
            if index + 1 < count {
                line = line.trim_end_matches(' ');
            }
            line
        })
        .collect();
    let joined = trimmed.join("\n");
    let mut collapsed = String::with_capacity(joined.len());
    for character in joined.chars() {
        if character == '\n' && collapsed.ends_with('\n') {
            continue;
        }
        collapsed.push(character);
    }
    collapsed.trim_matches('\n').replace('\n', " ")
}

// `create`: copy where a node sits, and its comments, onto its replacement
fn create(store: &mut Store, from: Id, to: Id) {
    store.copy_place(from, to);
    if let Some(comments) = store.get(from, Key::COMMENTS).cloned() {
        store.set(to, Key::COMMENTS, comments);
    }
}

fn to_property(store: &mut Store, attribute: Id) -> Id {
    let value = match store.child(attribute, Key::VALUE) {
        Some(value) if store.is(value, Kind::JSX_EXPRESSION_CONTAINER) => {
            store.child(value, Key::EXPRESSION).unwrap_or(value)
        }
        Some(value) => {
            store.remove(value, Key::RAW);
            value
        }
        None => store.literal(true),
    };
    let key = match store.child(attribute, Key::NAME) {
        Some(name) => to_identifier(store, name),
        None => store.identifier("undefined"),
    };
    let property = store.property(key, value, false);
    create(store, attribute, property);
    property
}

// `toIdentifier`; member names nest as deep as the source writes them, so it loops
fn to_identifier(store: &mut Store, node: Id) -> Id {
    let mut parts = Vec::new();
    let mut current = node;
    while store.is(current, Kind::JSX_MEMBER_EXPRESSION) {
        parts.push(current);
        match store.child(current, Key::OBJECT) {
            Some(object) => current = object,
            None => break,
        }
    }
    let mut result = simple_identifier(store, current);
    for member in parts.into_iter().rev() {
        let property = match store.child(member, Key::PROPERTY) {
            Some(property) => simple_identifier(store, property),
            None => store.identifier("undefined"),
        };
        let computed = store.is(property, Kind::LITERAL);
        let expression = store.member(result, property, computed);
        create(store, member, expression);
        result = expression;
    }
    result
}

fn simple_identifier(store: &mut Store, node: Id) -> Id {
    let replace = if store.is(node, Kind::JSX_NAMESPACED_NAME) {
        let namespace = store
            .child(node, Key::NAMESPACE)
            .and_then(|namespace| store.str(namespace, Key::NAME))
            .unwrap_or_default();
        let name = store
            .child(node, Key::NAME)
            .and_then(|name| store.str(name, Key::NAME))
            .unwrap_or_default();
        let value = format!("{namespace}:{name}");
        store.literal(value.as_str())
    } else {
        let name = store.str(node, Key::NAME).unwrap_or_default().to_string();
        if is_identifier_name(&name, false) {
            store.identifier(&name)
        } else {
            store.literal(name.as_str())
        }
    };
    create(store, node, replace);
    replace
}

fn to_member_expression(store: &mut Store, id: &str) -> Id {
    let mut result: Option<Id> = None;
    for (index, part) in id.split('.').enumerate() {
        let property = if is_identifier_name(part, false) {
            store.identifier(part)
        } else {
            store.literal(part)
        };
        result = Some(match result {
            Some(object) => {
                let computed = index > 0 && store.is(property, Kind::LITERAL);
                store.member(object, property, computed)
            }
            None => property,
        });
    }
    result.unwrap_or_else(|| store.identifier("undefined"))
}
