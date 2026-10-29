/// An ESTree node's `type`, oxc's or a transform's: an index into the known names, past
/// them into its store's own, so checking a kind compares two integers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Kind(pub(crate) u32);

/// An ESTree field name, interned as `Kind` is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Key(pub(crate) u32);

macro_rules! names {
    ($name:ident, $table:ident, $($constant:ident = $text:literal,)*) => {
        pub(crate) const $table: &[&str] = &[$($text),*];

        impl $name {
            // Some names are only ever read from oxc's JSON, never by a transform
            $(#[allow(dead_code)]
            pub(crate) const $constant: $name = $name(find_const($table, $text));)*

            /// The known name `text`, if it is one.
            pub(crate) fn find(text: &str) -> Option<$name> {
                const SLOTS: [u16; SLOT_COUNT] = build_slots($table);
                find_slot(&SLOTS, $table, text).map($name)
            }
        }
    };
}

// Every JSON key goes through a lookup: an open-addressed table probed by a hash of a few
// bytes beats a binary search's string compares
const SLOT_COUNT: usize = 1024;
const EMPTY: u16 = u16::MAX;

const fn hash(bytes: &[u8]) -> usize {
    let len = bytes.len();
    if len == 0 {
        return 0;
    }
    let mixed = (len as u32).wrapping_mul(0x9e37)
        ^ (bytes[0] as u32).wrapping_mul(31)
        ^ (bytes[len - 1] as u32).wrapping_mul(131)
        ^ (bytes[len / 2] as u32).wrapping_mul(1031);
    mixed as usize % SLOT_COUNT
}

const fn build_slots(table: &[&str]) -> [u16; SLOT_COUNT] {
    let mut slots = [EMPTY; SLOT_COUNT];
    let mut index = 0;
    while index < table.len() {
        let mut slot = hash(table[index].as_bytes());
        while slots[slot] != EMPTY {
            slot = (slot + 1) % SLOT_COUNT;
        }
        slots[slot] = index as u16;
        index += 1;
    }
    slots
}

fn find_slot(slots: &[u16; SLOT_COUNT], table: &[&str], text: &str) -> Option<u32> {
    let mut slot = hash(text.as_bytes());
    loop {
        let index = *slots.get(slot)?;
        if index == EMPTY {
            return None;
        }
        if table.get(usize::from(index)) == Some(&text) {
            return Some(u32::from(index));
        }
        slot = (slot + 1) % SLOT_COUNT;
    }
}

const fn find_const(table: &[&str], text: &str) -> u32 {
    let mut index = 0;
    while index < table.len() {
        if is_same(table[index].as_bytes(), text.as_bytes()) {
            return index as u32;
        }
        index += 1;
    }
    panic!("a name missing from its table")
}

const fn is_same(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut index = 0;
    while index < left.len() {
        if left[index] != right[index] {
            return false;
        }
        index += 1;
    }
    true
}

