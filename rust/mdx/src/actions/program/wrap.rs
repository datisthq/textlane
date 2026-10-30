use textlane_core::Error;

use crate::actions::hast::convert::Snippet;
use crate::helpers::place::stringify_place;
use crate::helpers::walk::{Enter, Visit, walk};
use crate::models::estree::{Id, Store, Value};
use crate::models::name::{Key, Kind};
use crate::models::options::Compile;
use crate::models::scope::Scopes;

/// recma-document of @mdx-js/mdx 3.1.1: inject the JSX pragmas, take the layout from
/// `export default`, and wrap the JSX in `_createMdxContent` and the `MDXContent` export;
/// as a function body, imports and exports become `await import()` and a final `return`.
pub(crate) fn wrap_program(
    store: &mut Store,
    program: Id,
    options: &Compile,
    snippets: &[Snippet],
) -> Result<(), Error> {
    let pragma = options.pragma.as_deref().unwrap_or("React.createElement");
    let pragma_frag = options.pragma_frag.as_deref().unwrap_or("React.Fragment");
    let pragma_import_source =
        non_empty(&options.pragma_import_source).unwrap_or("react");
    let jsx_import_source = non_empty(&options.jsx_import_source).unwrap_or("react");
    let jsx_runtime = non_empty(&options.jsx_runtime).unwrap_or("automatic");
    let function_body = options.output_format.as_deref() == Some("function-body");
    let mut wrap = Wrap {
        store,
        function_body,
        replacement: Vec::new(),
        exported: Vec::new(),
        export_all_count: 0,
    };
    if jsx_runtime == "classic" && !pragma_frag.is_empty() {
        wrap.inject_pragma(program, "@jsxFrag", pragma_frag);
    }
    if jsx_runtime == "classic" && !pragma.is_empty() {
        wrap.inject_pragma(program, "@jsx", pragma);
    }
    if jsx_runtime == "automatic" {
        wrap.inject_pragma(program, "@jsxImportSource", jsx_import_source);
    }
    wrap.inject_pragma(program, "@jsxRuntime", jsx_runtime);
    // `pragmaImportSource` defaults to `react`, so the classic runtime always imports it
    if jsx_runtime == "classic" {
        if pragma.is_empty() {
            return Err(Error::new(
                "mdx-failed",
                "Missing `pragma` in classic runtime with `pragmaImportSource`",
            ));
        }
        let name = pragma.split('.').next().unwrap_or_default();
        let local = wrap.store.identifier(name);
        let specifier = wrap
            .store
            .add(Kind::IMPORT_DEFAULT_SPECIFIER, [(Key::LOCAL, local.into())]);
        let source = wrap.store.literal(pragma_import_source);
        let import = wrap.store.add(
            Kind::IMPORT_DECLARATION,
            [
                (Key::SPECIFIERS, Value::nodes([specifier])),
                (Key::ATTRIBUTES, Value::List(Vec::new())),
                (Key::SOURCE, source.into()),
            ],
        );
        wrap.handle_esm(import);
    }
    let mut layout: Option<Id> = None;
    let mut content = false;
    for child in wrap.store.children(program, Key::BODY) {
        match wrap.store.kind(child) {
            Kind::EXPORT_DEFAULT_DECLARATION => {
                if let Some(previous) = layout {
                    return Err(duplicate_layout(wrap.store, snippets, previous));
                }
                layout = Some(child);
                let declaration = wrap.store.child(child, Key::DECLARATION);
                let init =
                    declaration.map(|declaration| match wrap.store.kind(declaration) {
                        Kind::FUNCTION_DECLARATION | Kind::CLASS_DECLARATION => {
                            let expression = wrap.store.copy(declaration);
                            let kind =
                                if wrap.store.is(declaration, Kind::FUNCTION_DECLARATION)
                                {
                                    Kind::FUNCTION_EXPRESSION
                                } else {
                                    Kind::CLASS_EXPRESSION
                                };
                            wrap.store.set_kind(expression, kind);
                            expression
                        }
                        _ => declaration,
                    });
                let id = wrap.store.identifier("MDXLayout");
                let declarator = wrap.store.declarator(id, init);
                let constant = wrap.store.declaration("const", vec![declarator]);
                wrap.replacement.push(constant);
            }
            Kind::EXPORT_NAMED_DECLARATION
                if wrap.store.child(child, Key::SOURCE).is_some() =>
            {
                let source = wrap.store.child(child, Key::SOURCE);
                let mut kept = Vec::new();
                for specifier in wrap.store.children(child, Key::SPECIFIERS) {
                    let exported = wrap.store.child(specifier, Key::EXPORTED);
                    let is_default = exported.is_some_and(|exported| {
                        wrap.store.is(exported, Kind::IDENTIFIER)
                            && wrap.store.str(exported, Key::NAME) == Some("default")
                    });
                    if !is_default {
                        kept.push(specifier);
                        continue;
                    }
                    if let Some(previous) = layout {
                        return Err(duplicate_layout(wrap.store, snippets, previous));
                    }
                    layout = Some(specifier);
                    let local = wrap.store.child(specifier, Key::LOCAL);
                    let mdx_layout = wrap.store.identifier("MDXLayout");
                    let import_specifier = if local.is_some_and(|local| {
                        wrap.store.is(local, Kind::IDENTIFIER)
                            && wrap.store.str(local, Key::NAME) == Some("default")
                    }) {
                        wrap.store.add(
                            Kind::IMPORT_DEFAULT_SPECIFIER,
                            [(Key::LOCAL, mdx_layout.into())],
                        )
                    } else {
                        let import_specifier = wrap.store.add(
                            Kind::IMPORT_SPECIFIER,
                            [
                                (Key::IMPORTED, local.into()),
                                (Key::LOCAL, mdx_layout.into()),
                            ],
                        );
                        if let Some(local) = local {
                            wrap.store.copy_place(local, import_specifier);
                        }
                        import_specifier
                    };
                    let value = source
                        .and_then(|source| wrap.store.get(source, Key::VALUE).cloned());
                    let from = wrap.store.literal(value.unwrap_or(Value::Null));
                    if let Some(source) = source {
                        wrap.store.copy_place(source, from);
                    }
                    let declaration = wrap.store.add(
                        Kind::IMPORT_DECLARATION,
                        [
                            (Key::SPECIFIERS, Value::nodes([import_specifier])),
                            (Key::ATTRIBUTES, Value::List(Vec::new())),
                            (Key::SOURCE, from.into()),
                        ],
                    );
                    wrap.store.copy_place(specifier, declaration);
                    wrap.handle_esm(declaration);
                }
                let remaining = !kept.is_empty();
                wrap.store.set(child, Key::SPECIFIERS, Value::nodes(kept));
                if remaining {
                    wrap.handle_export(child);
                }
            }
            Kind::EXPORT_NAMED_DECLARATION | Kind::EXPORT_ALL_DECLARATION => {
                wrap.handle_export(child)
            }
            Kind::IMPORT_DECLARATION => wrap.handle_esm(child),
            Kind::EXPRESSION_STATEMENT
                if wrap.store.child(child, Key::EXPRESSION).is_some_and(
                    |expression| {
                        matches!(
                            wrap.store.kind(expression),
                            Kind::JSX_ELEMENT | Kind::JSX_FRAGMENT
                        )
                    },
                ) =>
            {
                content = true;
                let expression = wrap.store.child(child, Key::EXPRESSION);
                let statements = wrap.create_mdx_content(expression, layout.is_some());
                wrap.replacement.extend(statements);
            }
            _ => wrap.replacement.push(child),
        }
    }
    if !content {
        let statements = wrap.create_mdx_content(None, layout.is_some());
        wrap.replacement.extend(statements);
    }
    wrap.exported
        .push(("MDXContent".to_string(), Some("default".to_string())));
    if function_body {
        let mut properties = Vec::new();
        for index in 1..=wrap.export_all_count {
            let argument = wrap.store.identifier(&format!("_exportAll{index}"));
            properties.push(wrap.store.spread(argument));
        }
        for (local, exported) in std::mem::take(&mut wrap.exported) {
            let key = wrap.store.identifier(exported.as_deref().unwrap_or(&local));
            let value = wrap.store.identifier(&local);
            properties.push(wrap.store.property(key, value, exported.is_none()));
        }
        let object = wrap.store.object(properties);
        let statement = wrap.store.return_statement(Some(object));
        wrap.replacement.push(statement);
    }
    let replacement = std::mem::take(&mut wrap.replacement);
    wrap.store
        .set(program, Key::BODY, Value::nodes(replacement));
    if function_body {
        resolve_dynamic(wrap.store, program);
    }
    Ok(())
}

