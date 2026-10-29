use serde::Deserialize;

/// Options of `prose.contractions`, as retext-contractions takes them.
#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
pub struct ContractionsOptions {
    /// Also check contractions written as literals, such as `“hasnt”`.
    #[serde(deserialize_with = "read_flag")]
    #[cfg_attr(feature = "schema", schemars(with = "Option<bool>"))]
    pub allow_literals: bool,
    /// Expect straight apostrophes (`'`) instead of smart ones (`’`).
    #[serde(deserialize_with = "read_flag")]
    #[cfg_attr(feature = "schema", schemars(with = "Option<bool>"))]
    pub straight: bool,
}

fn read_flag<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<bool, D::Error> {
    Ok(Option::<bool>::deserialize(deserializer)?.unwrap_or(false))
}