names! {
    Kind,
    KINDS,
    NONE = "",
    ACCESSOR_PROPERTY = "AccessorProperty",
    ARRAY_EXPRESSION = "ArrayExpression",
    ARRAY_PATTERN = "ArrayPattern",
    ARROW_FUNCTION_EXPRESSION = "ArrowFunctionExpression",
    ASSIGNMENT_EXPRESSION = "AssignmentExpression",
    ASSIGNMENT_PATTERN = "AssignmentPattern",
    AWAIT_EXPRESSION = "AwaitExpression",
    BINARY_EXPRESSION = "BinaryExpression",
    BLOCK = "Block",
    BLOCK_STATEMENT = "BlockStatement",
    BREAK_STATEMENT = "BreakStatement",
    CALL_EXPRESSION = "CallExpression",
    CATCH_CLAUSE = "CatchClause",
    CHAIN_EXPRESSION = "ChainExpression",
    CLASS_BODY = "ClassBody",
    CLASS_DECLARATION = "ClassDeclaration",
    CLASS_EXPRESSION = "ClassExpression",
    CONDITIONAL_EXPRESSION = "ConditionalExpression",
    CONTINUE_STATEMENT = "ContinueStatement",
    DEBUGGER_STATEMENT = "DebuggerStatement",
    DECORATOR = "Decorator",
    DO_WHILE_STATEMENT = "DoWhileStatement",
    EMPTY_STATEMENT = "EmptyStatement",
    EXPORT_ALL_DECLARATION = "ExportAllDeclaration",
    EXPORT_DEFAULT_DECLARATION = "ExportDefaultDeclaration",
    EXPORT_NAMED_DECLARATION = "ExportNamedDeclaration",
    EXPORT_SPECIFIER = "ExportSpecifier",
    EXPRESSION_STATEMENT = "ExpressionStatement",
    FOR_IN_STATEMENT = "ForInStatement",
    FOR_OF_STATEMENT = "ForOfStatement",
    FOR_STATEMENT = "ForStatement",
    FUNCTION_DECLARATION = "FunctionDeclaration",
    FUNCTION_EXPRESSION = "FunctionExpression",
    IDENTIFIER = "Identifier",
    IF_STATEMENT = "IfStatement",
    IMPORT_ATTRIBUTE = "ImportAttribute",
    IMPORT_DECLARATION = "ImportDeclaration",
    IMPORT_DEFAULT_SPECIFIER = "ImportDefaultSpecifier",
    IMPORT_EXPRESSION = "ImportExpression",
    IMPORT_NAMESPACE_SPECIFIER = "ImportNamespaceSpecifier",
    IMPORT_SPECIFIER = "ImportSpecifier",
    JSX_ATTRIBUTE = "JSXAttribute",
    JSX_CLOSING_ELEMENT = "JSXClosingElement",
    JSX_CLOSING_FRAGMENT = "JSXClosingFragment",
    JSX_ELEMENT = "JSXElement",
    JSX_EMPTY_EXPRESSION = "JSXEmptyExpression",
    JSX_EXPRESSION_CONTAINER = "JSXExpressionContainer",
    JSX_FRAGMENT = "JSXFragment",
    JSX_IDENTIFIER = "JSXIdentifier",
    JSX_MEMBER_EXPRESSION = "JSXMemberExpression",
    JSX_NAMESPACED_NAME = "JSXNamespacedName",
    JSX_OPENING_ELEMENT = "JSXOpeningElement",
    JSX_OPENING_FRAGMENT = "JSXOpeningFragment",
    JSX_SPREAD_ATTRIBUTE = "JSXSpreadAttribute",
    JSX_SPREAD_CHILD = "JSXSpreadChild",
    JSX_TEXT = "JSXText",
    LABELED_STATEMENT = "LabeledStatement",
    LINE = "Line",
    LITERAL = "Literal",
    LOGICAL_EXPRESSION = "LogicalExpression",
    MEMBER_EXPRESSION = "MemberExpression",
    META_PROPERTY = "MetaProperty",
    METHOD_DEFINITION = "MethodDefinition",
    NEW_EXPRESSION = "NewExpression",
    OBJECT_EXPRESSION = "ObjectExpression",
    OBJECT_PATTERN = "ObjectPattern",
    PARENTHESIZED_EXPRESSION = "ParenthesizedExpression",
    PRIVATE_IDENTIFIER = "PrivateIdentifier",
    PROGRAM = "Program",
    PROPERTY = "Property",
    PROPERTY_DEFINITION = "PropertyDefinition",
    REST_ELEMENT = "RestElement",
    RETURN_STATEMENT = "ReturnStatement",
    SEQUENCE_EXPRESSION = "SequenceExpression",
    SPREAD_ELEMENT = "SpreadElement",
    STATIC_BLOCK = "StaticBlock",
    SUPER = "Super",
    SWITCH_CASE = "SwitchCase",
    SWITCH_STATEMENT = "SwitchStatement",
    TAGGED_TEMPLATE_EXPRESSION = "TaggedTemplateExpression",
    TEMPLATE_ELEMENT = "TemplateElement",
    TEMPLATE_LITERAL = "TemplateLiteral",
    THIS_EXPRESSION = "ThisExpression",
    THROW_STATEMENT = "ThrowStatement",
    TRY_STATEMENT = "TryStatement",
    UNARY_EXPRESSION = "UnaryExpression",
    UPDATE_EXPRESSION = "UpdateExpression",
    VARIABLE_DECLARATION = "VariableDeclaration",
    VARIABLE_DECLARATOR = "VariableDeclarator",
    WHILE_STATEMENT = "WhileStatement",
    WITH_STATEMENT = "WithStatement",
    YIELD_EXPRESSION = "YieldExpression",
}

names! {
    Key,
    KEYS,
    ALTERNATE = "alternate",
    ARGUMENT = "argument",
    ARGUMENTS = "arguments",
    ASYNC = "async",
    ATTRIBUTES = "attributes",
    AWAIT = "await",
    BIGINT = "bigint",
    BLOCK = "block",
    BODY = "body",
    CALLEE = "callee",
    CASES = "cases",
    CHILDREN = "children",
    CLOSING_ELEMENT = "closingElement",
    CLOSING_FRAGMENT = "closingFragment",
    COMMENTS = "comments",
    COMPUTED = "computed",
    CONSEQUENT = "consequent",
    COOKED = "cooked",
    DECLARATION = "declaration",
    DECLARATIONS = "declarations",
    DECORATORS = "decorators",
    DELEGATE = "delegate",
    DIRECTIVE = "directive",
    DISCRIMINANT = "discriminant",
    ELEMENTS = "elements",
    END = "end",
    EXPLICIT = "explicit",
    EXPORTED = "exported",
    EXPRESSION = "expression",
    EXPRESSIONS = "expressions",
    FINALIZER = "finalizer",
    FLAGS = "flags",
    GENERATOR = "generator",
    HANDLER = "handler",
    HASHBANG = "hashbang",
    ID = "id",
    IMPORTED = "imported",
    INIT = "init",
    KEY = "key",
    KIND = "kind",
    LABEL = "label",
    LEADING = "leading",
    LEFT = "left",
    LOCAL = "local",
    META = "meta",
    METHOD = "method",
    NAME = "name",
    NAMESPACE = "namespace",
    OBJECT = "object",
    OPENING_ELEMENT = "openingElement",
    OPENING_FRAGMENT = "openingFragment",
    OPERATOR = "operator",
    OPTIONAL = "optional",
    OPTIONS = "options",
    PARAM = "param",
    PARAMS = "params",
    PATTERN = "pattern",
    PHASE = "phase",
    PRAGMA = "pragma",
    PREFIX = "prefix",
    PROPERTIES = "properties",
    PROPERTY = "property",
    QUASI = "quasi",
    QUASIS = "quasis",
    RAW = "raw",
    REGEX = "regex",
    RIGHT = "right",
    SELF_CLOSING = "selfClosing",
    SHORTHAND = "shorthand",
    SOURCE = "source",
    SOURCE_TYPE = "sourceType",
    SPECIFIERS = "specifiers",
    START = "start",
    STATIC = "static",
    SUPER_CLASS = "superClass",
    TAG = "tag",
    TAIL = "tail",
    TEST = "test",
    TRAILING = "trailing",
    TRAILING_COMMENTS = "trailingComments",
    TYPE = "type",
    UPDATE = "update",
    VALUE = "value",
}

#[cfg(test)]
#[path = "name_unit.rs"]
mod unit;
