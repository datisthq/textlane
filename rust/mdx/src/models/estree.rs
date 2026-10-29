use crate::models::name::{KEYS, KINDS, Key, Kind};

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

/// One ESTree object: its `type` (empty for plain objects such as `regex`) and where its
/// other fields lie in the store, in insertion order, which walks and comment attachment
/// follow as upstream does.
#[derive(Clone, Copy, Debug)]
struct Node {
    kind: Kind,
    len: u32,
    start: usize,
}

/// The JS tree `mdx.compile` builds, as one flat store of ESTree objects, so a walk or a
/// replacement never recurses through owned children. Every node's fields sit together in
/// one shared list: a node that gains a field moves its fields to the end first.
#[derive(Debug, Default)]
pub(crate) struct Store {
    nodes: Vec<Node>,
    fields: Vec<(Key, Value)>,
    // Names oxc wrote that the tables lack, numbered after them
    kinds: Vec<Box<str>>,
    keys: Vec<Box<str>>,
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
        kind: Kind,
        fields: impl IntoIterator<Item = (Key, Value)>,
    ) -> Id {
        let start = self.fields.len();
        self.fields.extend(fields);
        self.nodes.push(Node {
            kind,
            len: (self.fields.len() - start) as u32,
            start,
        });
        Id(self.nodes.len() - 1)
    }

    pub(crate) fn len(&self) -> usize {
        self.nodes.len()
    }

    /// A node's fields, in order.
    pub(crate) fn fields(&self, id: Id) -> &[(Key, Value)] {
        let node = self.nodes[id.0];
        &self.fields[node.start..node.start + node.len as usize]
    }

    fn fields_mut(&mut self, id: Id) -> &mut [(Key, Value)] {
        let node = self.nodes[id.0];
        &mut self.fields[node.start..node.start + node.len as usize]
    }

    // Appending after another node's fields would overwrite them: move this node's to the end
    fn push_field(&mut self, id: Id, key: Key, value: Value) -> usize {
        let node = self.nodes[id.0];
        let end = node.start + node.len as usize;
        if end != self.fields.len() {
            let start = self.fields.len();
            for index in node.start..end {
                let field =
                    std::mem::replace(&mut self.fields[index], (key, Value::Null));
                self.fields.push(field);
            }
            self.nodes[id.0].start = start;
        }
        self.fields.push((key, value));
        self.nodes[id.0].len += 1;
        node.len as usize
    }

    /// The kind of a name, adding it to this store when the table lacks it.
    pub(crate) fn intern_kind(&mut self, text: &str) -> Kind {
        Kind::find(text)
            .unwrap_or_else(|| Kind((KINDS.len() + intern(&mut self.kinds, text)) as u32))
    }

    /// The key of a name, adding it to this store when the table lacks it.
    pub(crate) fn intern_key(&mut self, text: &str) -> Key {
        Key::find(text)
            .unwrap_or_else(|| Key((KEYS.len() + intern(&mut self.keys, text)) as u32))
    }

    pub(crate) fn kind(&self, id: Id) -> Kind {
        self.nodes[id.0].kind
    }

    /// A node's `type` as written.
    pub(crate) fn kind_text(&self, id: Id) -> &str {
        let index = self.nodes[id.0].kind.0 as usize;
        match KINDS.get(index) {
            Some(text) => text,
            None => self.kinds.get(index - KINDS.len()).map_or("", |text| text),
        }
    }

    pub(crate) fn is(&self, id: Id, kind: Kind) -> bool {
        self.nodes[id.0].kind == kind
    }

    pub(crate) fn set_kind(&mut self, id: Id, kind: Kind) {
        self.nodes[id.0].kind = kind;
    }

    pub(crate) fn get(&self, id: Id, key: Key) -> Option<&Value> {
        self.fields(id)
            .iter()
            .find(|(name, _)| *name == key)
            .map(|(_, value)| value)
    }

    pub(crate) fn has(&self, id: Id, key: Key) -> bool {
        self.get(id, key).is_some()
    }

    /// A field holding a node.
    pub(crate) fn child(&self, id: Id, key: Key) -> Option<Id> {
        self.get(id, key).and_then(Value::as_node)
    }

    /// A field holding a list, empty when missing.
    pub(crate) fn list(&self, id: Id, key: Key) -> &[Value] {
        match self.get(id, key) {
            Some(Value::List(items)) => items,
            _ => &[],
        }
    }

    /// The nodes of a list field, skipping holes.
    pub(crate) fn children(&self, id: Id, key: Key) -> Vec<Id> {
        self.list(id, key)
            .iter()
            .filter_map(Value::as_node)
            .collect()
    }

    pub(crate) fn str(&self, id: Id, key: Key) -> Option<&str> {
        match self.get(id, key) {
            Some(Value::Str(value)) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn number(&self, id: Id, key: Key) -> Option<f64> {
        match self.get(id, key) {
            Some(Value::Number(value)) => Some(*value),
            _ => None,
        }
    }

    /// Whether a field is `true`, as JS reads a truthy flag.
    pub(crate) fn flag(&self, id: Id, key: Key) -> bool {
        matches!(self.get(id, key), Some(Value::Bool(true)))
    }

    pub(crate) fn set(&mut self, id: Id, key: Key, value: impl Into<Value>) {
        let value = value.into();
        match self
            .fields_mut(id)
            .iter_mut()
            .find(|(name, _)| *name == key)
        {
            Some(field) => field.1 = value,
            None => {
                self.push_field(id, key, value);
            }
        }
    }

    /// The value of a node's field by its place in the node's fields.
    pub(crate) fn value_at_mut(&mut self, id: Id, field: usize) -> Option<&mut Value> {
        self.fields_mut(id).get_mut(field).map(|entry| &mut entry.1)
    }

    pub(crate) fn remove(&mut self, id: Id, key: Key) {
        while let Some(index) = self.fields(id).iter().position(|(name, _)| *name == key)
        {
            self.fields_mut(id)[index..].rotate_left(1);
            self.nodes[id.0].len -= 1;
        }
    }

    pub(crate) fn list_mut(&mut self, id: Id, key: Key) -> &mut Vec<Value> {
        let index = match self.fields(id).iter().position(|(name, _)| *name == key) {
            Some(index) => index,
            None => self.push_field(id, key, Value::List(Vec::new())),
        };
        let fields = self.fields_mut(id);
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
        let fields = self.fields(id).to_vec();
        self.add(self.nodes[id.0].kind, fields)
    }

    /// Copy `start` and `end` from one node to another, as upstream's `create` does.
    pub(crate) fn copy_place(&mut self, from: Id, to: Id) {
        for key in [Key::START, Key::END] {
            if let Some(value) = self.get(from, key).cloned() {
                self.set(to, key, value);
            }
        }
    }

    pub(crate) fn identifier(&mut self, name: &str) -> Id {
        self.add(Kind::IDENTIFIER, [(Key::NAME, name.into())])
    }

    /// A literal made by a transform, which has no `raw` and prints as JSON.
    pub(crate) fn literal(&mut self, value: impl Into<Value>) -> Id {
        self.add(Kind::LITERAL, [(Key::VALUE, value.into())])
    }

    pub(crate) fn property(&mut self, key: Id, value: Id, shorthand: bool) -> Id {
        self.add(
            Kind::PROPERTY,
            [
                (Key::KIND, "init".into()),
                (Key::KEY, key.into()),
                (Key::VALUE, value.into()),
                (Key::METHOD, false.into()),
                (Key::SHORTHAND, shorthand.into()),
                (Key::COMPUTED, false.into()),
            ],
        )
    }

    pub(crate) fn object(&mut self, properties: Vec<Id>) -> Id {
        self.add(
            Kind::OBJECT_EXPRESSION,
            [(Key::PROPERTIES, Value::nodes(properties))],
        )
    }

    pub(crate) fn spread(&mut self, argument: Id) -> Id {
        self.add(Kind::SPREAD_ELEMENT, [(Key::ARGUMENT, argument.into())])
    }

    pub(crate) fn call(&mut self, callee: Id, arguments: Vec<Id>) -> Id {
        self.add(
            Kind::CALL_EXPRESSION,
            [
                (Key::CALLEE, callee.into()),
                (Key::ARGUMENTS, Value::nodes(arguments)),
                (Key::OPTIONAL, false.into()),
            ],
        )
    }

    pub(crate) fn member(&mut self, object: Id, property: Id, computed: bool) -> Id {
        self.add(
            Kind::MEMBER_EXPRESSION,
            [
                (Key::OBJECT, object.into()),
                (Key::PROPERTY, property.into()),
                (Key::COMPUTED, computed.into()),
                (Key::OPTIONAL, false.into()),
            ],
        )
    }

    pub(crate) fn declaration(&mut self, kind: &str, declarators: Vec<Id>) -> Id {
        self.add(
            Kind::VARIABLE_DECLARATION,
            [
                (Key::KIND, kind.into()),
                (Key::DECLARATIONS, Value::nodes(declarators)),
            ],
        )
    }

    pub(crate) fn declarator(&mut self, id: Id, init: Option<Id>) -> Id {
        self.add(
            Kind::VARIABLE_DECLARATOR,
            [(Key::ID, id.into()), (Key::INIT, init.into())],
        )
    }

    pub(crate) fn block(&mut self, body: Vec<Id>) -> Id {
        self.add(Kind::BLOCK_STATEMENT, [(Key::BODY, Value::nodes(body))])
    }

    pub(crate) fn return_statement(&mut self, argument: Option<Id>) -> Id {
        self.add(Kind::RETURN_STATEMENT, [(Key::ARGUMENT, argument.into())])
    }

    pub(crate) fn expression_statement(&mut self, expression: Id) -> Id {
        self.add(
            Kind::EXPRESSION_STATEMENT,
            [(Key::EXPRESSION, expression.into())],
        )
    }

    pub(crate) fn jsx_identifier(&mut self, name: &str) -> Id {
        self.add(Kind::JSX_IDENTIFIER, [(Key::NAME, name.into())])
    }

    pub(crate) fn fragment(&mut self, children: Vec<Id>) -> Id {
        let opening = self.add(Kind::JSX_OPENING_FRAGMENT, []);
        let closing = self.add(Kind::JSX_CLOSING_FRAGMENT, []);
        self.add(
            Kind::JSX_FRAGMENT,
            [
                (Key::OPENING_FRAGMENT, opening.into()),
                (Key::CLOSING_FRAGMENT, closing.into()),
                (Key::CHILDREN, Value::nodes(children)),
            ],
        )
    }
}

fn intern(names: &mut Vec<Box<str>>, text: &str) -> usize {
    match names.iter().position(|name| &**name == text) {
        Some(index) => index,
        None => {
            names.push(text.into());
            names.len() - 1
        }
    }
}
