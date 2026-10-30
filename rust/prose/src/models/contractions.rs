use serde::Deserialize;

/// `prose.contractions`: check apostrophes in contractions, such as `don’t`; options as
/// retext-contractions takes them.
#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "ContractionsOptions"))]
pub struct Contractions {
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
