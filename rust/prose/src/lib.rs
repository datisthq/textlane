//! Textlane's prose plugins: parsing natural language into paragraphs, sentences and words,
//! serializing it, and the checks that read it.

mod helpers {
    pub(crate) mod abbreviation;
    pub(crate) mod article_table;
    pub(crate) mod classify;
    pub(crate) mod contraction_table;
    pub(crate) mod equality_table;
    pub(crate) mod literal;
    pub(crate) mod node;
    pub(crate) mod normalize;
    pub(crate) mod number;
    pub(crate) mod passive_table;
    pub(crate) mod place;
    pub(crate) mod readability_table;
    pub(crate) mod unicode;
    pub(crate) mod words;
}
mod models {
    pub(crate) mod contractions;
    pub(crate) mod equality;
    pub(crate) mod options;
    pub(crate) mod parser;
    pub(crate) mod phrases;
    pub(crate) mod quotes;
    pub(crate) mod report;
    pub(crate) mod smartypants;
}
mod utils {
    pub(crate) mod parser {
        pub(crate) mod cursor;
        pub(crate) mod document;
        pub(crate) mod paragraph;
        pub(crate) mod sentence;
        pub(crate) mod store;
        pub(crate) mod tokenize;
    }
}
mod actions {
    pub(crate) mod article {
        pub(crate) mod check;
    }
    pub(crate) mod contractions {
        pub(crate) mod check;
    }
    pub(crate) mod equality {
        pub(crate) mod check;
    }
    pub(crate) mod message {
        pub(crate) mod place;
    }
    pub(crate) mod nlcst {
        pub(crate) mod search;
        pub(crate) mod stringify;
        pub(crate) mod visit;
    }
    pub(crate) mod passive {
        pub(crate) mod check;
    }
    pub(crate) mod prose {
        pub(crate) mod parse;
    }
    pub(crate) mod punctuation {
        pub(crate) mod smarten;
    }
    pub(crate) mod quotes {
        pub(crate) mod check;
    }
    pub(crate) mod readability {
        pub(crate) mod check;
    }
    pub(crate) mod repeated_words {
        pub(crate) mod check;
    }
    pub(crate) mod sentence_spacing {
        pub(crate) mod check;
    }
    pub(crate) mod syllable {
        pub(crate) mod count;
    }
    pub(crate) mod word {
        pub(crate) mod singularize;
    }
}
mod plugins {
    pub(crate) mod contractions {
        pub(crate) mod plugin;
    }
    pub(crate) mod equality {
        pub(crate) mod plugin;
    }
    pub(crate) mod indefinite_article {
        pub(crate) mod plugin;
    }
    pub(crate) mod parse {
        pub(crate) mod plugin;
    }
    pub(crate) mod passive {
        pub(crate) mod plugin;
    }
    pub(crate) mod quotes {
        pub(crate) mod plugin;
    }
    pub(crate) mod readability {
        pub(crate) mod plugin;
    }
    pub(crate) mod repeated_words {
        pub(crate) mod plugin;
    }
    pub(crate) mod sentence_spacing {
        pub(crate) mod plugin;
    }
    pub(crate) mod smartypants {
        pub(crate) mod plugin;
    }
    pub(crate) mod stringify {
        pub(crate) mod plugin;
    }
}

pub use actions::nlcst::stringify::stringify_nlcst;
pub use actions::prose::parse::parse_prose;
pub use models::contractions::Contractions;
pub use models::options::{
    Equality, Language, Parse, Passive, Readability, SentenceSpacing, Spacing,
};
pub use models::parser::LanguageParser;
pub use models::quotes::{QuoteStyle, Quotes};
pub use models::smartypants::{
    Backticks, Dashes, Ellipses, QuoteCharacters, Smartypants,
};
pub use plugins::indefinite_article::plugin::IndefiniteArticle;
pub use plugins::repeated_words::plugin::RepeatedWords;
pub use plugins::stringify::plugin::Stringify;