struct Wrap<'s> {
    store: &'s mut Store,
    function_body: bool,
    replacement: Vec<Id>,
    // A name exported as itself, or a local and the name it is exported as
    exported: Vec<(String, Option<String>)>,
    export_all_count: usize,
}

fn non_empty(value: &Option<String>) -> Option<&str> {
    value.as_deref().filter(|value| !value.is_empty())
}

fn duplicate_layout(store: &Store, snippets: &[Snippet], previous: Id) -> Error {
    Error::new(
        "recma-document:duplicate-layout",
        format!(
            "Unexpected duplicate layout, expected a single layout (previous: {})",
            stringify_place(store, snippets, previous)
        ),
    )
}

impl Wrap<'_> {
    fn inject_pragma(&mut self, program: Id, name: &str, value: &str) {
        let comment = self.store.add(
            Kind::BLOCK,
            [(Key::VALUE, format!("{name} {value}").as_str().into())],
        );
        self.store.set(comment, Key::PRAGMA, true);
        self.store
            .list_mut(program, Key::COMMENTS)
            .insert(0, comment.into());
    }

    fn handle_export(&mut self, node: Id) {
        if self.store.is(node, Kind::EXPORT_NAMED_DECLARATION) {
            if self.store.child(node, Key::DECLARATION).is_some() {
                let mut visitor = DefineVisitor {
                    scopes: Scopes::default(),
                };
                walk(self.store, node, &mut visitor);
                let defined = std::mem::take(&mut visitor.scopes.scopes[0].defined);
                self.exported
                    .extend(defined.into_iter().map(|name| (name, None)));
            }
            for specifier in self.store.children(node, Key::SPECIFIERS) {
                if let Some(exported) = self.store.child(specifier, Key::EXPORTED) {
                    let name = match self.store.get(exported, Key::NAME) {
                        Some(Value::Str(name)) => name.to_string(),
                        _ => match self.store.get(exported, Key::VALUE) {
                            Some(Value::Str(value)) => value.to_string(),
                            _ => String::new(),
                        },
                    };
                    self.exported.push((name, None));
                }
            }
        }
        self.handle_esm(node);
    }

    fn handle_esm(&mut self, node: Id) {
        if !self.function_body {
            self.replacement.push(node);
            return;
        }
        let kind = self.store.kind(node);
        let source = self.store.child(node, Key::SOURCE);
        let replace = if kind == Kind::IMPORT_DECLARATION
            || kind == Kind::EXPORT_ALL_DECLARATION
            || (kind == Kind::EXPORT_NAMED_DECLARATION && source.is_some())
        {
            let argument = self
                .store
                .add(Kind::IMPORT_EXPRESSION, [(Key::SOURCE, source.into())]);
            self.store.copy_place(node, argument);
            let init = self
                .store
                .add(Kind::AWAIT_EXPRESSION, [(Key::ARGUMENT, argument.into())]);
            let specifiers = self.store.children(node, Key::SPECIFIERS);
            if kind != Kind::EXPORT_ALL_DECLARATION && specifiers.is_empty() {
                Some(self.store.expression_statement(init))
            } else {
                let declarations = if kind == Kind::EXPORT_ALL_DECLARATION {
                    self.export_all_count += 1;
                    let id = self
                        .store
                        .identifier(&format!("_exportAll{}", self.export_all_count));
                    vec![self.store.declarator(id, Some(init))]
                } else {
                    specifiers_to_declarations(self.store, &specifiers, init)
                };
                Some(self.store.declaration("const", declarations))
            }
        } else if let Some(declaration) = self.store.child(node, Key::DECLARATION) {
            Some(declaration)
        } else {
            let mut declarators = Vec::new();
            for specifier in self.store.children(node, Key::SPECIFIERS) {
                let (Some(exported), Some(local)) = (
                    self.store.child(specifier, Key::EXPORTED),
                    self.store.child(specifier, Key::LOCAL),
                ) else {
                    continue;
                };
                if self.store.is(exported, Kind::IDENTIFIER)
                    && self.store.is(local, Kind::IDENTIFIER)
                    && self.store.str(local, Key::NAME)
                        != self.store.str(exported, Key::NAME)
                {
                    declarators.push(self.store.declarator(exported, Some(local)));
                }
            }
            (!declarators.is_empty())
                .then(|| self.store.declaration("const", declarators))
        };
        if let Some(replace) = replace {
            self.replacement.push(replace);
        }
    }

    fn create_mdx_content(
        &mut self,
        content: Option<Id>,
        has_internal_layout: bool,
    ) -> [Id; 2] {
        let function_body = self.function_body;
        let store = &mut *self.store;
        let props = store.identifier("props");
        let spread =
            store.add(Kind::JSX_SPREAD_ATTRIBUTE, [(Key::ARGUMENT, props.into())]);
        let name = store.jsx_identifier("_createMdxContent");
        let opening = store.add(
            Kind::JSX_OPENING_ELEMENT,
            [
                (Key::NAME, name.into()),
                (Key::ATTRIBUTES, Value::nodes([spread])),
                (Key::SELF_CLOSING, true.into()),
            ],
        );
        let inner = store.add(
            Kind::JSX_ELEMENT,
            [
                (Key::OPENING_ELEMENT, opening.into()),
                (Key::CLOSING_ELEMENT, Value::Null),
                (Key::CHILDREN, Value::List(Vec::new())),
            ],
        );
        let props = store.identifier("props");
        let spread =
            store.add(Kind::JSX_SPREAD_ATTRIBUTE, [(Key::ARGUMENT, props.into())]);
        let name = store.jsx_identifier("MDXLayout");
        let opening = store.add(
            Kind::JSX_OPENING_ELEMENT,
            [
                (Key::NAME, name.into()),
                (Key::ATTRIBUTES, Value::nodes([spread])),
                (Key::SELF_CLOSING, false.into()),
            ],
        );
        let name = store.jsx_identifier("MDXLayout");
        let closing = store.add(Kind::JSX_CLOSING_ELEMENT, [(Key::NAME, name.into())]);
        let element = store.add(
            Kind::JSX_ELEMENT,
            [
                (Key::OPENING_ELEMENT, opening.into()),
                (Key::CLOSING_ELEMENT, closing.into()),
                (Key::CHILDREN, Value::nodes([inner])),
            ],
        );
        let mut result = element;
        if !has_internal_layout {
            let test = store.identifier("MDXLayout");
            let callee = store.identifier("_createMdxContent");
            let props = store.identifier("props");
            let alternate = store.call(callee, vec![props]);
            result = store.add(
                Kind::CONDITIONAL_EXPRESSION,
                [
                    (Key::TEST, test.into()),
                    (Key::CONSEQUENT, element.into()),
                    (Key::ALTERNATE, alternate.into()),
                ],
            );
        }
        let mut argument = content.unwrap_or_else(|| store.identifier("undefined"));
        if store.is(argument, Kind::JSX_FRAGMENT) {
            let children = store.list(argument, Key::CHILDREN);
            if let [Value::Node(only)] = children
                && store.is(*only, Kind::JSX_ELEMENT)
            {
                argument = *only;
            }
        }
        let mut visitor = AwaitVisitor { found: false };
        walk(store, argument, &mut visitor);
        let left = store.identifier("props");
        let right = store.object(Vec::new());
        let parameter = store.add(
            Kind::ASSIGNMENT_PATTERN,
            [(Key::LEFT, left.into()), (Key::RIGHT, right.into())],
        );
        let returned = store.return_statement(Some(result));
        let body = store.block(vec![returned]);
        let id = store.identifier("MDXContent");
        let declaration = store.add(
            Kind::FUNCTION_DECLARATION,
            [
                (Key::ID, id.into()),
                (Key::PARAMS, Value::nodes([parameter])),
                (Key::BODY, body.into()),
            ],
        );
        let id = store.identifier("_createMdxContent");
        let props = store.identifier("props");
        let returned = store.return_statement(Some(argument));
        let body = store.block(vec![returned]);
        let create = store.add(
            Kind::FUNCTION_DECLARATION,
            [
                (Key::ASYNC, visitor.found.into()),
                (Key::ID, id.into()),
                (Key::PARAMS, Value::nodes([props])),
                (Key::BODY, body.into()),
            ],
        );
        let exported = if function_body {
            declaration
        } else {
            store.add(
                Kind::EXPORT_DEFAULT_DECLARATION,
                [(Key::DECLARATION, declaration.into())],
            )
        };
        [create, exported]
    }
}

