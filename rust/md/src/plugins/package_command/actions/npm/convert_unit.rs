use super::*;

#[test]
fn keeps_npm_lines_without_an_executor() {
    assert_eq!(convert_npm("npm i yarn", NpmManager::Npm), "npm i yarn");
    assert_eq!(convert_npm("yarn dlx tsc", NpmManager::Npm), "npx tsc");
}

#[test]
fn converts_every_command_in_a_chain_eating_its_trailing_space() {
    assert_eq!(
        convert_npm("npm i && npm run build", NpmManager::Yarn),
        "yarn install&& yarn build"
    );
}

#[test]
fn writes_a_bare_npm_as_upstream_does() {
    assert_eq!(
        convert_npm("npm", NpmManager::Pnpm),
        "npm undefined\n# couldn't auto-convert command"
    );
}

#[test]
fn stays_linear_on_a_long_line() {
    let line = "npm i a &".repeat(200_000);
    let start = std::time::Instant::now();
    let converted = convert_npm(&line, NpmManager::Bun);
    assert!(converted.starts_with("bun add a&bun add a&"));
    assert!(start.elapsed().as_secs() < 5);
}
