use std::borrow::Cow;

/// A node in a `Store`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Id(pub(crate) usize);

/// A field value, as ESTree's JSON has them: every object is a node in the store.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Value {
    Null,
    Bool(bool),
    Number(f64),
    Str(Box<str>),
    Node(Id),
    List(Vec<Value>),
}

/// One ESTree object: its `type` (empty for plain objects such as `regex`) and its other
/// fields in insertion order, which walks and comment attachment follow as upstream does.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Node {
    pub(crate) kind: Cow<'static, str>,
    pub(crate) fields: Vec<(Cow<'static, str>, Value)>,
}

/// The JS tree `mdx.compile` builds, as one flat store of ESTree objects, so a walk or a
/// replacement never recurses through owned children.
#[derive(Debug, Default)]
pub(crate) struct Store {
    nodes: Vec<Node>,
}

impl Value {
    pub(crate) fn str(value: &str) -> Self {
        Value::Str(value.into())
    }

    pub(crate) fn as_node(&self) -> Option<Id> {
        match self {
            Value::Node(id) => Some(*id),
            _ => None,
        }
    }

    pub(crate) fn nodes(ids: impl IntoIterator<Item = Id>) -> Self {
        Value::List(ids.into_iter().map(Value::Node).collect())
    }
}

impl From<Id> for Value {
    fn from(id: Id) -> Self {
        Value::Node(id)
    }
}

impl From<Option<Id>> for Value {
    fn from(id: Option<Id>) -> Self {
        id.map_or(Value::Null, Value::Node)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::str(value)
    }
}

