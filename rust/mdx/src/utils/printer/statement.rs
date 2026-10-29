use crate::models::estree::{Id, Value};
use crate::utils::printer::precedence::{NEEDS_PARENTHESES, below, precedence_of};
use crate::utils::printer::state::Printer;

impl Printer<'_> {
    /// astring's statement, declaration and module handlers; `false` for other kinds.
    pub(crate) fn statement(&mut self, id: Id) -> bool {
        match self.kind(id) {
            "Program" => self.program(id),
            "BlockStatement" | "ClassBody" => self.block(id),
            "StaticBlock" => {
                self.write("static ");
                self.block(id);
            }
            "EmptyStatement" => self.write(";"),
            "ExpressionStatement" => self.expression_statement(id),
            "IfStatement" => {
                self.write("if (");
                self.field(id, "test");
                self.write(") ");
                self.field(id, "consequent");
                if self.store.child(id, "alternate").is_some() {
                    self.write(" else ");
                    self.field(id, "alternate");
                }
            }
            "LabeledStatement" => {
                self.field(id, "label");
                self.write(": ");
                self.field(id, "body");
            }
            "BreakStatement" | "ContinueStatement" => {
                self.write(if self.kind(id).starts_with('B') {
                    "break"
                } else {
                    "continue"
                });
                if self.store.child(id, "label").is_some() {
                    self.write(" ");
                    self.field(id, "label");
                }
                self.write(";");
            }
            "WithStatement" => {
                self.write("with (");
                self.field(id, "object");
                self.write(") ");
                self.field(id, "body");
            }
            "SwitchStatement" => self.switch(id),
            "ReturnStatement" => {
                self.write("return");
                if self.store.child(id, "argument").is_some() {
                    self.write(" ");
                    self.field(id, "argument");
                }
                self.write(";");
            }
            "ThrowStatement" => {
                self.write("throw ");
                self.field(id, "argument");
                self.write(";");
            }
            "TryStatement" => self.try_statement(id),
            "WhileStatement" => {
                self.write("while (");
                self.field(id, "test");
                self.write(") ");
                self.field(id, "body");
            }
            "DoWhileStatement" => {
                self.write("do ");
                self.field(id, "body");
                self.write(" while (");
                self.field(id, "test");
                self.write(");");
            }
            "ForStatement" => self.for_statement(id),
            "ForInStatement" | "ForOfStatement" => self.for_in(id),
            "DebuggerStatement" => self.write("debugger;"),
            "FunctionDeclaration" | "FunctionExpression" => self.function(id),
            "VariableDeclaration" => {
                self.variable_declaration(id);
                self.write(";");
            }
            "VariableDeclarator" => {
                self.field(id, "id");
                if self.store.child(id, "init").is_some() {
                    self.write(" = ");
                    self.field(id, "init");
                }
            }
            "ClassDeclaration" | "ClassExpression" => self.class(id),
            "ImportDeclaration" => self.import(id),
            "ImportAttribute" => self.import_attribute(id),
            "ExportDefaultDeclaration" => {
                self.write("export default ");
                self.field(id, "declaration");
                let kind = self.kind_of(id, "declaration");
                if precedence_of(kind).is_some() && !kind.starts_with('F') {
                    self.write(";");
                }
            }
            "ExportNamedDeclaration" => self.export_named(id),
            "ExportAllDeclaration" => {
                if self.store.child(id, "exported").is_some() {
                    let line =
                        format!("export * as {} from ", self.name_of(id, "exported"));
                    self.write(&line);
                } else {
                    self.write("export * from ");
                }
                self.field(id, "source");
                self.attributes(id);
                self.write(";");
            }
            "MethodDefinition" => self.method(id),
            "PropertyDefinition" => self.property_definition(id),
            _ => return false,
        }
        true
    }

    fn program(&mut self, id: Id) {
        let indent = self.indent(self.level);
        self.comments_of(id, "comments", &indent);
        for statement in self.store.children(id, "body") {
            self.comments_of(statement, "comments", &indent);
            self.write(&indent);
            self.node(statement);
            self.write("\n");
        }
        self.comments_of(id, "trailingComments", &indent);
    }

    fn block(&mut self, id: Id) {
        let indent = self.indent(self.level);
        self.level += 1;
        let statement_indent = self.indent(self.level);
        self.write("{");
        let statements = self.store.children(id, "body");
        if !statements.is_empty() {
            self.write("\n");
            self.comments_of(id, "comments", &statement_indent);
            for statement in statements {
                self.comments_of(statement, "comments", &statement_indent);
                self.write(&statement_indent);
                self.node(statement);
                self.write("\n");
            }
            self.write(&indent);
        } else if self.store.has(id, "comments") {
            self.write("\n");
            self.comments_of(id, "comments", &statement_indent);
            self.write(&indent);
        }
        self.comments_of(id, "trailingComments", &statement_indent);
        self.write("}");
        self.level -= 1;
    }

    fn expression_statement(&mut self, id: Id) {
        let Some(expression) = self.store.child(id, "expression") else {
            self.write(";");
            return;
        };
        let precedence = self.precedence(expression);
        let left_object = self
            .store
            .child(expression, "left")
            .is_some_and(|left| self.kind(left).starts_with('O'));
        if precedence == Some(NEEDS_PARENTHESES) || (precedence == Some(3) && left_object)
        {
            self.write("(");
            self.node(expression);
            self.write(")");
        } else {
            self.node(expression);
        }
        self.write(";");
    }

    fn switch(&mut self, id: Id) {
        let indent = self.indent(self.level);
        self.level += 2;
        let case_indent = self.indent(self.level - 1);
        let statement_indent = self.indent(self.level);
        self.write("switch (");
        self.field(id, "discriminant");
        self.write(") {\n");
        for occurrence in self.store.children(id, "cases") {
            self.comments_of(occurrence, "comments", &case_indent);
            if self.store.child(occurrence, "test").is_some() {
                let line = format!("{case_indent}case ");
                self.write(&line);
                self.field(occurrence, "test");
                self.write(":\n");
            } else {
                let line = format!("{case_indent}default:\n");
                self.write(&line);
            }
            for statement in self.store.children(occurrence, "consequent") {
                self.comments_of(statement, "comments", &statement_indent);
                self.write(&statement_indent);
                self.node(statement);
                self.write("\n");
            }
        }
        self.level -= 2;
        let end = format!("{indent}}}");
        self.write(&end);
    }

    fn try_statement(&mut self, id: Id) {
        self.write("try ");
        self.field(id, "block");
        if let Some(handler) = self.store.child(id, "handler") {
            if self.store.child(handler, "param").is_none() {
                self.write(" catch ");
            } else {
                self.write(" catch (");
                self.field(handler, "param");
                self.write(") ");
            }
            self.field(handler, "body");
        }
        if self.store.child(id, "finalizer").is_some() {
            self.write(" finally ");
            self.field(id, "finalizer");
        }
    }

    fn for_statement(&mut self, id: Id) {
        self.write("for (");
        if let Some(init) = self.store.child(id, "init") {
            if self.kind(init).starts_with('V') {
                self.variable_declaration(init);
            } else {
                self.node(init);
            }
        }
        self.write("; ");
        self.field(id, "test");
        self.write("; ");
        self.field(id, "update");
        self.write(") ");
        self.field(id, "body");
    }

    fn for_in(&mut self, id: Id) {
        self.write(if self.store.flag(id, "await") {
            "for await ("
        } else {
            "for ("
        });
        if let Some(left) = self.store.child(id, "left") {
            if self.kind(left).starts_with('V') {
                self.variable_declaration(left);
            } else {
                self.node(left);
            }
        }
        self.write(if self.kind(id) == "ForInStatement" {
            " in "
        } else {
            " of "
        });
        self.field(id, "right");
        self.write(") ");
        self.field(id, "body");
    }

    pub(crate) fn function(&mut self, id: Id) {
        let mut head = String::new();
        if self.store.flag(id, "async") {
            head.push_str("async ");
        }
        head.push_str(if self.store.flag(id, "generator") {
            "function* "
        } else {
            "function "
        });
        if let Some(name) = self.store.child(id, "id") {
            head.push_str(self.store.str(name, "name").unwrap_or("undefined"));
        }
        self.write(&head);
        let params = self.store.list(id, "params").to_vec();
        self.sequence(&params);
        self.write(" ");
        self.field(id, "body");
    }

    fn variable_declaration(&mut self, id: Id) {
        let kind = format!("{} ", self.store.str(id, "kind").unwrap_or("var"));
        self.write(&kind);
        for (index, declarator) in self
            .store
            .children(id, "declarations")
            .into_iter()
            .enumerate()
        {
            if index > 0 {
                self.write(", ");
            }
            self.node(declarator);
        }
    }

    fn class(&mut self, id: Id) {
        match self.store.child(id, "id") {
            Some(name) => {
                let head = format!(
                    "class {} ",
                    self.store.str(name, "name").unwrap_or("undefined")
                );
                self.write(&head);
            }
            None => self.write("class "),
        }
        if let Some(super_class) = self.store.child(id, "superClass") {
            self.write("extends ");
            let precedence = self.precedence(super_class);
            if self.kind(super_class) != "ClassExpression"
                && (precedence == Some(NEEDS_PARENTHESES)
                    || below(precedence, Some(NEEDS_PARENTHESES)))
            {
                self.write("(");
                self.node(super_class);
                self.write(")");
            } else {
                self.node(super_class);
            }
            self.write(" ");
        }
        self.field(id, "body");
    }

    fn import(&mut self, id: Id) {
        self.write("import ");
        let specifiers = self.store.children(id, "specifiers");
        let mut index = 0;
        if !specifiers.is_empty() {
            while index < specifiers.len() {
                if index > 0 {
                    self.write(", ");
                }
                let specifier = specifiers[index];
                match self.kind(specifier) {
                    "ImportDefaultSpecifier" => {
                        let name = self.name_of(specifier, "local");
                        self.write(&name);
                    }
                    "ImportNamespaceSpecifier" => {
                        let name = format!("* as {}", self.name_of(specifier, "local"));
                        self.write(&name);
                    }
                    _ => break,
                }
                index += 1;
            }
            if index < specifiers.len() {
                self.write("{");
                loop {
                    let specifier = specifiers[index];
                    let name = self.name_of(specifier, "imported");
                    self.write(&name);
                    let local = self.name_of(specifier, "local");
                    if name != local {
                        self.write(&format!(" as {local}"));
                    }
                    index += 1;
                    if index < specifiers.len() {
                        self.write(", ");
                    } else {
                        break;
                    }
                }
                self.write("}");
            }
            self.write(" from ");
        }
        self.field(id, "source");
        self.attributes(id);
        self.write(";");
    }

    fn attributes(&mut self, id: Id) {
        let attributes = self.store.children(id, "attributes");
        if attributes.is_empty() {
            return;
        }
        self.write(" with { ");
        let count = attributes.len();
        for (index, attribute) in attributes.into_iter().enumerate() {
            self.import_attribute(attribute);
            if index + 1 < count {
                self.write(", ");
            }
        }
        self.write(" }");
    }

    fn import_attribute(&mut self, id: Id) {
        let key = self.name_of(id, "key");
        self.write(&key);
        self.write(": ");
        self.field(id, "value");
    }

    fn export_named(&mut self, id: Id) {
        self.write("export ");
        if self.store.child(id, "declaration").is_some() {
            self.field(id, "declaration");
            return;
        }
        self.write("{");
        let specifiers = self.store.children(id, "specifiers");
        let count = specifiers.len();
        for (index, specifier) in specifiers.into_iter().enumerate() {
            let name = self.name_of(specifier, "local");
            self.write(&name);
            let exported = self.name_of(specifier, "exported");
            if name != exported {
                self.write(&format!(" as {exported}"));
            }
            if index + 1 < count {
                self.write(", ");
            }
        }
        self.write("}");
        if self.store.child(id, "source").is_some() {
            self.write(" from ");
            self.field(id, "source");
        }
        self.attributes(id);
        self.write(";");
    }

    pub(crate) fn method(&mut self, id: Id) {
        if self.store.flag(id, "static") {
            self.write("static ");
        }
        let kind = self.store.str(id, "kind").unwrap_or_default().to_string();
        if kind.starts_with('g') || kind.starts_with('s') {
            self.write(&format!("{kind} "));
        }
        let value = self.store.child(id, "value");
        if value.is_some_and(|value| self.store.flag(value, "async")) {
            self.write("async ");
        }
        if value.is_some_and(|value| self.store.flag(value, "generator")) {
            self.write("*");
        }
        if self.store.flag(id, "computed") {
            self.write("[");
            self.field(id, "key");
            self.write("]");
        } else {
            self.field(id, "key");
        }
        if let Some(value) = value {
            let params = self.store.list(value, "params").to_vec();
            self.sequence(&params);
            self.write(" ");
            self.field(value, "body");
        }
    }

    fn property_definition(&mut self, id: Id) {
        if self.store.flag(id, "static") {
            self.write("static ");
        }
        let computed = self.store.flag(id, "computed");
        if computed {
            self.write("[");
        }
        self.field(id, "key");
        if computed {
            self.write("]");
        }
        if matches!(self.store.get(id, "value"), None | Some(Value::Null)) {
            if !self.kind_of(id, "key").starts_with('F') {
                self.write(";");
            }
            return;
        }
        self.write(" = ");
        self.field(id, "value");
        self.write(";");
    }
}
