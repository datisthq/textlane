use std::collections::HashMap;

use crate::models::estree::{Id, Store, Value};

/// One scope: a function's (`block: false`) or a block's, and the names defined in it.
pub(crate) struct Scope {
    pub(crate) block: bool,
    pub(crate) defined: Vec<String>,
}

/// estree-util-scope's visitors: the scopes open at a node of a walk, the program's first.
pub(crate) struct Scopes {
    pub(crate) scopes: Vec<Scope>,
    // How many open scopes define each name, so a lookup never scans them
    counts: HashMap<String, usize>,
}

impl Default for Scopes {
    fn default() -> Self {
        Self {
            scopes: vec![Scope {
                block: false,
                defined: Vec::new(),
            }],
            counts: HashMap::new(),
        }
    }
}

impl Scopes {
    pub(crate) fn enter(&mut self, store: &Store, id: Id) {
        match store.kind(id) {
            "ArrowFunctionExpression" => {
                self.open(false);
                self.define_parameters(store, id);
            }
            "BlockStatement" | "DoWhileStatement" | "ForInStatement"
            | "ForOfStatement" | "ForStatement" | "WhileStatement" => self.open(true),
            "CatchClause" => {
                self.open(true);
                if let Some(param) = store.child(id, "param") {
                    self.define_pattern(store, param, true);
                }
            }
            "ClassDeclaration" => {
                if let Some(name) = name_of(store, id) {
                    self.define(name, false);
                }
            }
            "FunctionDeclaration" | "FunctionExpression" => {
                if let Some(name) = name_of(store, id) {
                    self.define(name, false);
                }
                self.open(false);
                self.define_parameters(store, id);
            }
            "ImportDeclaration" => {
                for specifier in store.children(id, "specifiers") {
                    if let Some(local) = store.child(specifier, "local")
                        && let Some(name) = store.str(local, "name")
                    {
                        self.define(name.to_string(), false);
                    }
                }
            }
            "VariableDeclaration" => {
                let block = store.str(id, "kind") != Some("var");
                for declarator in store.children(id, "declarations") {
                    if let Some(pattern) = store.child(declarator, "id") {
                        self.define_pattern(store, pattern, block);
                    }
                }
            }
            _ => {}
        }
    }

    pub(crate) fn exit(&mut self, store: &Store, id: Id) {
        if matches!(
            store.kind(id),
            "ArrowFunctionExpression"
                | "FunctionDeclaration"
                | "FunctionExpression"
                | "BlockStatement"
                | "CatchClause"
                | "DoWhileStatement"
                | "ForInStatement"
                | "ForOfStatement"
                | "ForStatement"
                | "WhileStatement"
        ) && let Some(scope) = self.scopes.pop()
        {
            for name in scope.defined {
                if let Some(count) = self.counts.get_mut(&name) {
                    *count -= 1;
                }
            }
        }
    }

    /// Whether `name` is defined in an open scope.
    pub(crate) fn has(&self, name: &str) -> bool {
        self.counts.get(name).is_some_and(|count| *count > 0)
    }

    fn open(&mut self, block: bool) {
        self.scopes.push(Scope {
            block,
            defined: Vec::new(),
        });
    }

    fn define(&mut self, name: String, block: bool) {
        let index = self
            .scopes
            .iter()
            .rposition(|scope| block || !scope.block)
            .unwrap_or(0);
        *self.counts.entry(name.clone()).or_default() += 1;
        self.scopes[index].defined.push(name);
    }

    fn define_parameters(&mut self, store: &Store, id: Id) {
        for parameter in store.children(id, "params") {
            self.define_pattern(store, parameter, false);
        }
    }

    // Patterns nest as deep as the source, so they are walked with a stack
    fn define_pattern(&mut self, store: &Store, pattern: Id, block: bool) {
        let mut stack = vec![pattern];
        while let Some(pattern) = stack.pop() {
            match store.kind(pattern) {
                "ArrayPattern" => {
                    let elements = store.list(pattern, "elements");
                    stack.extend(elements.iter().rev().filter_map(Value::as_node));
                }
                "AssignmentPattern" => stack.extend(store.child(pattern, "left")),
                "Identifier" => {
                    if let Some(name) = store.str(pattern, "name") {
                        self.define(name.to_string(), block);
                    }
                }
                "ObjectPattern" => {
                    for property in
                        store.children(pattern, "properties").into_iter().rev()
                    {
                        if store.is(property, "Property") {
                            stack.extend(store.child(property, "value"));
                        } else {
                            stack.push(property);
                        }
                    }
                }
                _ => stack.extend(store.child(pattern, "argument")),
            }
        }
    }
}

fn name_of(store: &Store, id: Id) -> Option<String> {
    store
        .child(id, "id")
        .and_then(|name| store.str(name, "name"))
        .map(str::to_string)
}