/// `specifiersToDeclarations`: import or export specifiers as declarators taking them from
/// `init`, a namespace import first.
pub(crate) fn specifiers_to_declarations(
    store: &mut Store,
    specifiers: &[Id],
    init: Id,
) -> Vec<Id> {
    let mut declarations = Vec::new();
    let namespace = specifiers
        .iter()
        .copied()
        .rfind(|specifier| store.is(*specifier, Kind::IMPORT_NAMESPACE_SPECIFIER));
    let mut init = init;
    if let Some(namespace) = namespace {
        let local = store.child(namespace, Key::LOCAL);
        let declarator = store.add(
            Kind::VARIABLE_DECLARATOR,
            [(Key::ID, local.into()), (Key::INIT, init.into())],
        );
        store.copy_place(namespace, declarator);
        declarations.push(declarator);
        let name = local
            .and_then(|local| store.str(local, Key::NAME))
            .unwrap_or_default()
            .to_string();
        init = store.identifier(&name);
    }
    let mut properties = Vec::new();
    for specifier in specifiers.iter().copied() {
        if store.is(specifier, Kind::IMPORT_NAMESPACE_SPECIFIER) {
            continue;
        }
        let (key, value) = match store.kind(specifier) {
            Kind::IMPORT_SPECIFIER => (
                store.child(specifier, Key::IMPORTED),
                store.child(specifier, Key::LOCAL),
            ),
            Kind::EXPORT_SPECIFIER => (
                store.child(specifier, Key::LOCAL),
                store.child(specifier, Key::EXPORTED),
            ),
            _ => (
                Some(store.identifier("default")),
                store.child(specifier, Key::LOCAL),
            ),
        };
        let (Some(key), Some(value)) = (key, value) else {
            continue;
        };
        let shorthand = store.is(key, Kind::IDENTIFIER)
            && store.is(value, Kind::IDENTIFIER)
            && store.str(key, Key::NAME) == store.str(value, Key::NAME);
        let property = store.property(key, value, shorthand);
        store.copy_place(specifier, property);
        properties.push(property);
    }
    let pattern = store.add(
        Kind::OBJECT_PATTERN,
        [(Key::PROPERTIES, Value::nodes(properties))],
    );
    declarations.push(store.declarator(pattern, Some(init)));
    declarations
}

