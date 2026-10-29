use std::collections::HashSet;

use crate::actions::program::wrap::specifiers_to_declarations;
use crate::helpers::identifier::is_identifier_name;
use crate::helpers::walk::{Enter, Visit, walk};
use crate::models::estree::{Id, Store, Value};
use crate::models::options::CompileOptions;
use crate::models::scope::Scopes;

/// recma-jsx-rewrite of @mdx-js/mdx 3.1.1: in `_createMdxContent` (in every function with
/// a provider), take lowercase tags from `_components` and components from
/// `props.components` or the provider, checking each reference not in scope at runtime.
pub(crate) fn rewrite_program(store: &mut Store, program: Id, options: &CompileOptions) {
    let provider = options
        .provider_import_source
        .as_deref()
        .filter(|source| !source.is_empty());
    let function_body = options.output_format.as_deref() == Some("function-body");
    let mut rewrite = Rewrite {
        scopes: Scopes::default(),
        functions: Vec::new(),
        provider: provider.is_some(),
        import_provider: false,
        error_helper: false,
    };
    walk(store, program, &mut rewrite);
    if rewrite.import_provider
        && let Some(source) = provider
    {
        let statement = create_import_provider(store, source, function_body);
        store.list_mut(program, "body").insert(0, statement.into());
    }
    if rewrite.error_helper {
        let helper = create_error_helper(store);
        store.list_mut(program, "body").push(helper.into());
    }
    if function_body {
        let literal = store.literal("use strict");
        let directive = store.add(
            "ExpressionStatement",
            vec![
                ("expression", literal.into()),
                ("directive", "use strict".into()),
            ],
        );
        store.list_mut(program, "body").insert(0, directive.into());
    }
}

struct Reference {
    id: String,
    component: bool,
}

// Names in the order they were met, each once; a set makes membership free
#[derive(Default)]
struct Names {
    items: Vec<String>,
    seen: HashSet<String>,
}

impl Names {
    fn contains(&self, name: &str) -> bool {
        self.seen.contains(name)
    }

    fn insert(&mut self, name: &str) {
        if self.seen.insert(name.to_string()) {
            self.items.push(name.to_string());
        }
    }
}

struct FunctionInfo {
    components: Names,
    invalid: Vec<(String, String)>,
    node: Id,
    objects: Names,
    references: Vec<Reference>,
    referenced: HashSet<String>,
    tags: Names,
}

struct Rewrite {
    scopes: Scopes,
    functions: Vec<FunctionInfo>,
    provider: bool,
    import_provider: bool,
    error_helper: bool,
}

fn is_function(store: &Store, id: Id) -> bool {
    matches!(
        store.kind(id),
        "FunctionDeclaration" | "FunctionExpression" | "ArrowFunctionExpression"
    )
}

fn is_named(store: &Store, id: Id, name: &str) -> bool {
    store
        .child(id, "id")
        .and_then(|id| store.str(id, "name"))
        .is_some_and(|found| found == name)
}

// JS's default `sort`: by UTF-16 code units
fn sort_units(names: &mut [String]) {
    names.sort_by(|left, right| left.encode_utf16().cmp(right.encode_utf16()));
}

impl FunctionInfo {
    fn has_reference(&self, id: &str) -> bool {
        self.referenced.contains(id)
    }

    fn add_reference(&mut self, id: String, component: bool) {
        self.referenced.insert(id.clone());
        self.references.push(Reference { id, component });
    }
}

impl Visit for Rewrite {
    fn enter(&mut self, store: &mut Store, id: Id) -> Enter {
        self.scopes.enter(store, id);
        if is_function(store, id) {
            self.functions.push(FunctionInfo {
                components: Names::default(),
                invalid: Vec::new(),
                node: id,
                objects: Names::default(),
                references: Vec::new(),
                referenced: HashSet::new(),
                tags: Names::default(),
            });
            if is_named(store, id, "MDXContent") && !self.scopes.has("MDXLayout") {
                self.functions[0].components.insert("MDXLayout");
            }
        }
        // Upstream records into the outermost function, wherever the element is
        let Some(info) = self.functions.first() else {
            return Enter::default();
        };
        if !is_named(store, info.node, "_createMdxContent") && !self.provider {
            return Enter::default();
        }
        if store.is(id, "JSXElement") {
            self.element(store, id);
        }
        Enter::default()
    }

    fn leave(&mut self, store: &mut Store, id: Id) -> Option<Id> {
        self.scopes.exit(store, id);
        if is_function(store, id)
            && let Some(info) = self.functions.pop()
        {
            self.finish(store, id, info);
        }
        None
    }
}

