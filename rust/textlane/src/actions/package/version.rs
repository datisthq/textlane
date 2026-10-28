/// The crate's version, which every binding reports as its own.
pub fn read_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
#[path = "version_unit.rs"]
mod unit;