struct DefineVisitor {
    scopes: Scopes,
}

impl Visit for DefineVisitor {
    fn enter(&mut self, store: &mut Store, id: Id) -> Enter {
        self.scopes.enter(store, id);
        if matches!(
            store.kind(id),
            Kind::ARROW_FUNCTION_EXPRESSION
                | Kind::FUNCTION_DECLARATION
                | Kind::FUNCTION_EXPRESSION
        ) {
            self.scopes.exit(store, id);
            return Enter {
                skip: true,
                replace: None,
            };
        }
        Enter::default()
    }

    fn leave(&mut self, store: &mut Store, id: Id) -> Option<Id> {
        self.scopes.exit(store, id);
        None
    }
}

struct AwaitVisitor {
    found: bool,
}

impl Visit for AwaitVisitor {
    fn enter(&mut self, store: &mut Store, id: Id) -> Enter {
        match store.kind(id) {
            Kind::ARROW_FUNCTION_EXPRESSION
            | Kind::FUNCTION_DECLARATION
            | Kind::FUNCTION_EXPRESSION => {
                return Enter {
                    skip: true,
                    replace: None,
                };
            }
            Kind::AWAIT_EXPRESSION => self.found = true,
            Kind::FOR_OF_STATEMENT if store.flag(id, Key::AWAIT) => self.found = true,
            _ => {}
        }
        Enter::default()
    }