impl Rewrite {
    fn element(&mut self, store: &mut Store, node: Id) {
        let Some(opening) = store.child(node, "openingElement") else {
            return;
        };
        let Some(mut name) = store.child(opening, "name") else {
            return;
        };
        let outermost = self.functions.len() == 1
            && store.is(self.functions[0].node, "FunctionDeclaration")
            && is_named(store, self.functions[0].node, "_createMdxContent");
        match store.kind(name) {
            "JSXMemberExpression" => {
                let mut ids = Vec::new();
                while store.is(name, "JSXMemberExpression") {
                    if let Some(property) = store.child(name, "property") {
                        ids.push(
                            store.str(property, "name").unwrap_or_default().to_string(),
                        );
                    }
                    match store.child(name, "object") {
                        Some(object) => name = object,
                        None => break,
                    }
                }
                let id = store.str(name, "name").unwrap_or_default().to_string();
                ids.push(id.clone());
                ids.reverse();
                let full = ids.join(".");
                let in_scope = self.scopes.has(&id);
                let info = &mut self.functions[0];
                if !info.has_reference(&full) && (!in_scope || outermost) {
                    info.add_reference(full, true);
                }
                if !in_scope {
                    info.objects.insert(&id);
                }
            }
            "JSXNamespacedName" => {}
            _ => {
                let id = store.str(name, "name").unwrap_or_default().to_string();
                if is_identifier_name(&id, false)
                    && !id.starts_with(|c: char| c.is_ascii_lowercase())
                {
                    if !self.scopes.has(&id) {
                        let info = &mut self.functions[0];
                        if id != "MDXLayout" && !info.has_reference(&id) {
                            info.add_reference(id.clone(), true);
                        }
                        info.components.insert(&id);
                    }
                } else if !store.flag(node, "explicit") {
                    let info = &mut self.functions[0];
                    info.tags.insert(&id);
                    let path = if is_identifier_name(&id, false) {
                        vec!["_components".to_string(), id]
                    } else {
                        let found = info
                            .invalid
                            .iter()
                            .find(|(tag, _)| *tag == id)
                            .map(|(_, name)| name.clone());
                        let name = found.unwrap_or_else(|| {
                            let name = format!("_component{}", info.invalid.len());
                            info.invalid.push((id, name.clone()));
                            name
                        });
                        vec![name]
                    };
                    let opening_name = jsx_member(store, &path);
                    store.set(opening, "name", opening_name);
                    if let Some(closing) = store.child(node, "closingElement") {
                        let closing_name = jsx_member(store, &path);
                        store.set(closing, "name", closing_name);
                    }
                }
            }
        }
    }

    fn finish(&mut self, store: &mut Store, node: Id, mut info: FunctionInfo) {
        let mut defaults = Vec::new();
        let mut tags = std::mem::take(&mut info.tags.items);
        sort_units(&mut tags);
        for name in &tags {
            let key = if is_identifier_name(name, false) {
                store.identifier(name)
            } else {
                store.literal(name.as_str())
            };
            let value = store.literal(name.as_str());
            defaults.push(store.property(key, value, false));
        }
        let mut actual = info.components.items.clone();
        for name in &info.objects.items {
            if !info.components.contains(name) {
                actual.push(name.clone());
            }
        }
        sort_units(&mut actual);
        let mut statements = Vec::new();
        let create = is_named(store, info.node, "_createMdxContent");
        if !defaults.is_empty() || !actual.is_empty() || !info.invalid.is_empty() {
            let mut parameters = Vec::new();
            if self.provider {
                self.import_provider = true;
                let callee = store.identifier("_provideComponents");
                parameters.push(store.call(callee, Vec::new()));
            }
            if is_named(store, info.node, "MDXContent") || create {
                parameters.push(id_or_member(store, &["props", "components"]));
            }
            if !defaults.is_empty() || parameters.len() > 1 {
                for parameter in &parameters {
                    defaults.push(store.spread(*parameter));
                }
            }
            let mut init = if defaults.is_empty() {
                let left = parameters.first().copied();
                let right = store.object(Vec::new());
                store.add(
                    "LogicalExpression",
                    vec![
                        ("operator", "||".into()),
                        ("left", left.into()),
                        ("right", right.into()),
                    ],
                )
            } else {
                store.object(defaults)
            };
            let pattern = (!actual.is_empty()).then(|| {
                let properties: Vec<Id> = actual
                    .iter()
                    .map(|name| {
                        let key_name = if name == "MDXLayout" { "wrapper" } else { name };
                        let key = store.identifier(key_name);
                        let value = store.identifier(name);
                        store.property(key, value, name != "MDXLayout")
                    })
                    .collect();
                store.add(
                    "ObjectPattern",
                    vec![("properties", Value::nodes(properties))],
                )
            });
            let mut declarations = Vec::new();
            if !tags.is_empty() {
                let id = store.identifier("_components");
                declarations.push(store.declarator(id, Some(init)));
                init = store.identifier("_components");
            }
            if create {
                let mut invalid = info.invalid.clone();
                invalid.sort_by(|left, right| left.0.cmp(&right.0));
                for (tag, name) in invalid {
                    let id = store.identifier(&name);
                    let object = store.identifier("_components");
                    let property = store.literal(tag.as_str());
                    let member = store.member(object, property, true);
                    declarations.push(store.declarator(id, Some(member)));
                }
            }
            if let Some(pattern) = pattern {
                declarations.push(store.declarator(pattern, Some(init)));
            }
            if !declarations.is_empty() {
                statements.push(store.declaration("const", declarations));
            }
        }
        let mut partials = Vec::new();
        for reference in &info.references {
            let parts: Vec<&str> = reference.id.split('.').collect();
            for index in 1..parts.len() {
                partials.push(parts[..index].join("."));
            }
        }
        for partial in partials {
            if !info.has_reference(&partial) {
                info.add_reference(partial, false);
            }
        }
        info.references
            .sort_by(|left, right| left.id.encode_utf16().cmp(right.id.encode_utf16()));
        for reference in &info.references {
            self.error_helper = true;
            let parts: Vec<&str> = reference.id.split('.').collect();
            let argument = id_or_member(store, &parts);
            let test = store.add(
                "UnaryExpression",
                vec![
                    ("operator", "!".into()),
                    ("prefix", true.into()),
                    ("argument", argument.into()),
                ],
            );
            let callee = store.identifier("_missingMdxReference");
            let id = store.literal(reference.id.as_str());
            let component = store.literal(reference.component);
            let call = store.call(callee, vec![id, component]);
            let consequent = store.expression_statement(call);
            statements.push(store.add(
                "IfStatement",
                vec![
                    ("test", test.into()),
                    ("consequent", consequent.into()),
                    ("alternate", Value::Null),
                ],
            ));
        }
        if statements.is_empty() {
            return;
        }
        let body = match store.child(node, "body") {
            Some(body) if store.is(body, "BlockStatement") => body,
            body => {
                let returned = store.return_statement(body);
                let block = store.block(vec![returned]);
                store.set(node, "body", block);
                block
            }
        };
        let list = store.list_mut(body, "body");
        for (index, statement) in statements.into_iter().enumerate() {
            list.insert(index, statement.into());
        }
    }
}