impl Store {
    pub(crate) fn add(
        &mut self,
        kind: &'static str,
        fields: Vec<(&'static str, Value)>,
    ) -> Id {
        self.nodes.push(Node {
            kind: Cow::Borrowed(kind),
            fields: fields
                .into_iter()
                .map(|(key, value)| (Cow::Borrowed(key), value))
                .collect(),
        });
        Id(self.nodes.len() - 1)
    }

    pub(crate) fn add_node(&mut self, node: Node) -> Id {
        self.nodes.push(node);
        Id(self.nodes.len() - 1)
    }

    pub(crate) fn len(&self) -> usize {
        self.nodes.len()
    }

    pub(crate) fn node(&self, id: Id) -> &Node {
        &self.nodes[id.0]
    }

    pub(crate) fn kind(&self, id: Id) -> &str {
        &self.nodes[id.0].kind
    }

    pub(crate) fn is(&self, id: Id, kind: &str) -> bool {
        &*self.nodes[id.0].kind == kind
    }

    pub(crate) fn set_kind(&mut self, id: Id, kind: &'static str) {
        self.nodes[id.0].kind = Cow::Borrowed(kind);
    }

    pub(crate) fn get(&self, id: Id, key: &str) -> Option<&Value> {
        self.nodes[id.0]
            .fields
            .iter()
            .find(|(name, _)| &**name == key)
            .map(|(_, value)| value)
    }

    pub(crate) fn has(&self, id: Id, key: &str) -> bool {
        self.get(id, key).is_some()
    }

    /// A field holding a node.
    pub(crate) fn child(&self, id: Id, key: &str) -> Option<Id> {
        self.get(id, key).and_then(Value::as_node)
    }

    /// A field holding a list, empty when missing.
    pub(crate) fn list(&self, id: Id, key: &str) -> &[Value] {
        match self.get(id, key) {
            Some(Value::List(items)) => items,
            _ => &[],
        }
    }

    /// The nodes of a list field, skipping holes.
    pub(crate) fn children(&self, id: Id, key: &str) -> Vec<Id> {
        self.list(id, key)
            .iter()
            .filter_map(Value::as_node)
            .collect()
    }

    pub(crate) fn str(&self, id: Id, key: &str) -> Option<&str> {
        match self.get(id, key) {
            Some(Value::Str(value)) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn number(&self, id: Id, key: &str) -> Option<f64> {
        match self.get(id, key) {
            Some(Value::Number(value)) => Some(*value),
            _ => None,
        }
    }

    /// Whether a field is `true`, as JS reads a truthy flag.
    pub(crate) fn flag(&self, id: Id, key: &str) -> bool {
        matches!(self.get(id, key), Some(Value::Bool(true)))
    }

    pub(crate) fn set(&mut self, id: Id, key: &'static str, value: impl Into<Value>) {
        let value = value.into();
        let fields = &mut self.nodes[id.0].fields;
        match fields.iter_mut().find(|(name, _)| &**name == key) {
            Some(field) => field.1 = value,
            None => fields.push((Cow::Borrowed(key), value)),
        }
    }

    /// The value of a node's field by its place in the node's fields.
    pub(crate) fn value_at_mut(&mut self, id: Id, field: usize) -> Option<&mut Value> {
        self.nodes[id.0]
            .fields
            .get_mut(field)
            .map(|entry| &mut entry.1)
    }

    pub(crate) fn remove(&mut self, id: Id, key: &str) {
        self.nodes[id.0].fields.retain(|(name, _)| &**name != key);
    }

    pub(crate) fn list_mut(&mut self, id: Id, key: &'static str) -> &mut Vec<Value> {
        let fields = &mut self.nodes[id.0].fields;
        let index = match fields.iter().position(|(name, _)| &**name == key) {
            Some(index) => index,
            None => {
                fields.push((Cow::Borrowed(key), Value::List(Vec::new())));
                fields.len() - 1
            }
        };
        if !matches!(fields[index].1, Value::List(_)) {
            fields[index].1 = Value::List(Vec::new());
        }
        match &mut fields[index].1 {
            Value::List(items) => items,
            _ => unreachable!("the field was just made a list"),
        }
    }

    /// A shallow copy, as JS's `{...node}`.
    pub(crate) fn copy(&mut self, id: Id) -> Id {
        let node = self.nodes[id.0].clone();
        self.add_node(node)
    }

    /// Copy `start` and `end` from one node to another, as upstream's `create` does.
    pub(crate) fn copy_place(&mut self, from: Id, to: Id) {
        for key in ["start", "end"] {
            if let Some(value) = self.get(from, key).cloned() {
                self.set(to, key, value);
            }
        }
    }

    pub(crate) fn identifier(&mut self, name: &str) -> Id {
        self.add("Identifier", vec![("name", name.into())])
    }

    /// A literal made by a transform, which has no `raw` and prints as JSON.
    pub(crate) fn literal(&mut self, value: impl Into<Value>) -> Id {
        self.add("Literal", vec![("value", value.into())])
    }

    pub(crate) fn property(&mut self, key: Id, value: Id, shorthand: bool) -> Id {
        self.add(
            "Property",
            vec![
                ("kind", "init".into()),
                ("key", key.into()),
                ("value", value.into()),
                ("method", false.into()),
                ("shorthand", shorthand.into()),
                ("computed", false.into()),
            ],
        )
    }

    pub(crate) fn object(&mut self, properties: Vec<Id>) -> Id {
        self.add(
            "ObjectExpression",
            vec![("properties", Value::nodes(properties))],
        )
    }

    pub(crate) fn spread(&mut self, argument: Id) -> Id {
        self.add("SpreadElement", vec![("argument", argument.into())])
    }

    pub(crate) fn call(&mut self, callee: Id, arguments: Vec<Id>) -> Id {
        self.add(
            "CallExpression",
            vec![
                ("callee", callee.into()),
                ("arguments", Value::nodes(arguments)),
                ("optional", false.into()),
            ],
        )
    }

    pub(crate) fn member(&mut self, object: Id, property: Id, computed: bool) -> Id {
        self.add(
            "MemberExpression",
            vec![
                ("object", object.into()),
                ("property", property.into()),
                ("computed", computed.into()),
                ("optional", false.into()),
            ],
        )
    }

    pub(crate) fn declaration(&mut self, kind: &str, declarators: Vec<Id>) -> Id {
        self.add(
            "VariableDeclaration",
            vec![
                ("kind", kind.into()),
                ("declarations", Value::nodes(declarators)),
            ],
        )
    }

    pub(crate) fn declarator(&mut self, id: Id, init: Option<Id>) -> Id {
        self.add(
            "VariableDeclarator",
            vec![("id", id.into()), ("init", init.into())],
        )
    }

    pub(crate) fn block(&mut self, body: Vec<Id>) -> Id {
        self.add("BlockStatement", vec![("body", Value::nodes(body))])
    }

    pub(crate) fn return_statement(&mut self, argument: Option<Id>) -> Id {
        self.add("ReturnStatement", vec![("argument", argument.into())])
    }

    pub(crate) fn expression_statement(&mut self, expression: Id) -> Id {
        self.add(
            "ExpressionStatement",
            vec![("expression", expression.into())],
        )
    }

    pub(crate) fn jsx_identifier(&mut self, name: &str) -> Id {
        self.add("JSXIdentifier", vec![("name", name.into())])
    }

    pub(crate) fn fragment(&mut self, children: Vec<Id>) -> Id {
        let opening = self.add("JSXOpeningFragment", vec![]);
        let closing = self.add("JSXClosingFragment", vec![]);
        self.add(
            "JSXFragment",
            vec![
                ("openingFragment", opening.into()),
                ("closingFragment", closing.into()),
                ("children", Value::nodes(children)),
            ],
        )
    }
}
