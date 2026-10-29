use serde::Deserialize;

/// Options of `mdx.compile`, named as @mdx-js/mdx's `compile` names them.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
pub struct CompileOptions {
    /// Keep JSX in the output instead of compiling it to function calls.
    pub jsx: bool,
    /// The JSX runtime: `automatic` (default) or `classic`.
    pub jsx_runtime: Option<String>,
    /// Where the automatic runtime imports `jsx` and `jsxs` from (default `react`).
    pub jsx_import_source: Option<String>,
    /// The classic runtime's element function (default `React.createElement`).
    pub pragma: Option<String>,
    /// The classic runtime's fragment (default `React.Fragment`).
    pub pragma_frag: Option<String>,
    /// Where the classic runtime imports the pragma's first name from (default `react`).
    pub pragma_import_source: Option<String>,
    /// Where `useMDXComponents` is imported from, to take components from a provider.
    pub provider_import_source: Option<String>,
    /// `program` (default), a module, or `function-body`, the body of a function that gets
    /// the runtime as its first argument.
    pub output_format: Option<String>,
    /// How attribute names of HTML elements are cased: `react` (default) or `html`.
    pub element_attribute_name_case: Option<String>,
    /// How style property names are cased: `dom` (default) or `css`.
    pub style_property_name_case: Option<String>,
    /// Turn `align` on table cells into a `textAlign` style (default `true`).
    pub table_cell_align_to_style: Option<bool>,
}
