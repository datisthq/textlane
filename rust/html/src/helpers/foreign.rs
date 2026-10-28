use crate::helpers::tag::{TagId, find_tag_id};
use crate::models::namespace::Namespace;
use crate::models::token::TagToken;

// Lowercase name to SVG name, sorted by the lowercase name
static SVG_ATTRIBUTES: [(&str, &str); 58] = [
    ("attributename", "attributeName"),
    ("attributetype", "attributeType"),
    ("basefrequency", "baseFrequency"),
    ("baseprofile", "baseProfile"),
    ("calcmode", "calcMode"),
    ("clippathunits", "clipPathUnits"),
    ("diffuseconstant", "diffuseConstant"),
    ("edgemode", "edgeMode"),
    ("filterunits", "filterUnits"),
    ("glyphref", "glyphRef"),
    ("gradienttransform", "gradientTransform"),
    ("gradientunits", "gradientUnits"),
    ("kernelmatrix", "kernelMatrix"),
    ("kernelunitlength", "kernelUnitLength"),
    ("keypoints", "keyPoints"),
    ("keysplines", "keySplines"),
    ("keytimes", "keyTimes"),
    ("lengthadjust", "lengthAdjust"),
    ("limitingconeangle", "limitingConeAngle"),
    ("markerheight", "markerHeight"),
    ("markerunits", "markerUnits"),
    ("markerwidth", "markerWidth"),
    ("maskcontentunits", "maskContentUnits"),
    ("maskunits", "maskUnits"),
    ("numoctaves", "numOctaves"),
    ("pathlength", "pathLength"),
    ("patterncontentunits", "patternContentUnits"),
    ("patterntransform", "patternTransform"),
    ("patternunits", "patternUnits"),
    ("pointsatx", "pointsAtX"),
    ("pointsaty", "pointsAtY"),
    ("pointsatz", "pointsAtZ"),
    ("preservealpha", "preserveAlpha"),
    ("preserveaspectratio", "preserveAspectRatio"),
    ("primitiveunits", "primitiveUnits"),
    ("refx", "refX"),
    ("refy", "refY"),
    ("repeatcount", "repeatCount"),
    ("repeatdur", "repeatDur"),
    ("requiredextensions", "requiredExtensions"),
    ("requiredfeatures", "requiredFeatures"),
    ("specularconstant", "specularConstant"),
    ("specularexponent", "specularExponent"),
    ("spreadmethod", "spreadMethod"),
    ("startoffset", "startOffset"),
    ("stddeviation", "stdDeviation"),
    ("stitchtiles", "stitchTiles"),
    ("surfacescale", "surfaceScale"),
    ("systemlanguage", "systemLanguage"),
    ("tablevalues", "tableValues"),
    ("targetx", "targetX"),
    ("targety", "targetY"),
    ("textlength", "textLength"),
    ("viewbox", "viewBox"),
    ("viewtarget", "viewTarget"),
    ("xchannelselector", "xChannelSelector"),
    ("ychannelselector", "yChannelSelector"),
    ("zoomandpan", "zoomAndPan"),
];

static SVG_TAG_NAMES: [(&str, &str); 36] = [
    ("altglyph", "altGlyph"),
    ("altglyphdef", "altGlyphDef"),
    ("altglyphitem", "altGlyphItem"),
    ("animatecolor", "animateColor"),
    ("animatemotion", "animateMotion"),
    ("animatetransform", "animateTransform"),
    ("clippath", "clipPath"),
    ("feblend", "feBlend"),
    ("fecolormatrix", "feColorMatrix"),
    ("fecomponenttransfer", "feComponentTransfer"),
    ("fecomposite", "feComposite"),
    ("feconvolvematrix", "feConvolveMatrix"),
    ("fediffuselighting", "feDiffuseLighting"),
    ("fedisplacementmap", "feDisplacementMap"),
    ("fedistantlight", "feDistantLight"),
    ("feflood", "feFlood"),
    ("fefunca", "feFuncA"),
    ("fefuncb", "feFuncB"),
    ("fefuncg", "feFuncG"),
    ("fefuncr", "feFuncR"),
    ("fegaussianblur", "feGaussianBlur"),
    ("feimage", "feImage"),
    ("femerge", "feMerge"),
    ("femergenode", "feMergeNode"),
    ("femorphology", "feMorphology"),
    ("feoffset", "feOffset"),
    ("fepointlight", "fePointLight"),
    ("fespecularlighting", "feSpecularLighting"),
    ("fespotlight", "feSpotLight"),
    ("fetile", "feTile"),
    ("feturbulence", "feTurbulence"),
    ("foreignobject", "foreignObject"),
    ("glyphref", "glyphRef"),
    ("lineargradient", "linearGradient"),
    ("radialgradient", "radialGradient"),
    ("textpath", "textPath"),
];

