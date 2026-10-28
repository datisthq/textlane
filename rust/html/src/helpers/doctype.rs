use crate::models::token::DoctypeToken;
use crate::utils::builder::dom::DocumentMode;

const QUIRKS_PUBLIC_ID_PREFIXES: [&str; 55] = [
    "+//silmaril//dtd html pro v0r11 19970101//",
    "-//as//dtd html 3.0 aswedit + extensions//",
    "-//advasoft ltd//dtd html 3.0 aswedit + extensions//",
    "-//ietf//dtd html 2.0 level 1//",
    "-//ietf//dtd html 2.0 level 2//",
    "-//ietf//dtd html 2.0 strict level 1//",
    "-//ietf//dtd html 2.0 strict level 2//",
    "-//ietf//dtd html 2.0 strict//",
    "-//ietf//dtd html 2.0//",
    "-//ietf//dtd html 2.1e//",
    "-//ietf//dtd html 3.0//",
    "-//ietf//dtd html 3.2 final//",
    "-//ietf//dtd html 3.2//",
    "-//ietf//dtd html 3//",
    "-//ietf//dtd html level 0//",
    "-//ietf//dtd html level 1//",
    "-//ietf//dtd html level 2//",
    "-//ietf//dtd html level 3//",
    "-//ietf//dtd html strict level 0//",
    "-//ietf//dtd html strict level 1//",
    "-//ietf//dtd html strict level 2//",
    "-//ietf//dtd html strict level 3//",
    "-//ietf//dtd html strict//",
    "-//ietf//dtd html//",
    "-//metrius//dtd metrius presentational//",
    "-//microsoft//dtd internet explorer 2.0 html strict//",
    "-//microsoft//dtd internet explorer 2.0 html//",
    "-//microsoft//dtd internet explorer 2.0 tables//",
    "-//microsoft//dtd internet explorer 3.0 html strict//",
    "-//microsoft//dtd internet explorer 3.0 html//",
    "-//microsoft//dtd internet explorer 3.0 tables//",
    "-//netscape comm. corp.//dtd html//",
    "-//netscape comm. corp.//dtd strict html//",
    "-//o'reilly and associates//dtd html 2.0//",
    "-//o'reilly and associates//dtd html extended 1.0//",
    "-//o'reilly and associates//dtd html extended relaxed 1.0//",
    "-//sq//dtd html 2.0 hotmetal + extensions//",
    "-//softquad software//dtd hotmetal pro 6.0::19990601::extensions to html 4.0//",
    "-//softquad//dtd hotmetal pro 4.0::19971010::extensions to html 4.0//",
    "-//spyglass//dtd html 2.0 extended//",
    "-//sun microsystems corp.//dtd hotjava html//",
    "-//sun microsystems corp.//dtd hotjava strict html//",
    "-//w3c//dtd html 3 1995-03-24//",
    "-//w3c//dtd html 3.2 draft//",
    "-//w3c//dtd html 3.2 final//",
    "-//w3c//dtd html 3.2//",
    "-//w3c//dtd html 3.2s draft//",
    "-//w3c//dtd html 4.0 frameset//",
    "-//w3c//dtd html 4.0 transitional//",
    "-//w3c//dtd html experimental 19960712//",
    "-//w3c//dtd html experimental 970421//",
    "-//w3c//dtd w3 html//",
    "-//w3o//dtd w3 html 3.0//",
    "-//webtechs//dtd mozilla html 2.0//",
    "-//webtechs//dtd mozilla html//",
];

// Quirky without a system identifier only
const QUIRKS_NO_SYSTEM_ID_PUBLIC_ID_PREFIXES: [&str; 2] = [
    "-//w3c//dtd html 4.01 frameset//",
    "-//w3c//dtd html 4.01 transitional//",
];

const QUIRKS_PUBLIC_IDS: [&str; 3] = [
    "-//w3o//dtd w3 html strict 3.0//en//",
    "-/w3c/dtd html 4.0 transitional/en",
    "html",
];

const LIMITED_QUIRKS_PUBLIC_ID_PREFIXES: [&str; 2] = [
    "-//w3c//dtd xhtml 1.0 frameset//",
    "-//w3c//dtd xhtml 1.0 transitional//",
];

/// Whether a doctype is `<!doctype html>`, optionally with the legacy-compat system id.
pub(crate) fn is_conforming(token: &DoctypeToken) -> bool {
    token.name.as_deref() == Some("html")
        && token.public_id.is_none()
        && token
            .system_id
            .as_deref()
            .is_none_or(|id| id == "about:legacy-compat")
}

/// The document mode a doctype asks for, as parse5's `getDocumentMode`.
pub(crate) fn find_document_mode(token: &DoctypeToken) -> DocumentMode {
    if token.name.as_deref() != Some("html") {
        return DocumentMode::Quirks;
    }
    let system_id = token.system_id.as_deref();
    if system_id.is_some_and(|id| {
        id.to_lowercase() == "http://www.ibm.com/data/dtd/v11/ibmxhtml1-transitional.dtd"
    }) {
        return DocumentMode::Quirks;
    }
    if let Some(public_id) = &token.public_id {
        let public_id = public_id.to_lowercase();
        if QUIRKS_PUBLIC_IDS.contains(&public_id.as_str()) {
            return DocumentMode::Quirks;
        }
        let has_prefix = |prefixes: &[&str]| {
            prefixes.iter().any(|prefix| public_id.starts_with(prefix))
        };
        if has_prefix(&QUIRKS_PUBLIC_ID_PREFIXES)
            || (system_id.is_none()
                && has_prefix(&QUIRKS_NO_SYSTEM_ID_PUBLIC_ID_PREFIXES))
        {
            return DocumentMode::Quirks;
        }
        if has_prefix(&LIMITED_QUIRKS_PUBLIC_ID_PREFIXES)
            || (system_id.is_some()
                && has_prefix(&QUIRKS_NO_SYSTEM_ID_PUBLIC_ID_PREFIXES))
        {
            return DocumentMode::LimitedQuirks;
        }
    }
    DocumentMode::NoQuirks
}
