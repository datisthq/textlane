//! Print every plugin's option schema, for `pnpm generate:bindings`.

fn main() {
    println!("{:#}", textlane::describe_plugins());
}
