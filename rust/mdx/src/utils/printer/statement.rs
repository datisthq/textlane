use crate::models::estree::{Id, Value};
use crate::models::name::{Key, Kind};
use crate::utils::printer::precedence::{NEEDS_PARENTHESES, below, precedence_of};
use crate::utils::printer::state::Printer;

impl Printer<'_> {
    /// astring's statement, declaration and module handlers; `false` for other kinds.
    pub(crate) fn statement(&mut self, id: Id) -> bool {
        match self.kind(id) {
            Kind::PROGRAM => self.program(id),
            Kind::BLOCK_STATEMENT | Kind::CLASS_BODY => self.block(id),
            Kind::STATIC_BLOCK => {
                self.write("static ");
                self.block(id);
            }
            Kind::EMPTY_STATEMENT => self.write(";"),
            Kind::EXPRESSION_STATEMENT => self.expression_statement(id),
            Kind::IF_STATEMENT => {
                self.write("if (");
                self.field(id, Key::TEST);
                self.write(") ");
                self.field(id, Key::CONSEQUENT);
                if self.store.child(id, Key::ALTERNATE).is_some() {
                    self.write(" else ");
                    self.field(id, Key::ALTERNATE);
                }
            }
            Kind::LABELED_STATEMENT => {
                self.field(id, Key::LABEL);
                self.write(": ");
                self.field(id, Key::BODY);
            }
            Kind::BREAK_STATEMENT | Kind::CONTINUE_STATEMENT => {
                self.write(if self.kind_text(id).starts_with('B') {
                    "break"
                } else {
                    "continue"
                });
                if self.store.child(id, Key::LABEL).is_some() {
                    self.write(" ");
                    self.field(id, Key::LABEL);
                }
                self.write(";");
            }
            Kind::WITH_STATEMENT => {
                self.write("with (");
                self.field(id, Key::OBJECT);
                self.write(") ");
                self.field(id, Key::BODY);
            }
            Kind::SWITCH_STATEMENT => self.switch(id),
            Kind::RETURN_STATEMENT => {
                self.write("return");
                if self.store.child(id, Key::ARGUMENT).is_some() {
                    self.write(" ");
                    self.field(id, Key::ARGUMENT);
                }
                self.write(";");
            }
            Kind::THROW_STATEMENT => {
                self.write("throw ");
                self.field(id, Key::ARGUMENT);
                self.write(";");
            }
            Kind::TRY_STATEMENT => self.try_statement(id),
            Kind::WHILE_STATEMENT => {
                self.write("while (");
                self.field(id, Key::TEST);
                self.write(") ");
                self.field(id, Key::BODY);
            }
            Kind::DO_WHILE_STATEMENT => {
                self.write("do ");
                self.field(id, Key::BODY);
                self.write(" while (");
                self.field(id, Key::TEST);
                self.write(");");
            }
            Kind::FOR_STATEMENT => self.for_statement(id),
            Kind::FOR_IN_STATEMENT | Kind::FOR_OF_STATEMENT => self.for_in(id),
            Kind::DEBUGGER_STATEMENT => self.write("debugger;"),
            Kind::FUNCTION_DECLARATION | Kind::FUNCTION_EXPRESSION => self.function(id),
            Kind::VARIABLE_DECLARATION => {
                self.variable_declaration(id);
                self.write(";");
            }
            Kind::VARIABLE_DECLARATOR => {
                self.field(id, Key::ID);
                if self.store.child(id, Key::INIT).is_some() {
                    self.write(" = ");
                    self.field(id, Key::INIT);
                }
            }
            Kind::CLASS_DECLARATION | Kind::CLASS_EXPRESSION => self.class(id),
            Kind::IMPORT_DECLARATION => self.import(id),
            Kind::IMPORT_ATTRIBUTE => self.import_attribute(id),
            Kind::EXPORT_DEFAULT_DECLARATION => {
                self.write("export default ");
                self.field(id, Key::DECLARATION);
                let kind = self.kind_of(id, Key::DECLARATION);
                if precedence_of(kind).is_some()
                    && !self.kind_text_of(id, Key::DECLARATION).starts_with('F')
                {
                    self.write(";");
                }
            }
            Kind::EXPORT_NAMED_DECLARATION => self.export_named(id),
            Kind::EXPORT_ALL_DECLARATION => {
                if self.store.child(id, Key::EXPORTED).is_some() {
                    let line =
                        format!("export * as {} from ", self.name_of(id, Key::EXPORTED));
                    self.write(&line);
                } else {
                    self.write("export * from ");
                }
                self.field(id, Key::SOURCE);
                self.attributes(id);
                self.write(";");
            }
            Kind::METHOD_DEFINITION => self.method(id),
            Kind::PROPERTY_DEFINITION => self.property_definition(id),
            _ => return false,
        }
        true
    }

    fn program(&mut self, id: Id) {
        let indent = self.indent(self.level);
        self.comments_of(id, Key::COMMENTS, &indent);
        for statement in self.store.children(id, Key::BODY) {
            self.comments_of(statement, Key::COMMENTS, &indent);
            self.write(&indent);
            self.node(statement);
            self.write("\n");
        }
        self.comments_of(id, Key::TRAILING_COMMENTS, &indent);
    }

    fn block(&mut self, id: Id) {
        let indent = self.indent(self.level);
        self.level += 1;
        let statement_indent = self.indent(self.level);
        self.write("{");
        let statements = self.store.children(id, Key::BODY);
        if !statements.is_empty() {
            self.write("\n");
            self.comments_of(id, Key::COMMENTS, &statement_indent);
            for statement in statements {
                self.comments_of(statement, Key::COMMENTS, &statement_indent);
                self.write(&statement_indent);
                self.node(statement);
                self.write("\n");
            }
            self.write(&indent);
        } else if self.store.has(id, Key::COMMENTS) {
            self.write("\n");
            self.comments_of(id, Key::COMMENTS, &statement_indent);
            self.write(&indent);
        }
        self.comments_of(id, Key::TRAILING_COMMENTS, &statement_indent);
        self.write("}");
        self.level -= 1;
    }

    fn expression_statement(&mut self, id: Id) {
        let Some(expression) = self.store.child(id, Key::EXPRESSION) else {
            self.write(";");
            return;
        };
        let precedence = self.precedence(expression);
        let left_object = self
            .store
            .child(expression, Key::LEFT)
            .is_some_and(|left| self.kind_text(left).starts_with('O'));
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
        self.field(id, Key::DISCRIMINANT);
        self.write(") {\n");
        for occurrence in self.store.children(id, Key::CASES) {
            self.comments_of(occurrence, Key::COMMENTS, &case_indent);
            if self.store.child(occurrence, Key::TEST).is_some() {
                let line = format!("{case_indent}case ");
                self.write(&line);
                self.field(occurrence, Key::TEST);
                self.write(":\n");
            } else {
                let line = format!("{case_indent}default:\n");
                self.write(&line);
            }
            for statement in self.store.children(occurrence, Key::CONSEQUENT) {
                self.comments_of(statement, Key::COMMENTS, &statement_indent);
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
        self.field(id, Key::BLOCK);
        if let Some(handler) = self.store.child(id, Key::HANDLER) {
            if self.store.child(handler, Key::PARAM).is_none() {
                self.write(" catch ");
            } else {
                self.write(" catch (");
                self.field(handler, Key::PARAM);
                self.write(") ");
            }
            self.field(handler, Key::BODY);
        }
        if self.store.child(id, Key::FINALIZER).is_some() {
            self.write(" finally ");
            self.field(id, Key::FINALIZER);
        }
    }

    fn for_statement(&mut self, id: Id) {
        self.write("for (");
        if let Some(init) = self.store.child(id, Key::INIT) {
            if self.kind_text(init).starts_with('V') {
                self.variable_declaration(init);
            } else {
                self.node(init);
            }
        }
        self.write("; ");
        self.field(id, Key::TEST);
        self.write("; ");
        self.field(id, Key::UPDATE);
        self.write(") ");
        self.field(id, Key::BODY);
    }

    fn for_in(&mut self, id: Id) {
        self.write(if self.store.flag(id, Key::AWAIT) {
            "for await ("
        } else {
            "for ("
        });
        if let Some(left) = self.store.child(id, Key::LEFT) {
            if self.kind_text(left).starts_with('V') {
                self.variable_declaration(left);
            } else {
                self.node(left);
            }
        }
        self.write(if self.kind(id) == Kind::FOR_IN_STATEMENT {
            " in "
        } else {
            " of "
        });
        self.field(id, Key::RIGHT);
        self.write(") ");
        self.field(id, Key::BODY);
    }

    pub(crate) fn function(&mut self, id: Id) {
        let mut head = String::new();
        if self.store.flag(id, Key::ASYNC) {
            head.push_str("async ");
        }
        head.push_str(if self.store.flag(id, Key::GENERATOR) {
            "function* "
        } else {
            "function "
        });
        if let Some(name) = self.store.child(id, Key::ID) {
            head.push_str(self.store.str(name, Key::NAME).unwrap_or("undefined"));
        }
        self.write(&head);
        let params = self.store.list(id, Key::PARAMS).to_vec();
        self.sequence(&params);
        self.write(" ");
        self.field(id, Key::BODY);
    }

    fn variable_declaration(&mut self, id: Id) {
        let kind = format!("{} ", self.store.str(id, Key::KIND).unwrap_or("var"));
        self.write(&kind);
        for (index, declarator) in self
            .store
            .children(id, Key::DECLARATIONS)
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
        match self.store.child(id, Key::ID) {
            Some(name) => {
                let head = format!(
                    "class {} ",
                    self.store.str(name, Key::NAME).unwrap_or("undefined")
                );
                self.write(&head);
            }
            None => self.write("class "),
        }
        if let Some(super_class) = self.store.child(id, Key::SUPER_CLASS) {
            self.write("extends ");
            let precedence = self.precedence(super_class);
            if self.kind(super_class) != Kind::CLASS_EXPRESSION
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
        self.field(id, Key::BODY);
    }

    fn import(&mut self, id: Id) {
        self.write("import ");
        let specifiers = self.store.children(id, Key::SPECIFIERS);
        let mut index = 0;
        if !specifiers.is_empty() {
            while index < specifiers.len() {
                if index > 0 {
                    self.write(", ");
                }
                let specifier = specifiers[index];
                match self.kind(specifier) {
                    Kind::IMPORT_DEFAULT_SPECIFIER => {
                        let name = self.name_of(specifier, Key::LOCAL);
                        self.write(&name);
                    }
                    Kind::IMPORT_NAMESPACE_SPECIFIER => {
                        let name =
                            format!("* as {}", self.name_of(specifier, Key::LOCAL));
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
                    let name = self.name_of(specifier, Key::IMPORTED);
                    self.write(&name);
                    let local = self.name_of(specifier, Key::LOCAL);
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
        self.field(id, Key::SOURCE);
        self.attributes(id);
        self.write(";");
    }

    fn attributes(&mut self, id: Id) {
        let attributes = self.store.children(id, Key::ATTRIBUTES);
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
        let key = self.name_of(id, Key::KEY);
        self.write(&key);
        self.write(": ");
        self.field(id, Key::VALUE);
    }

    fn export_named(&mut self, id: Id) {
        self.write("export ");
        if self.store.child(id, Key::DECLARATION).is_some() {
            self.field(id, Key::DECLARATION);
            return;
        }
        self.write("{");
        let specifiers = self.store.children(id, Key::SPECIFIERS);
        let count = specifiers.len();
        for (index, specifier) in specifiers.into_iter().enumerate() {
            let name = self.name_of(specifier, Key::LOCAL);
            self.write(&name);
            let exported = self.name_of(specifier, Key::EXPORTED);
            if name != exported {
                self.write(&format!(" as {exported}"));
            }
            if index + 1 < count {
                self.write(", ");
            }
        }
        self.write("}");
        if self.store.child(id, Key::SOURCE).is_some() {
            self.write(" from ");
            self.field(id, Key::SOURCE);
        }
        self.attributes(id);
        self.write(";");
    }

    pub(crate) fn method(&mut self, id: Id) {
        if self.store.flag(id, Key::STATIC) {
            self.write("static ");
        }
        let kind = self
            .store
            .str(id, Key::KIND)
            .unwrap_or_default()
            .to_string();
        if kind.starts_with('g') || kind.starts_with('s') {
            self.write(&format!("{kind} "));
        }
        let value = self.store.child(id, Key::VALUE);
        if value.is_some_and(|value| self.store.flag(value, Key::ASYNC)) {
            self.write("async ");
        }
        if value.is_some_and(|value| self.store.flag(value, Key::GENERATOR)) {
            self.write("*");
        }
        if self.store.flag(id, Key::COMPUTED) {
            self.write("[");
            self.field(id, Key::KEY);
            self.write("]");
        } else {
            self.field(id, Key::KEY);
        }
        if let Some(value) = value {
            let params = self.store.list(value, Key::PARAMS).to_vec();
            self.sequence(&params);
            self.write(" ");
            self.field(value, Key::BODY);
        }
    }

    fn property_definition(&mut self, id: Id) {
        if self.store.flag(id, Key::STATIC) {
            self.write("static ");
        }
        let computed = self.store.flag(id, Key::COMPUTED);
        if computed {
            self.write("[");
        }
        self.field(id, Key::KEY);
        if computed {
            self.write("]");
        }
        if matches!(self.store.get(id, Key::VALUE), None | Some(Value::Null)) {
            if !self.kind_text_of(id, Key::KEY).starts_with('F') {
                self.write(";");
            }
            return;
        }
        self.write(" = ");
        self.field(id, Key::VALUE);
        self.write(";");
    }
}