    fn leave(&mut self, _store: &mut Store, _id: Id) -> Option<Id> {
        None
    }
}

// Without `baseUrl`, a function body resolves `import()` and `import.meta.url` against the
// `baseUrl` its caller passes, through helpers the program gets
fn resolve_dynamic(store: &mut Store, program: Id) {
    let mut visitor = DynamicVisitor {
        import: false,
        meta: false,
    };
    walk(store, program, &mut visitor);
    if visitor.import {
        visitor.meta = true;
        let helper = resolve_dynamic_specifier(store);
        store.list_mut(program, Key::BODY).push(helper.into());
    }
    if visitor.meta {
        let statements = import_meta_url_variable(store);
        let body = store.list_mut(program, Key::BODY);
        for (index, statement) in statements.into_iter().enumerate() {
            body.insert(index, statement.into());
        }
    }
}

struct DynamicVisitor {
    import: bool,
    meta: bool,
}

impl Visit for DynamicVisitor {
    fn enter(&mut self, store: &mut Store, id: Id) -> Enter {
        if store.is(id, Kind::IMPORT_EXPRESSION) {
            self.import = true;
            let callee = store.identifier("_resolveDynamicMdxSpecifier");
            let source = store.child(id, Key::SOURCE);
            let call = store.call(callee, source.into_iter().collect());
            store.set(id, Key::SOURCE, call);
            return Enter::default();
        }
        let object = store.child(id, Key::OBJECT);
        let property = store.child(id, Key::PROPERTY);
        if store.is(id, Kind::MEMBER_EXPRESSION)
            && let (Some(object), Some(property)) = (object, property)
            && store.is(object, Kind::META_PROPERTY)
            && store.is(property, Kind::IDENTIFIER)
            && store
                .child(object, Key::META)
                .and_then(|meta| store.str(meta, Key::NAME))
                == Some("import")
            && store
                .child(object, Key::PROPERTY)
                .and_then(|name| store.str(name, Key::NAME))
                == Some("meta")
            && store.str(property, Key::NAME) == Some("url")
        {
            self.meta = true;
            let replacement = store.identifier("_importMetaUrl");
            store.copy_place(id, replacement);
            return Enter {
                skip: false,
                replace: Some(replacement),
            };
        }
        Enter::default()
    }

