/// The document a pipeline processes: where it came from and what plugins reported about it.
#[derive(Clone, Debug, Default)]
pub struct File {
    pub path: Option<String>,
    pub messages: Vec<Message>,
}

/// A diagnostic a plugin reported, as vfile-message does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    pub reason: String,
    pub source: Option<String>,
    pub rule: Option<String>,
}

impl File {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a warning against the file.
    pub fn warn(&mut self, reason: impl Into<String>, rule: Option<&str>) {
        self.messages.push(Message {
            reason: reason.into(),
            source: None,
            rule: rule.map(str::to_string),
        });
    }
}
