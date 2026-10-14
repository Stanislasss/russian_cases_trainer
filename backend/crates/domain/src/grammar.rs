use serde::{Deserialize, Serialize};

/// The six Russian grammatical cases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Case {
    Nominative,
    Genitive,
    Dative,
    Accusative,
    Instrumental,
    Prepositional,
}

impl Case {
    pub const ALL: [Case; 6] = [
        Case::Nominative,
        Case::Genitive,
        Case::Dative,
        Case::Accusative,
        Case::Instrumental,
        Case::Prepositional,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Number {
    Singular,
    Plural,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Gender {
    Masculine,
    Feminine,
    Neuter,
}