    fn leave(&mut self, _store: &mut Store, _id: Id) -> Option<Id> {
        None
    }
}

fn member_chain(store: &mut Store, names: &[&str]) -> Id {
    let mut object = store.identifier(names[0]);
    for name in &names[1..] {
        let property = store.identifier(name);
        object = store.member(object, property, false);
    }
    object
}

fn starts_with_call(store: &mut Store, prefix: &str) -> Id {
    let callee = member_chain(store, &["d", "startsWith"]);
    let argument = store.literal(prefix);
    store.call(callee, vec![argument])
}

fn resolve_dynamic_specifier(store: &mut Store) -> Id {
    let d = store.identifier("d");
    let typeof_d = store.add(
        Kind::UNARY_EXPRESSION,
        [
            (Key::OPERATOR, "typeof".into()),
            (Key::PREFIX, true.into()),
            (Key::ARGUMENT, d.into()),
        ],
    );
    let string = store.literal("string");
    let test = store.add(
        Kind::BINARY_EXPRESSION,
        [
            (Key::LEFT, typeof_d.into()),
            (Key::OPERATOR, "!==".into()),
            (Key::RIGHT, string.into()),
        ],
    );
    let d = store.identifier("d");
    let consequent = store.return_statement(Some(d));
    let first = store.add(
        Kind::IF_STATEMENT,
        [
            (Key::TEST, test.into()),
            (Key::CONSEQUENT, consequent.into()),
            (Key::ALTERNATE, Value::Null),
        ],
    );
    let url = store.identifier("URL");
    let d = store.identifier("d");
    let construct = store.add(
        Kind::NEW_EXPRESSION,
        [
            (Key::CALLEE, url.into()),
            (Key::ARGUMENTS, Value::nodes([d])),
        ],
    );
    let construct = store.expression_statement(construct);
    let d = store.identifier("d");
    let returned = store.return_statement(Some(d));
    let block = store.block(vec![construct, returned]);
    let empty = store.block(Vec::new());
    let handler = store.add(
        Kind::CATCH_CLAUSE,
        [(Key::PARAM, Value::Null), (Key::BODY, empty.into())],
    );
    let second = store.add(
        Kind::TRY_STATEMENT,
        [
            (Key::BLOCK, block.into()),
            (Key::HANDLER, handler.into()),
            (Key::FINALIZER, Value::Null),
        ],
    );
    let absolute = starts_with_call(store, "/");
    let current = starts_with_call(store, "./");
    let left = store.add(
        Kind::LOGICAL_EXPRESSION,
        [
            (Key::LEFT, absolute.into()),
            (Key::OPERATOR, "||".into()),
            (Key::RIGHT, current.into()),
        ],
    );
    let parent = starts_with_call(store, "../");
    let test = store.add(
        Kind::LOGICAL_EXPRESSION,
        [
            (Key::LEFT, left.into()),
            (Key::OPERATOR, "||".into()),
            (Key::RIGHT, parent.into()),
        ],
    );
    let url = store.identifier("URL");
    let d = store.identifier("d");
    let meta = store.identifier("_importMetaUrl");
    let construct = store.add(
        Kind::NEW_EXPRESSION,
        [
            (Key::CALLEE, url.into()),
            (Key::ARGUMENTS, Value::nodes([d, meta])),
        ],
    );
    let href = store.identifier("href");
    let resolved = store.member(construct, href, false);
    let consequent = store.return_statement(Some(resolved));
    let third = store.add(
        Kind::IF_STATEMENT,
        [
            (Key::TEST, test.into()),
            (Key::CONSEQUENT, consequent.into()),
            (Key::ALTERNATE, Value::Null),
        ],
    );
    let d = store.identifier("d");
    let fourth = store.return_statement(Some(d));
    let body = store.block(vec![first, second, third, fourth]);
    let id = store.identifier("_resolveDynamicMdxSpecifier");
    let d = store.identifier("d");
    store.add(
        Kind::FUNCTION_DECLARATION,
        [
            (Key::ID, id.into()),
            (Key::GENERATOR, false.into()),
            (Key::ASYNC, false.into()),
            (Key::PARAMS, Value::nodes([d])),
            (Key::BODY, body.into()),
        ],
    )
}

