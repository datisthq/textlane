use crate::helpers::json::{format_number, json_string};
use crate::models::estree::{Id, Value};
use crate::models::name::{Key, Kind};
use crate::utils::printer::precedence::{NEEDS_PARENTHESES, below, precedence_of};
use crate::utils::printer::state::Printer;

impl Printer<'_> {
    /// astring's expression and pattern handlers; `false` for other kinds.
    pub(crate) fn expression(&mut self, id: Id) -> bool {
        match self.kind(id) {
            Kind::ARROW_FUNCTION_EXPRESSION => self.arrow(id),
            Kind::THIS_EXPRESSION => self.write("this"),
            Kind::SUPER => self.write("super"),
            Kind::REST_ELEMENT | Kind::SPREAD_ELEMENT => {
                self.write("...");
                self.field(id, Key::ARGUMENT);
            }
            Kind::YIELD_EXPRESSION => {
                self.write(if self.store.flag(id, Key::DELEGATE) {
                    "yield*"
                } else {
                    "yield"
                });
                if self.store.child(id, Key::ARGUMENT).is_some() {
                    self.write(" ");
                    self.field(id, Key::ARGUMENT);
                }
            }
            Kind::AWAIT_EXPRESSION => {
                self.write("await ");
                if let Some(argument) = self.store.child(id, Key::ARGUMENT) {
                    self.operand(argument, id, false);
                }
            }
            Kind::TEMPLATE_LITERAL => self.template(id),
            Kind::TEMPLATE_ELEMENT => self.template_raw(id),
            Kind::TAGGED_TEMPLATE_EXPRESSION => {
                if let Some(tag) = self.store.child(id, Key::TAG) {
                    self.operand(tag, id, false);
                }
                self.field(id, Key::QUASI);
            }
            Kind::ARRAY_EXPRESSION | Kind::ARRAY_PATTERN => self.array(id),
            Kind::OBJECT_EXPRESSION => self.object(id),
            Kind::PROPERTY => self.property(id),
            Kind::OBJECT_PATTERN => {
                self.write("{");
                for (index, property) in self
                    .store
                    .children(id, Key::PROPERTIES)
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
            Kind::SEQUENCE_EXPRESSION => {
                let expressions = self.store.list(id, Key::EXPRESSIONS).to_vec();
                self.sequence(&expressions);
            }
            Kind::UNARY_EXPRESSION => self.unary(id),
            Kind::UPDATE_EXPRESSION => {
                let operator = self
                    .store
                    .str(id, Key::OPERATOR)
                    .unwrap_or_default()
                    .to_string();
                if self.store.flag(id, Key::PREFIX) {
                    self.write(&operator);
                    self.field(id, Key::ARGUMENT);
                } else {
                    self.field(id, Key::ARGUMENT);
                    self.write(&operator);
                }
            }
            Kind::ASSIGNMENT_EXPRESSION => {
                self.field(id, Key::LEFT);
                let operator =
                    format!(" {} ", self.store.str(id, Key::OPERATOR).unwrap_or("="));
                self.write(&operator);
                self.field(id, Key::RIGHT);
            }
            Kind::ASSIGNMENT_PATTERN => {
                self.field(id, Key::LEFT);
                self.write(" = ");
                self.field(id, Key::RIGHT);
            }
            Kind::BINARY_EXPRESSION | Kind::LOGICAL_EXPRESSION => self.binary(id),
            Kind::CONDITIONAL_EXPRESSION => self.conditional(id),
            Kind::NEW_EXPRESSION => self.new_expression(id),
            Kind::CALL_EXPRESSION => self.call(id),
            Kind::CHAIN_EXPRESSION => self.field(id, Key::EXPRESSION),
            Kind::MEMBER_EXPRESSION => self.member(id),
            Kind::META_PROPERTY => {
                let text = format!(
                    "{}.{}",
                    self.name_of(id, Key::META),
                    self.name_of(id, Key::PROPERTY)
                );
                self.write(&text);
            }
            Kind::IDENTIFIER => {
                let name = self
                    .store
                    .str(id, Key::NAME)
                    .unwrap_or("undefined")
                    .to_string();
                self.write(&name);
            }
            Kind::PRIVATE_IDENTIFIER => {
                let name =
                    format!("#{}", self.store.str(id, Key::NAME).unwrap_or("undefined"));
                self.write(&name);
            }
            Kind::LITERAL => self.literal(id),
            Kind::IMPORT_EXPRESSION => {
                self.write("import(");
                self.field(id, Key::SOURCE);
                self.write(")");
            }
            _ => return false,
        }
        true
    }

    fn arrow(&mut self, id: Id) {
        if self.store.flag(id, Key::ASYNC) {
            self.write("async ");
        }
        let params = self.store.list(id, Key::PARAMS).to_vec();
        match params.as_slice() {
            [Value::Node(param)] if self.kind_text(*param).starts_with('I') => {
                let name = self
                    .store
                    .str(*param, Key::NAME)
                    .unwrap_or("undefined")
                    .to_string();
                self.write(&name);
            }
            _ => self.sequence(&params),
        }
        self.write(" => ");
        if self.kind_text_of(id, Key::BODY).starts_with('O') {
            self.write("(");
            self.field(id, Key::BODY);
            self.write(")");
        } else {
            self.field(id, Key::BODY);
        }
    }

    fn template(&mut self, id: Id) {
        let quasis = self.store.children(id, Key::QUASIS);
        let expressions = self.store.children(id, Key::EXPRESSIONS);
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
            .child(id, Key::VALUE)
            .and_then(|value| self.store.str(value, Key::RAW))
            .unwrap_or("undefined")
            .to_string();
        self.write(&raw);
    }

    fn array(&mut self, id: Id) {
        self.write("[");
        let elements = self.store.list(id, Key::ELEMENTS).to_vec();
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
        let properties = self.store.children(id, Key::PROPERTIES);
        if !properties.is_empty() {
            self.write("\n");
            self.comments_of(id, Key::COMMENTS, &property_indent);
            let count = properties.len();
            for (index, property) in properties.into_iter().enumerate() {
                self.comments_of(property, Key::COMMENTS, &property_indent);
                self.write(&property_indent);
                self.node(property);
                if index + 1 < count {
                    self.write(",\n");
                }
            }
            self.write("\n");
            self.comments_of(id, Key::TRAILING_COMMENTS, &property_indent);
            self.write(&indent);
            self.write("}");
        } else if self.store.has(id, Key::COMMENTS) {
            self.write("\n");
            self.comments_of(id, Key::COMMENTS, &property_indent);
            self.comments_of(id, Key::TRAILING_COMMENTS, &property_indent);
            self.write(&indent);
            self.write("}");
        } else if self.store.has(id, Key::TRAILING_COMMENTS) {
            self.write("\n");
            self.comments_of(id, Key::TRAILING_COMMENTS, &property_indent);
            self.write(&indent);
            self.write("}");
        } else {
            self.write("}");
        }
        self.level -= 1;
    }

    fn property(&mut self, id: Id) {
        if self.store.flag(id, Key::METHOD)
            || !self
                .store
                .str(id, Key::KIND)
                .unwrap_or("init")
                .starts_with('i')
        {
            self.method(id);
            return;
        }
        if !self.store.flag(id, Key::SHORTHAND) {
            if self.store.flag(id, Key::COMPUTED) {
                self.write("[");
                self.field(id, Key::KEY);
                self.write("]");
            } else {
                self.field(id, Key::KEY);
            }
            self.write(": ");
        }
        self.field(id, Key::VALUE);
    }

    fn unary(&mut self, id: Id) {
        let operator = self
            .store
            .str(id, Key::OPERATOR)
            .unwrap_or_default()
            .to_string();
        let Some(argument) = self.store.child(id, Key::ARGUMENT) else {
            return;
        };
        if !self.store.flag(id, Key::PREFIX) {
            self.node(argument);
            self.write(&operator);
            return;
        }
        self.write(&operator);
        let needs_parentheses = self.needs_parentheses(argument, id, false);
        let kind = self.kind(argument);
        let nested = (kind == Kind::UNARY_EXPRESSION || kind == Kind::UPDATE_EXPRESSION)
            && self.store.flag(argument, Key::PREFIX)
            && self
                .store
                .str(argument, Key::OPERATOR)
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
            .str(id, Key::OPERATOR)
            .unwrap_or_default()
            .to_string();
        let is_in = operator == "in";
        if is_in {
            self.write("(");
        }
        if let Some(left) = self.store.child(id, Key::LEFT) {
            self.operand(left, id, false);
        }
        self.write(&format!(" {operator} "));
        if let Some(right) = self.store.child(id, Key::RIGHT) {
            self.operand(right, id, true);
        }
        if is_in {
            self.write(")");
        }
    }

    fn conditional(&mut self, id: Id) {
        if let Some(test) = self.store.child(id, Key::TEST) {
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
        self.field(id, Key::CONSEQUENT);
        self.write(" : ");
        self.field(id, Key::ALTERNATE);
    }

    fn new_expression(&mut self, id: Id) {
        self.write("new ");
        if let Some(callee) = self.store.child(id, Key::CALLEE) {
            let precedence = self.precedence(callee);
            if precedence == Some(NEEDS_PARENTHESES)
                || below(precedence, precedence_of(Kind::CALL_EXPRESSION))
                || self.has_call(callee)
            {
                self.write("(");
                self.node(callee);
                self.write(")");
            } else {
                self.node(callee);
            }
        }
        let arguments = self.store.list(id, Key::ARGUMENTS).to_vec();
        self.sequence(&arguments);
    }

    // `hasCallExpression`: a call at the root of a member chain
    fn has_call(&self, id: Id) -> bool {
        let mut current = Some(id);
        while let Some(node) = current {
            let kind = self.kind_text(node);
            if kind.starts_with("Ca") {
                return true;
            }
            if !kind.starts_with("Mem") {
                return false;
            }
            current = self.store.child(node, Key::OBJECT);
        }
        false
    }

    fn call(&mut self, id: Id) {
        if let Some(callee) = self.store.child(id, Key::CALLEE) {
            let precedence = self.precedence(callee);
            if precedence == Some(NEEDS_PARENTHESES)
                || below(precedence, precedence_of(Kind::CALL_EXPRESSION))
            {
                self.write("(");
                self.node(callee);
                self.write(")");
            } else {
                self.node(callee);
            }
        }
        if self.store.flag(id, Key::OPTIONAL) {
            self.write("?.");
        }
        let arguments = self.store.list(id, Key::ARGUMENTS).to_vec();
        self.sequence(&arguments);
    }

    fn member(&mut self, id: Id) {
        if let Some(object) = self.store.child(id, Key::OBJECT) {
            let precedence = self.precedence(object);
            if precedence == Some(NEEDS_PARENTHESES)
                || below(precedence, precedence_of(Kind::MEMBER_EXPRESSION))
            {
                self.write("(");
                self.node(object);
                self.write(")");
            } else {
                self.node(object);
            }
        }
        let optional = self.store.flag(id, Key::OPTIONAL);
        if self.store.flag(id, Key::COMPUTED) {
            if optional {
                self.write("?.");
            }
            self.write("[");
            self.field(id, Key::PROPERTY);
            self.write("]");
        } else {
            self.write(if optional { "?." } else { "." });
            self.field(id, Key::PROPERTY);
        }
    }

    fn literal(&mut self, id: Id) {
        if let Some(raw) = self.store.str(id, Key::RAW) {
            let raw = raw.to_string();
            self.write(&raw);
            return;
        }
        if let Some(regex) = self.store.child(id, Key::REGEX) {
            let text = format!(
                "/{}/{}",
                self.store.str(regex, Key::PATTERN).unwrap_or_default(),
                self.store.str(regex, Key::FLAGS).unwrap_or_default()
            );
            self.write(&text);
            return;
        }
        if let Some(bigint) = self.store.str(id, Key::BIGINT) {
            let text = format!("{bigint}n");
            self.write(&text);
            return;
        }
        let text = match self.store.get(id, Key::VALUE) {
            Some(Value::Str(value)) => json_string(value),
            Some(Value::Number(value)) if value.is_finite() => format_number(*value),
            Some(Value::Bool(value)) => value.to_string(),
            _ => "null".to_string(),
        };
        self.write(&text);
    }
}
