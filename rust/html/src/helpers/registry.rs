use std::sync::OnceLock;

use giallo::Registry;

// Decoded once per process: the built-in grammars and themes are a 1 MB compressed dump
static REGISTRY: OnceLock<Option<Registry>> = OnceLock::new();

// Each scope a Shiki 4.5.0 grammar names in `injectTo`, and the language it is; not
// `text.pug`, where giallo applies Vue's injected interpolations unlike Shiki and loses pug's
pub(crate) const INJECTION_TARGETS: [(&str, &str); 6] = [
    ("source.js", "javascript"),
    ("source.ts", "typescript"),
    ("source.ts.ng", "angular-ts"),
    ("text.html.derivative", "html-derivative"),
    ("text.html.derivative.ng", "angular-html"),
    ("text.html.markdown", "markdown"),
];

pub(crate) fn load_registry() -> Option<&'static Registry> {
    let registry = REGISTRY.get_or_init(|| {
        let mut registry = Registry::builtin().ok()?;
        // giallo 0.5.2 looks `injectTo` scopes up among language names, so injected
        // grammars (Angular blocks, `/* css */` templates) never run without these aliases
        for (scope, language) in INJECTION_TARGETS {
            registry.add_alias(language, scope);
        }
        registry.link_grammars();
        Some(registry)
    });
    registry.as_ref()
}