// Attribute name to prefix, local name and namespace
static XML_ATTRIBUTES: [(&str, &str, &str, Namespace); 11] = [
    ("xlink:actuate", "xlink", "actuate", Namespace::Xlink),
    ("xlink:arcrole", "xlink", "arcrole", Namespace::Xlink),
    ("xlink:href", "xlink", "href", Namespace::Xlink),
    ("xlink:role", "xlink", "role", Namespace::Xlink),
    ("xlink:show", "xlink", "show", Namespace::Xlink),
    ("xlink:title", "xlink", "title", Namespace::Xlink),
    ("xlink:type", "xlink", "type", Namespace::Xlink),
    ("xml:lang", "xml", "lang", Namespace::Xml),
    ("xml:space", "xml", "space", Namespace::Xml),
    ("xmlns", "", "xmlns", Namespace::Xmlns),
    ("xmlns:xlink", "xmlns", "xlink", Namespace::Xmlns),
];

/// Whether a start tag in foreign content leaves it, as parse5's `causesExit`.
pub(crate) fn causes_exit(token: &TagToken) -> bool {
    let is_font_with_attributes = token.tag_id == TagId::Font
        && token.attrs.iter().any(|attribute| {
            matches!(attribute.name.as_str(), "color" | "size" | "face")
        });
    is_font_with_attributes
        || matches!(
            token.tag_id,
            TagId::B
                | TagId::Big
                | TagId::Blockquote
                | TagId::Body
                | TagId::Br
                | TagId::Center
                | TagId::Code
                | TagId::Dd
                | TagId::Div
                | TagId::Dl
                | TagId::Dt
                | TagId::Em
                | TagId::Embed
                | TagId::H1
                | TagId::H2
                | TagId::H3
                | TagId::H4
                | TagId::H5
                | TagId::H6
                | TagId::Head
                | TagId::Hr
                | TagId::I
                | TagId::Img
                | TagId::Li
                | TagId::Listing
                | TagId::Menu
                | TagId::Meta
                | TagId::Nobr
                | TagId::Ol
                | TagId::P
                | TagId::Pre
                | TagId::Ruby
                | TagId::S
                | TagId::Small
                | TagId::Span
                | TagId::Strong
                | TagId::Strike
                | TagId::Sub
                | TagId::Sup
                | TagId::Table
                | TagId::Tt
                | TagId::U
                | TagId::Ul
                | TagId::Var
        )
}

/// Spell `definitionurl` as MathML does.
pub(crate) fn adjust_mathml_attributes(token: &mut TagToken) {
    if let Some(attribute) = token
        .attrs
        .iter_mut()
        .find(|attribute| attribute.name == "definitionurl")
    {
        attribute.name = "definitionURL".to_string();
    }
}

/// Spell SVG attributes in their camel case.
pub(crate) fn adjust_svg_attributes(token: &mut TagToken) {
    for attribute in &mut token.attrs {
        if let Ok(index) = SVG_ATTRIBUTES
            .binary_search_by(|(each, _)| each.cmp(&attribute.name.as_str()))
        {
            attribute.name = SVG_ATTRIBUTES[index].1.to_string();
        }
    }
}

/// Split `xlink:`, `xml:` and `xmlns` attributes into prefix, name and namespace.
pub(crate) fn adjust_xml_attributes(token: &mut TagToken) {
    for attribute in &mut token.attrs {
        if let Some((_, prefix, name, namespace)) = XML_ATTRIBUTES
            .iter()
            .find(|(each, ..)| *each == attribute.name)
        {
            attribute.prefix = Some(prefix.to_string());
            attribute.name = name.to_string();
            attribute.namespace = Some(*namespace);
        }
    }
}

/// Spell an SVG tag name in its camel case.
pub(crate) fn adjust_svg_tag_name(token: &mut TagToken) {
    if let Ok(index) =
        SVG_TAG_NAMES.binary_search_by(|(each, _)| each.cmp(&token.tag_name.as_str()))
    {
        token.tag_name = SVG_TAG_NAMES[index].1.to_string();
        token.tag_id = find_tag_id(&token.tag_name);
    }
}

fn is_mathml_text_integration_point(tag: TagId, namespace: Namespace) -> bool {
    namespace == Namespace::MathMl
        && matches!(
            tag,
            TagId::Mi | TagId::Mo | TagId::Mn | TagId::Ms | TagId::Mtext
        )
}

fn is_html_integration_point(
    tag: TagId,
    namespace: Namespace,
    attributes: &[(&str, &str)],
) -> bool {
    if namespace == Namespace::MathMl && tag == TagId::AnnotationXml {
        // Only the first `encoding` counts, as parse5 returns on it
        if let Some((_, value)) = attributes.iter().find(|(name, _)| *name == "encoding")
        {
            let value = value.to_ascii_lowercase();
            return value == "text/html" || value == "application/xhtml+xml";
        }
    }
    namespace == Namespace::Svg
        && matches!(tag, TagId::ForeignObject | TagId::Desc | TagId::Title)
}

/// Whether an element is an integration point, of the kind `foreign` names if any.
pub(crate) fn is_integration_point(
    tag: TagId,
    namespace: Namespace,
    attributes: &[(&str, &str)],
    foreign: Option<Namespace>,
) -> bool {
    (foreign.is_none_or(|foreign| foreign == Namespace::Html)
        && is_html_integration_point(tag, namespace, attributes))
        || (foreign.is_none_or(|foreign| foreign == Namespace::MathMl)
            && is_mathml_text_integration_point(tag, namespace))
}