fn import_meta_url_variable(store: &mut Store) -> [Id; 2] {
    let arguments = store.identifier("arguments");
    let zero = store.literal(Value::Number(0.0));
    let first = store.member(arguments, zero, true);
    let base_url = store.identifier("baseUrl");
    let init = store.member(first, base_url, false);
    let id = store.identifier("_importMetaUrl");
    let declarator = store.declarator(id, Some(init));
    let variable = store.declaration("const", vec![declarator]);
    let meta = store.identifier("_importMetaUrl");
    let test = store.add(
        Kind::UNARY_EXPRESSION,
        [
            (Key::OPERATOR, "!".into()),
            (Key::PREFIX, true.into()),
            (Key::ARGUMENT, meta.into()),
        ],
    );
    let error = store.identifier("Error");
    let message = store.literal(
        "Unexpected missing `options.baseUrl` needed to support `export … from`, `import`, or `import.meta.url` when generating `function-body`",
    );
    let construct = store.add(
        Kind::NEW_EXPRESSION,
        [
            (Key::CALLEE, error.into()),
            (Key::ARGUMENTS, Value::nodes([message])),
        ],
    );
    let thrown = store.add(Kind::THROW_STATEMENT, [(Key::ARGUMENT, construct.into())]);
    let check = store.add(
        Kind::IF_STATEMENT,
        [
            (Key::TEST, test.into()),
            (Key::CONSEQUENT, thrown.into()),
            (Key::ALTERNATE, Value::Null),
        ],
    );
    [variable, check]
}
