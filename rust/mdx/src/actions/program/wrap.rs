use textlane_core::Error;

use crate::actions::hast::convert::Snippet;
use crate::helpers::place::stringify_place;
use crate::helpers::walk::{Enter, Visit, walk};
use crate::models::estree::{Id, Store, Value};
use crate::models::options::CompileOptions;
use crate::models::scope::Scopes;

/// recma-document of @mdx-js/mdx 3.1.1: inject the JSX pragmas, take the layout from
/// `export default`, and wrap the JSX in `_createMdxContent` and the `MDXContent` export;
/// as a function body, imports and exports become `await import()` and a final `return`.
pub(crate) fn wrap_program(
    store: &mut Store,
    program: Id,
    options: &CompileOptions,
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
            .add("ImportDefaultSpecifier", vec![("local", local.into())]);
        let source = wrap.store.literal(pragma_import_source);
        let import = wrap.store.add(
            "ImportDeclaration",
            vec![
                ("specifiers", Value::nodes([specifier])),
                ("attributes", Value::List(Vec::new())),
                ("source", source.into()),
            ],
        );
        wrap.handle_esm(import);
    }
    let mut layout: Option<Id> = None;
    let mut content = false;
    for child in wrap.store.children(program, "body") {
        match wrap.store.kind(child) {
            "ExportDefaultDeclaration" => {
                if let Some(previous) = layout {
                    return Err(duplicate_layout(wrap.store, snippets, previous));
                }
                layout = Some(child);
                let declaration = wrap.store.child(child, "declaration");
                let init =
                    declaration.map(|declaration| match wrap.store.kind(declaration) {
                        "FunctionDeclaration" | "ClassDeclaration" => {
                            let expression = wrap.store.copy(declaration);
                            let kind =
                                if wrap.store.is(declaration, "FunctionDeclaration") {
                                    "FunctionExpression"
                                } else {
                                    "ClassExpression"
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
            "ExportNamedDeclaration" if wrap.store.child(child, "source").is_some() => {
                let source = wrap.store.child(child, "source");
                let mut kept = Vec::new();
                for specifier in wrap.store.children(child, "specifiers") {
                    let exported = wrap.store.child(specifier, "exported");
                    let is_default = exported.is_some_and(|exported| {
                        wrap.store.is(exported, "Identifier")
                            && wrap.store.str(exported, "name") == Some("default")
                    });
                    if !is_default {
                        kept.push(specifier);
                        continue;
                    }
                    if let Some(previous) = layout {
                        return Err(duplicate_layout(wrap.store, snippets, previous));
                    }
                    layout = Some(specifier);
                    let local = wrap.store.child(specifier, "local");
                    let mdx_layout = wrap.store.identifier("MDXLayout");
                    let import_specifier = if local.is_some_and(|local| {
                        wrap.store.is(local, "Identifier")
                            && wrap.store.str(local, "name") == Some("default")
                    }) {
                        wrap.store.add(
                            "ImportDefaultSpecifier",
                            vec![("local", mdx_layout.into())],
                        )
                    } else {
                        let import_specifier = wrap.store.add(
                            "ImportSpecifier",
                            vec![
                                ("imported", local.into()),
                                ("local", mdx_layout.into()),
                            ],
                        );
                        if let Some(local) = local {
                            wrap.store.copy_place(local, import_specifier);
                        }
                        import_specifier
                    };
                    let value = source
                        .and_then(|source| wrap.store.get(source, "value").cloned());
                    let from = wrap.store.literal(value.unwrap_or(Value::Null));
                    if let Some(source) = source {
                        wrap.store.copy_place(source, from);
                    }
                    let declaration = wrap.store.add(
                        "ImportDeclaration",
                        vec![
                            ("specifiers", Value::nodes([import_specifier])),
                            ("attributes", Value::List(Vec::new())),
                            ("source", from.into()),
                        ],
                    );
                    wrap.store.copy_place(specifier, declaration);
                    wrap.handle_esm(declaration);
                }
                let remaining = !kept.is_empty();
                wrap.store.set(child, "specifiers", Value::nodes(kept));
                if remaining {
                    wrap.handle_export(child);
                }
            }
            "ExportNamedDeclaration" | "ExportAllDeclaration" => {
                wrap.handle_export(child)
            }
            "ImportDeclaration" => wrap.handle_esm(child),
            "ExpressionStatement"
                if wrap
                    .store
                    .child(child, "expression")
                    .is_some_and(|expression| {
                        matches!(
                            wrap.store.kind(expression),
                            "JSXElement" | "JSXFragment"
                        )
                    }) =>
            {
                content = true;
                let expression = wrap.store.child(child, "expression");
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
    wrap.store.set(program, "body", Value::nodes(replacement));
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
            "Block",
            vec![("value", format!("{name} {value}").as_str().into())],
        );
        self.store.set(comment, "pragma", true);
        self.store
            .list_mut(program, "comments")
            .insert(0, comment.into());
    }

    fn handle_export(&mut self, node: Id) {
        if self.store.is(node, "ExportNamedDeclaration") {
            if self.store.child(node, "declaration").is_some() {
                let mut visitor = DefineVisitor {
                    scopes: Scopes::default(),
                };
                walk(self.store, node, &mut visitor);
                let defined = std::mem::take(&mut visitor.scopes.scopes[0].defined);
                self.exported
                    .extend(defined.into_iter().map(|name| (name, None)));
            }
            for specifier in self.store.children(node, "specifiers") {
                if let Some(exported) = self.store.child(specifier, "exported") {
                    let name = match self.store.get(exported, "name") {
                        Some(Value::Str(name)) => name.to_string(),
                        _ => match self.store.get(exported, "value") {
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
        let kind = self.store.kind(node).to_string();
        let source = self.store.child(node, "source");
        let replace = if kind == "ImportDeclaration"
            || kind == "ExportAllDeclaration"
            || (kind == "ExportNamedDeclaration" && source.is_some())
        {
            let argument = self
                .store
                .add("ImportExpression", vec![("source", source.into())]);
            self.store.copy_place(node, argument);
            let init = self
                .store
                .add("AwaitExpression", vec![("argument", argument.into())]);
            let specifiers = self.store.children(node, "specifiers");
            if kind != "ExportAllDeclaration" && specifiers.is_empty() {
                Some(self.store.expression_statement(init))
            } else {
                let declarations = if kind == "ExportAllDeclaration" {
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
        } else if let Some(declaration) = self.store.child(node, "declaration") {
            Some(declaration)
        } else {
            let mut declarators = Vec::new();
            for specifier in self.store.children(node, "specifiers") {
                let (Some(exported), Some(local)) = (
                    self.store.child(specifier, "exported"),
                    self.store.child(specifier, "local"),
                ) else {
                    continue;
                };
                if self.store.is(exported, "Identifier")
                    && self.store.is(local, "Identifier")
                    && self.store.str(local, "name") != self.store.str(exported, "name")
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
        let spread = store.add("JSXSpreadAttribute", vec![("argument", props.into())]);
        let name = store.jsx_identifier("_createMdxContent");
        let opening = store.add(
            "JSXOpeningElement",
            vec![
                ("name", name.into()),
                ("attributes", Value::nodes([spread])),
                ("selfClosing", true.into()),
            ],
        );
        let inner = store.add(
            "JSXElement",
            vec![
                ("openingElement", opening.into()),
                ("closingElement", Value::Null),
                ("children", Value::List(Vec::new())),
            ],
        );
        let props = store.identifier("props");
        let spread = store.add("JSXSpreadAttribute", vec![("argument", props.into())]);
        let name = store.jsx_identifier("MDXLayout");
        let opening = store.add(
            "JSXOpeningElement",
            vec![
                ("name", name.into()),
                ("attributes", Value::nodes([spread])),
                ("selfClosing", false.into()),
            ],
        );
        let name = store.jsx_identifier("MDXLayout");
        let closing = store.add("JSXClosingElement", vec![("name", name.into())]);
        let element = store.add(
            "JSXElement",
            vec![
                ("openingElement", opening.into()),
                ("closingElement", closing.into()),
                ("children", Value::nodes([inner])),
            ],
        );
        let mut result = element;
        if !has_internal_layout {
            let test = store.identifier("MDXLayout");
            let callee = store.identifier("_createMdxContent");
            let props = store.identifier("props");
            let alternate = store.call(callee, vec![props]);
            result = store.add(
                "ConditionalExpression",
                vec![
                    ("test", test.into()),
                    ("consequent", element.into()),
                    ("alternate", alternate.into()),
                ],
            );
        }
        let mut argument = content.unwrap_or_else(|| store.identifier("undefined"));
        if store.is(argument, "JSXFragment") {
            let children = store.list(argument, "children");
            if let [Value::Node(only)] = children
                && store.is(*only, "JSXElement")
            {
                argument = *only;
            }
        }
        let mut visitor = AwaitVisitor { found: false };
        walk(store, argument, &mut visitor);
        let left = store.identifier("props");
        let right = store.object(Vec::new());
        let parameter = store.add(
            "AssignmentPattern",
            vec![("left", left.into()), ("right", right.into())],
        );
        let returned = store.return_statement(Some(result));
        let body = store.block(vec![returned]);
        let id = store.identifier("MDXContent");
        let declaration = store.add(
            "FunctionDeclaration",
            vec![
                ("id", id.into()),
                ("params", Value::nodes([parameter])),
                ("body", body.into()),
            ],
        );
        let id = store.identifier("_createMdxContent");
        let props = store.identifier("props");
        let returned = store.return_statement(Some(argument));
        let body = store.block(vec![returned]);
        let create = store.add(
            "FunctionDeclaration",
            vec![
                ("async", visitor.found.into()),
                ("id", id.into()),
                ("params", Value::nodes([props])),
                ("body", body.into()),
            ],
        );
        let exported = if function_body {
            declaration
        } else {
            store.add(
                "ExportDefaultDeclaration",
                vec![("declaration", declaration.into())],
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
        .rfind(|specifier| store.is(*specifier, "ImportNamespaceSpecifier"));
    let mut init = init;
    if let Some(namespace) = namespace {
        let local = store.child(namespace, "local");
        let declarator = store.add(
            "VariableDeclarator",
            vec![("id", local.into()), ("init", init.into())],
        );
        store.copy_place(namespace, declarator);
        declarations.push(declarator);
        let name = local
            .and_then(|local| store.str(local, "name"))
            .unwrap_or_default()
            .to_string();
        init = store.identifier(&name);
    }
    let mut properties = Vec::new();
    for specifier in specifiers.iter().copied() {
        if store.is(specifier, "ImportNamespaceSpecifier") {
            continue;
        }
        let (key, value) = match store.kind(specifier) {
            "ImportSpecifier" => (
                store.child(specifier, "imported"),
                store.child(specifier, "local"),
            ),
            "ExportSpecifier" => (
                store.child(specifier, "local"),
                store.child(specifier, "exported"),
            ),
            _ => (
                Some(store.identifier("default")),
                store.child(specifier, "local"),
            ),
        };
        let (Some(key), Some(value)) = (key, value) else {
            continue;
        };
        let shorthand = store.is(key, "Identifier")
            && store.is(value, "Identifier")
            && store.str(key, "name") == store.str(value, "name");
        let property = store.property(key, value, shorthand);
        store.copy_place(specifier, property);
        properties.push(property);
    }
    let pattern = store.add(
        "ObjectPattern",
        vec![("properties", Value::nodes(properties))],
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
            "ArrowFunctionExpression" | "FunctionDeclaration" | "FunctionExpression"
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
            "ArrowFunctionExpression" | "FunctionDeclaration" | "FunctionExpression" => {
                return Enter {
                    skip: true,
                    replace: None,
                };
            }
            "AwaitExpression" => self.found = true,
            "ForOfStatement" if store.flag(id, "await") => self.found = true,
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
        store.list_mut(program, "body").push(helper.into());
    }
    if visitor.meta {
        let statements = import_meta_url_variable(store);
        let body = store.list_mut(program, "body");
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
        if store.is(id, "ImportExpression") {
            self.import = true;
            let callee = store.identifier("_resolveDynamicMdxSpecifier");
            let source = store.child(id, "source");
            let call = store.call(callee, source.into_iter().collect());
            store.set(id, "source", call);
            return Enter::default();
        }
        let object = store.child(id, "object");
        let property = store.child(id, "property");
        if store.is(id, "MemberExpression")
            && let (Some(object), Some(property)) = (object, property)
            && store.is(object, "MetaProperty")
            && store.is(property, "Identifier")
            && store
                .child(object, "meta")
                .and_then(|meta| store.str(meta, "name"))
                == Some("import")
            && store
                .child(object, "property")
                .and_then(|name| store.str(name, "name"))
                == Some("meta")
            && store.str(property, "name") == Some("url")
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
        "UnaryExpression",
        vec![
            ("operator", "typeof".into()),
            ("prefix", true.into()),
            ("argument", d.into()),
        ],
    );
    let string = store.literal("string");
    let test = store.add(
        "BinaryExpression",
        vec![
            ("left", typeof_d.into()),
            ("operator", "!==".into()),
            ("right", string.into()),
        ],
    );
    let d = store.identifier("d");
    let consequent = store.return_statement(Some(d));
    let first = store.add(
        "IfStatement",
        vec![
            ("test", test.into()),
            ("consequent", consequent.into()),
            ("alternate", Value::Null),
        ],
    );
    let url = store.identifier("URL");
    let d = store.identifier("d");
    let construct = store.add(
        "NewExpression",
        vec![("callee", url.into()), ("arguments", Value::nodes([d]))],
    );
    let construct = store.expression_statement(construct);
    let d = store.identifier("d");
    let returned = store.return_statement(Some(d));
    let block = store.block(vec![construct, returned]);
    let empty = store.block(Vec::new());
    let handler = store.add(
        "CatchClause",
        vec![("param", Value::Null), ("body", empty.into())],
    );
    let second = store.add(
        "TryStatement",
        vec![
            ("block", block.into()),
            ("handler", handler.into()),
            ("finalizer", Value::Null),
        ],
    );
    let absolute = starts_with_call(store, "/");
    let current = starts_with_call(store, "./");
    let left = store.add(
        "LogicalExpression",
        vec![
            ("left", absolute.into()),
            ("operator", "||".into()),
            ("right", current.into()),
        ],
    );
    let parent = starts_with_call(store, "../");
    let test = store.add(
        "LogicalExpression",
        vec![
            ("left", left.into()),
            ("operator", "||".into()),
            ("right", parent.into()),
        ],
    );
    let url = store.identifier("URL");
    let d = store.identifier("d");
    let meta = store.identifier("_importMetaUrl");
    let construct = store.add(
        "NewExpression",
        vec![
            ("callee", url.into()),
            ("arguments", Value::nodes([d, meta])),
        ],
    );
    let href = store.identifier("href");
    let resolved = store.member(construct, href, false);
    let consequent = store.return_statement(Some(resolved));
    let third = store.add(
        "IfStatement",
        vec![
            ("test", test.into()),
            ("consequent", consequent.into()),
            ("alternate", Value::Null),
        ],
    );
    let d = store.identifier("d");
    let fourth = store.return_statement(Some(d));
    let body = store.block(vec![first, second, third, fourth]);
    let id = store.identifier("_resolveDynamicMdxSpecifier");
    let d = store.identifier("d");
    store.add(
        "FunctionDeclaration",
        vec![
            ("id", id.into()),
            ("generator", false.into()),
            ("async", false.into()),
            ("params", Value::nodes([d])),
            ("body", body.into()),
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
        "UnaryExpression",
        vec![
            ("operator", "!".into()),
            ("prefix", true.into()),
            ("argument", meta.into()),
        ],
    );
    let error = store.identifier("Error");
    let message = store.literal(
        "Unexpected missing `options.baseUrl` needed to support `export … from`, `import`, or `import.meta.url` when generating `function-body`",
    );
    let construct = store.add(
        "NewExpression",
        vec![
            ("callee", error.into()),
            ("arguments", Value::nodes([message])),
        ],
    );
    let thrown = store.add("ThrowStatement", vec![("argument", construct.into())]);
    let check = store.add(
        "IfStatement",
        vec![
            ("test", test.into()),
            ("consequent", thrown.into()),
            ("alternate", Value::Null),
        ],
    );
    [variable, check]
}