/// `toIdOrMemberExpression`: `a.b`, with `a["b-c"]` for a name that is no identifier.
pub(crate) fn id_or_member(store: &mut Store, names: &[&str]) -> Id {
    let mut object: Option<Id> = None;
    for name in names {
        let id = if is_identifier_name(name, false) {
            store.identifier(name)
        } else {
            store.literal(*name)
        };
        object = Some(match object {
            Some(object) => {
                let computed = store.is(id, "Literal");
                store.member(object, id, computed)
            }
            None => id,
        });
    }
    object.unwrap_or_else(|| store.identifier("undefined"))
}

// `toJsxIdOrMemberExpression`
fn jsx_member(store: &mut Store, names: &[String]) -> Id {
    let mut object: Option<Id> = None;
    for name in names {
        let id = store.jsx_identifier(name);
        object = Some(match object {
            Some(object) => store.add(
                "JSXMemberExpression",
                vec![("object", object.into()), ("property", id.into())],
            ),
            None => id,
        });
    }
    object.unwrap_or_else(|| store.jsx_identifier(""))
}

fn create_import_provider(store: &mut Store, source: &str, function_body: bool) -> Id {
    let imported = store.identifier("useMDXComponents");
    let local = store.identifier("_provideComponents");
    let specifier = store.add(
        "ImportSpecifier",
        vec![("imported", imported.into()), ("local", local.into())],
    );
    if function_body {
        let arguments = store.identifier("arguments");
        let zero = store.literal(Value::Number(0.0));
        let init = store.member(arguments, zero, true);
        let declarations = specifiers_to_declarations(store, &[specifier], init);
        return store.declaration("const", declarations);
    }
    let source = store.literal(source);
    store.add(
        "ImportDeclaration",
        vec![
            ("specifiers", Value::nodes([specifier])),
            ("attributes", Value::List(Vec::new())),
            ("source", source.into()),
        ],
    )
}

fn create_error_helper(store: &mut Store) -> Id {
    let expected = store.literal("Expected ");
    let test = store.identifier("component");
    let consequent = store.literal("component");
    let alternate = store.literal("object");
    let conditional = store.add(
        "ConditionalExpression",
        vec![
            ("test", test.into()),
            ("consequent", consequent.into()),
            ("alternate", alternate.into()),
        ],
    );
    let tick = store.literal(" `");
    let id = store.identifier("id");
    let rest = store
        .literal("` to be defined: you likely forgot to import, pass, or provide it.");
    let mut message = expected;
    for right in [conditional, tick, id, rest] {
        message = store.add(
            "BinaryExpression",
            vec![
                ("left", message.into()),
                ("operator", "+".into()),
                ("right", right.into()),
            ],
        );
    }
    let error = store.identifier("Error");
    let construct = store.add(
        "NewExpression",
        vec![
            ("callee", error.into()),
            ("arguments", Value::nodes([message])),
        ],
    );
    let thrown = store.add("ThrowStatement", vec![("argument", construct.into())]);
    let body = store.block(vec![thrown]);
    let name = store.identifier("_missingMdxReference");
    let id = store.identifier("id");
    let component = store.identifier("component");
    store.add(
        "FunctionDeclaration",
        vec![
            ("id", name.into()),
            ("generator", false.into()),
            ("async", false.into()),
            ("params", Value::nodes([id, component])),
            ("body", body.into()),
        ],
    )
}
