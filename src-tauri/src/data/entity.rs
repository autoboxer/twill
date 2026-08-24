use crate::data::{DataError, DataResult};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntityKind {
    Concept,
    Card,
    Deck,
    Tag,
    Template,
    Review,
    ReviewReversal,
    Media,
    CssSnippet,
}

impl EntityKind {
    pub const ALL: [Self; 9] = [
        Self::Concept,
        Self::Card,
        Self::Deck,
        Self::Tag,
        Self::Template,
        Self::Review,
        Self::ReviewReversal,
        Self::Media,
        Self::CssSnippet,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Concept => "concept",
            Self::Card => "card",
            Self::Deck => "deck",
            Self::Tag => "tag",
            Self::Template => "template",
            Self::Review => "review",
            Self::ReviewReversal => "review_reversal",
            Self::Media => "media",
            Self::CssSnippet => "css_snippet",
        }
    }
}

impl TryFrom<&str> for EntityKind {
    type Error = DataError;

    fn try_from(value: &str) -> DataResult<Self> {
        match value {
            "concept" => Ok(Self::Concept),
            "card" => Ok(Self::Card),
            "deck" => Ok(Self::Deck),
            "tag" => Ok(Self::Tag),
            "template" => Ok(Self::Template),
            "review" => Ok(Self::Review),
            "review_reversal" => Ok(Self::ReviewReversal),
            "media" => Ok(Self::Media),
            "css_snippet" => Ok(Self::CssSnippet),
            _ => Err(DataError::UnknownEntityKind(value.to_owned())),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChangeOperation {
    Create,
    Update,
    Delete,
}

impl ChangeOperation {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Update => "update",
            Self::Delete => "delete",
        }
    }
}

impl TryFrom<&str> for ChangeOperation {
    type Error = DataError;

    fn try_from(value: &str) -> DataResult<Self> {
        match value {
            "create" => Ok(Self::Create),
            "update" => Ok(Self::Update),
            "delete" => Ok(Self::Delete),
            _ => Err(DataError::UnknownChangeOperation(value.to_owned())),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntityMetadata {
    pub id: String,
    pub kind: EntityKind,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
    pub revision: i64,
    pub last_change_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangeRecord {
    pub sequence: i64,
    pub id: String,
    pub entity_id: String,
    pub entity_kind: EntityKind,
    pub operation: ChangeOperation,
    pub recorded_at: i64,
}
