use crate::helpers::json::{format_number, json_string};
use crate::models::estree::{Id, Value};
use crate::utils::printer::precedence::{NEEDS_PARENTHESES, below, precedence_of};
use crate::utils::printer::state::Printer;

impl Printer<'_> {
    /// astring's expression and pattern handlers; `false` for other kinds.
    pub(crate) fn expression(&mut self, id: Id) -> bool {
        match self.kind(id) {
            "ArrowFunctionExpression" => self.arrow(id),
            "ThisExpression" => self.write("this"),
            "Super" => self.write("super"),
            "RestElement" | "SpreadElement" => {
                self.write("...");
                self.field(id, "argument");
            }
            "YieldExpression" => {
                self.write(if self.store.flag(id, "delegate") {
                    "yield*"
                } else {
                    "yield"
                });
                if self.store.child(id, "argument").is_some() {
                    self.write(" ");
                    self.field(id, "argument");
                }
            }
            "AwaitExpression" => {
                self.write("await ");
                if let Some(argument) = self.store.child(id, "argument") {
                    self.operand(argument, id, false);
                }
            }
            "TemplateLiteral" => self.template(id),
            "TemplateElement" => self.template_raw(id),
            "TaggedTemplateExpression" => {
                if let Some(tag) = self.store.child(id, "tag") {
                    self.operand(tag, id, false);
                }
                self.field(id, "quasi");
            }
            "ArrayExpression" | "ArrayPattern" => self.array(id),
            "ObjectExpression" => self.object(id),
            "Property" => self.property(id),
            "ObjectPattern" => {
                self.write("{");
                for (index, property) in self
                    .store
                    .children(id, "properties")
                    .into_iter()
                    .enumerate()
                {
                    if index > 0 {
                        self.write(", ");
                    }
                    self.node(property);
                }
                self.write("}");
            }
            "SequenceExpression" => {
                let expressions = self.store.list(id, "expressions").to_vec();
                self.sequence(&expressions);
            }
            "UnaryExpression" => self.unary(id),
            "UpdateExpression" => {
                let operator = self
                    .store
                    .str(id, "operator")
                    .unwrap_or_default()
                    .to_string();
                if self.store.flag(id, "prefix") {
                    self.write(&operator);
                    self.field(id, "argument");
                } else {
                    self.field(id, "argument");
                    self.write(&operator);
                }
            }
            "AssignmentExpression" => {
                self.field(id, "left");
                let operator =
                    format!(" {} ", self.store.str(id, "operator").unwrap_or("="));
                self.write(&operator);
                self.field(id, "right");
            }
            "AssignmentPattern" => {
                self.field(id, "left");
                self.write(" = ");
                self.field(id, "right");
            }
            "BinaryExpression" | "LogicalExpression" => self.binary(id),
            "ConditionalExpression" => self.conditional(id),
            "NewExpression" => self.new_expression(id),
            "CallExpression" => self.call(id),
            "ChainExpression" => self.field(id, "expression"),
            "MemberExpression" => self.member(id),
            "MetaProperty" => {
                let text = format!(
                    "{}.{}",
                    self.name_of(id, "meta"),
                    self.name_of(id, "property")
                );
                self.write(&text);
            }
            "Identifier" => {
                let name = self
                    .store
                    .str(id, "name")
                    .unwrap_or("undefined")
                    .to_string();
                self.write(&name);
            }
            "PrivateIdentifier" => {
                let name =
                    format!("#{}", self.store.str(id, "name").unwrap_or("undefined"));
                self.write(&name);
            }
            "Literal" => self.literal(id),
            "ImportExpression" => {
                self.write("import(");
                self.field(id, "source");
                self.write(")");
            }
            _ => return false,
        }
        true
    }

    fn arrow(&mut self, id: Id) {
        if self.store.flag(id, "async") {
            self.write("async ");
        }
        let params = self.store.list(id, "params").to_vec();
        match params.as_slice() {
            [Value::Node(param)] if self.kind(*param).starts_with('I') => {
                let name = self
                    .store
                    .str(*param, "name")
                    .unwrap_or("undefined")
                    .to_string();
                self.write(&name);
            }
            _ => self.sequence(&params),
        }
        self.write(" => ");
        if self.kind_of(id, "body").starts_with('O') {
            self.write("(");
            self.field(id, "body");
            self.write(")");
        } else {
            self.field(id, "body");
        }
    }

    fn template(&mut self, id: Id) {
        let quasis = self.store.children(id, "quasis");
        let expressions = self.store.children(id, "expressions");
        self.write("`");
        for (index, expression) in expressions.iter().enumerate() {
            if let Some(quasi) = quasis.get(index) {
                self.template_raw(*quasi);
            }
            self.write("${");
            self.node(*expression);
            self.write("}");
        }
        if let Some(quasi) = quasis.last() {
            self.template_raw(*quasi);
        }
        self.write("`");
    }

    fn template_raw(&mut self, id: Id) {
        let raw = self
            .store
            .child(id, "value")
            .and_then(|value| self.store.str(value, "raw"))
            .unwrap_or("undefined")
            .to_string();
        self.write(&raw);
    }

    fn array(&mut self, id: Id) {
        self.write("[");
        let elements = self.store.list(id, "elements").to_vec();
        let count = elements.len();
        for (index, element) in elements.iter().enumerate() {
            if let Value::Node(element) = element {
                self.node(*element);
            }
            // A trailing hole needs its comma written to stay a hole
            if index + 1 < count || !matches!(element, Value::Node(_)) {
                self.write(", ");
            }
        }
        self.write("]");
    }

    fn object(&mut self, id: Id) {
        let indent = self.indent(self.level);
        self.level += 1;
        let property_indent = self.indent(self.level);
        self.write("{");
        let properties = self.store.children(id, "properties");
        if !properties.is_empty() {
            self.write("\n");
            self.comments_of(id, "comments", &property_indent);
            let count = properties.len();
            for (index, property) in properties.into_iter().enumerate() {
                self.comments_of(property, "comments", &property_indent);
                self.write(&property_indent);
                self.node(property);
                if index + 1 < count {
                    self.write(",\n");
                }
            }
            self.write("\n");
            self.comments_of(id, "trailingComments", &property_indent);
            self.write(&indent);
            self.write("}");
        } else if self.store.has(id, "comments") {
            self.write("\n");
            self.comments_of(id, "comments", &property_indent);
            self.comments_of(id, "trailingComments", &property_indent);
            self.write(&indent);
            self.write("}");
        } else if self.store.has(id, "trailingComments") {
            self.write("\n");
            self.comments_of(id, "trailingComments", &property_indent);
            self.write(&indent);
            self.write("}");
        } else {
            self.write("}");
        }
        self.level -= 1;
    }

    fn property(&mut self, id: Id) {
        if self.store.flag(id, "method")
            || !self
                .store
                .str(id, "kind")
                .unwrap_or("init")
                .starts_with('i')
        {
            self.method(id);
            return;
        }
        if !self.store.flag(id, "shorthand") {
            if self.store.flag(id, "computed") {
                self.write("[");
                self.field(id, "key");
                self.write("]");
            } else {
                self.field(id, "key");
            }
            self.write(": ");
        }
        self.field(id, "value");
    }

    fn unary(&mut self, id: Id) {
        let operator = self
            .store
            .str(id, "operator")
            .unwrap_or_default()
            .to_string();
        let Some(argument) = self.store.child(id, "argument") else {
            return;
        };
        if !self.store.flag(id, "prefix") {
            self.node(argument);
            self.write(&operator);
            return;
        }
        self.write(&operator);
        let needs_parentheses = self.needs_parentheses(argument, id, false);
        let kind = self.kind(argument);
        let nested = (kind == "UnaryExpression" || kind == "UpdateExpression")
            && self.store.flag(argument, "prefix")
            && self
                .store
                .str(argument, "operator")
                .is_some_and(|inner| inner.chars().next() == operator.chars().next())
            && (operator == "+" || operator == "-");
        if !needs_parentheses && (operator.len() > 1 || nested) {
            self.write(" ");
        }
        if needs_parentheses {
            self.write(if operator.len() > 1 { " (" } else { "(" });
            self.node(argument);
            self.write(")");
        } else {
            self.node(argument);
        }
    }

    fn binary(&mut self, id: Id) {
        let operator = self
            .store
            .str(id, "operator")
            .unwrap_or_default()
            .to_string();
        let is_in = operator == "in";
        if is_in {
            self.write("(");
        }
        if let Some(left) = self.store.child(id, "left") {
            self.operand(left, id, false);
        }
        self.write(&format!(" {operator} "));
        if let Some(right) = self.store.child(id, "right") {
            self.operand(right, id, true);
        }
        if is_in {
            self.write(")");
        }
    }

    fn conditional(&mut self, id: Id) {
        if let Some(test) = self.store.child(id, "test") {
            let precedence = self.precedence(test);
            if precedence == Some(NEEDS_PARENTHESES)
                || matches!(precedence, Some(precedence) if precedence <= 4)
            {
                self.write("(");
                self.node(test);
                self.write(")");
            } else {
                self.node(test);
            }
        }
        self.write(" ? ");
        self.field(id, "consequent");
        self.write(" : ");
        self.field(id, "alternate");
    }

    fn new_expression(&mut self, id: Id) {
        self.write("new ");
        if let Some(callee) = self.store.child(id, "callee") {
            let precedence = self.precedence(callee);
            if precedence == Some(NEEDS_PARENTHESES)
                || below(precedence, precedence_of("CallExpression"))
                || self.has_call(callee)
            {
                self.write("(");
                self.node(callee);
                self.write(")");
            } else {
                self.node(callee);
            }
        }
        let arguments = self.store.list(id, "arguments").to_vec();
        self.sequence(&arguments);
    }

    // `hasCallExpression`: a call at the root of a member chain
    fn has_call(&self, id: Id) -> bool {
        let mut current = Some(id);
        while let Some(node) = current {
            let kind = self.kind(node);
            if kind.starts_with("Ca") {
                return true;
            }
            if !kind.starts_with("Mem") {
                return false;
            }
            current = self.store.child(node, "object");
        }
        false
    }

    fn call(&mut self, id: Id) {
        if let Some(callee) = self.store.child(id, "callee") {
            let precedence = self.precedence(callee);
            if precedence == Some(NEEDS_PARENTHESES)
                || below(precedence, precedence_of("CallExpression"))
            {
                self.write("(");
                self.node(callee);
                self.write(")");
            } else {
                self.node(callee);
            }
        }
        if self.store.flag(id, "optional") {
            self.write("?.");
        }
        let arguments = self.store.list(id, "arguments").to_vec();
        self.sequence(&arguments);
    }

    fn member(&mut self, id: Id) {
        if let Some(object) = self.store.child(id, "object") {
            let precedence = self.precedence(object);
            if precedence == Some(NEEDS_PARENTHESES)
                || below(precedence, precedence_of("MemberExpression"))
            {
                self.write("(");
                self.node(object);
                self.write(")");
            } else {
                self.node(object);
            }
        }
        let optional = self.store.flag(id, "optional");
        if self.store.flag(id, "computed") {
            if optional {
                self.write("?.");
            }
            self.write("[");
            self.field(id, "property");
            self.write("]");
        } else {
            self.write(if optional { "?." } else { "." });
            self.field(id, "property");
        }
    }

    fn literal(&mut self, id: Id) {
        if let Some(raw) = self.store.str(id, "raw") {
            let raw = raw.to_string();
            self.write(&raw);
            return;
        }
        if let Some(regex) = self.store.child(id, "regex") {
            let text = format!(
                "/{}/{}",
                self.store.str(regex, "pattern").unwrap_or_default(),
                self.store.str(regex, "flags").unwrap_or_default()
            );
            self.write(&text);
            return;
        }
        if let Some(bigint) = self.store.str(id, "bigint") {
            let text = format!("{bigint}n");
            self.write(&text);
            return;
        }
        let text = match self.store.get(id, "value") {
            Some(Value::Str(value)) => json_string(value),
            Some(Value::Number(value)) if value.is_finite() => format_number(*value),
            Some(Value::Bool(value)) => value.to_string(),
            _ => "null".to_string(),
        };
        self.write(&text);
    }
}
