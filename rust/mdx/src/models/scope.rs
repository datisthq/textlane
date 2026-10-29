use std::collections::HashMap;

use crate::models::estree::{Id, Store, Value};
use crate::models::name::{Key, Kind};

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
            Kind::ARROW_FUNCTION_EXPRESSION => {
                self.open(false);
                self.define_parameters(store, id);
            }
            Kind::BLOCK_STATEMENT
            | Kind::DO_WHILE_STATEMENT
            | Kind::FOR_IN_STATEMENT
            | Kind::FOR_OF_STATEMENT
            | Kind::FOR_STATEMENT
            | Kind::WHILE_STATEMENT => self.open(true),
            Kind::CATCH_CLAUSE => {
                self.open(true);
                if let Some(param) = store.child(id, Key::PARAM) {
                    self.define_pattern(store, param, true);
                }
            }
            Kind::CLASS_DECLARATION => {
                if let Some(name) = name_of(store, id) {
                    self.define(name, false);
                }
            }
            Kind::FUNCTION_DECLARATION | Kind::FUNCTION_EXPRESSION => {
                if let Some(name) = name_of(store, id) {
                    self.define(name, false);
                }
                self.open(false);
                self.define_parameters(store, id);
            }
            Kind::IMPORT_DECLARATION => {
                for specifier in store.children(id, Key::SPECIFIERS) {
                    if let Some(local) = store.child(specifier, Key::LOCAL)
                        && let Some(name) = store.str(local, Key::NAME)
                    {
                        self.define(name.to_string(), false);
                    }
                }
            }
            Kind::VARIABLE_DECLARATION => {
                let block = store.str(id, Key::KIND) != Some("var");
                for declarator in store.children(id, Key::DECLARATIONS) {
                    if let Some(pattern) = store.child(declarator, Key::ID) {
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
            Kind::ARROW_FUNCTION_EXPRESSION
                | Kind::FUNCTION_DECLARATION
                | Kind::FUNCTION_EXPRESSION
                | Kind::BLOCK_STATEMENT
                | Kind::CATCH_CLAUSE
                | Kind::DO_WHILE_STATEMENT
                | Kind::FOR_IN_STATEMENT
                | Kind::FOR_OF_STATEMENT
                | Kind::FOR_STATEMENT
                | Kind::WHILE_STATEMENT
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
        for parameter in store.children(id, Key::PARAMS) {
            self.define_pattern(store, parameter, false);
        }
    }

    // Patterns nest as deep as the source, so they are walked with a stack
    fn define_pattern(&mut self, store: &Store, pattern: Id, block: bool) {
        let mut stack = vec![pattern];
        while let Some(pattern) = stack.pop() {
            match store.kind(pattern) {
                Kind::ARRAY_PATTERN => {
                    let elements = store.list(pattern, Key::ELEMENTS);
                    stack.extend(elements.iter().rev().filter_map(Value::as_node));
                }
                Kind::ASSIGNMENT_PATTERN => stack.extend(store.child(pattern, Key::LEFT)),
                Kind::IDENTIFIER => {
                    if let Some(name) = store.str(pattern, Key::NAME) {
                        self.define(name.to_string(), block);
                    }
                }
                Kind::OBJECT_PATTERN => {
                    for property in
                        store.children(pattern, Key::PROPERTIES).into_iter().rev()
                    {
                        if store.is(property, Kind::PROPERTY) {
                            stack.extend(store.child(property, Key::VALUE));
                        } else {
                            stack.push(property);
                        }
                    }
                }
                _ => stack.extend(store.child(pattern, Key::ARGUMENT)),
            }
        }
    }
}

fn name_of(store: &Store, id: Id) -> Option<String> {
    store
        .child(id, Key::ID)
        .and_then(|name| store.str(name, Key::NAME))
        .map(str::to_string)
}
