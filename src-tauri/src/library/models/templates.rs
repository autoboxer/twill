use serde::{Deserialize, Serialize};

pub const TEMPLATE_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TemplateField {
    Title,
    Prompt,
    Answer,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TemplateAlignment {
    Left,
    Center,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum TemplateBlock {
    Field { field: TemplateField },
    Text { text: String },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VisualTemplateSide {
    pub blocks: Vec<TemplateBlock>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VisualTemplateAppearance {
    pub alignment: TemplateAlignment,
    pub show_field_labels: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VisualTemplate {
    pub front: VisualTemplateSide,
    pub answer: VisualTemplateSide,
    pub appearance: VisualTemplateAppearance,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CustomTemplate {
    pub front_html: String,
    pub answer_html: String,
    pub css: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TemplateMode {
    Visual,
    Custom,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TemplateContent {
    pub schema_version: u32,
    pub mode: TemplateMode,
    pub visual: VisualTemplate,
    pub custom: CustomTemplate,
}

impl Default for TemplateContent {
    fn default() -> Self {
        Self {
            schema_version: TEMPLATE_SCHEMA_VERSION,
            mode: TemplateMode::Visual,
            visual: VisualTemplate {
                front: VisualTemplateSide {
                    blocks: vec![TemplateBlock::Field {
                        field: TemplateField::Prompt,
                    }],
                },
                answer: VisualTemplateSide {
                    blocks: vec![
                        TemplateBlock::Field {
                            field: TemplateField::Prompt,
                        },
                        TemplateBlock::Field {
                            field: TemplateField::Answer,
                        },
                    ],
                },
                appearance: VisualTemplateAppearance {
                    alignment: TemplateAlignment::Left,
                    show_field_labels: true,
                },
            },
            custom: CustomTemplate {
                front_html: concat!("<main class=\"card\">\n", "  {{ prompt }}\n", "</main>",)
                    .to_owned(),
                answer_html: concat!(
                    "<main class=\"card\">\n",
                    "  <section class=\"prompt\">{{ prompt }}</section>\n",
                    "  <hr>\n",
                    "  <section class=\"answer\">{{ answer }}</section>\n",
                    "</main>",
                )
                .to_owned(),
                css: concat!(
                    ".card {\n",
                    "  max-width: 42rem;\n",
                    "  margin: 0 auto;\n",
                    "  color: var(--twill-text-highlighted);\n",
                    "  font-family: var(--twill-reading-font);\n",
                    "  line-height: 1.6;\n",
                    "}\n\n",
                    "hr {\n",
                    "  margin: 2rem 0;\n",
                    "  border: 0;\n",
                    "  border-top: 1px solid var(--twill-border);\n",
                    "}",
                )
                .to_owned(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateSummary {
    pub id: String,
    pub name: String,
    pub updated_at: i64,
    pub mode: TemplateMode,
    pub retrieval_form_count: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateDetail {
    pub id: String,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_change_id: String,
    pub content: TemplateContent,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateCatalog {
    pub templates: Vec<TemplateSummary>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateTemplateInput {
    pub name: String,
    pub content: TemplateContent,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateTemplateInput {
    pub id: String,
    pub name: String,
    pub content: TemplateContent,
}
