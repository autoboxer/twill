use serde::{Deserialize, Serialize};

pub const CSS_SNIPPET_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CssSnippetContent {
    pub schema_version: u32,
    pub source: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssSnippet {
    pub id: String,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub content: CssSnippetContent,
    pub enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssSnippetCatalog {
    pub snippets: Vec<CssSnippet>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateCssSnippetInput {
    pub name: String,
    pub content: CssSnippetContent,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateCssSnippetInput {
    pub id: String,
    pub name: String,
    pub content: CssSnippetContent,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetCssSnippetEnabledInput {
    pub id: String,
    pub enabled: bool,
}
