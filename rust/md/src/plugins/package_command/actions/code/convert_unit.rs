use textlane_core::Allocator;

use super::*;

#[test]
fn stays_linear_on_a_huge_block() -> Result<(), String> {
    let alloc = Allocator::new();
    let value = "npm install --save-dev a b c\npip install -U x\n".repeat(50_000);
    let code = Code {
        lang: Some("npm"),
        meta: None,
        value: &value,
        position: None,
        data: None,
    };
    let start = std::time::Instant::now();
    let node = convert_code(&PackageCommand::default(), code, &alloc);
    let Node::MdxJsxFlowElement(element) = node else {
        return Err("expected an element".to_owned());
    };
    assert_eq!(element.attributes.len(), 4);
    assert!(start.elapsed().as_secs() < 10);
    Ok(())
}

#[test]
fn leaves_other_languages_code() {
    let alloc = Allocator::new();
    let code = Code {
        lang: Some("sh"),
        meta: None,
        value: "npm i",
        position: None,
        data: None,
    };
    let node = convert_code(&PackageCommand::default(), code, &alloc);
    assert!(matches!(node, Node::Code(_)));
}
