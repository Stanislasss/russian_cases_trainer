use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown {kind} `{value}`")]
pub struct ParseEnumError {
    kind: &'static str,
    value: String,
}

/// Defines a fieldless enum whose string form (serde, `Display`, `FromStr`, `as_str`)
/// is the same snake_case name, which is also what the database stores.
macro_rules! string_enum {
    ($(#[$meta:meta])* $name:ident, $kind:literal { $($variant:ident => $text:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum $name {
            $(#[serde(rename = $text)] $variant),+
        }

        impl $name {
            pub const ALL: &'static [$name] = &[$($name::$variant),+];

            pub const fn as_str(self) -> &'static str {
                match self {
                    $($name::$variant => $text),+
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl FromStr for $name {
            type Err = ParseEnumError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($text => Ok($name::$variant),)+
                    _ => Err(ParseEnumError { kind: $kind, value: s.to_owned() }),
                }
            }
        }
    };
}

string_enum! {
    /// The six Russian grammatical cases.
    Case, "case" {
        Nominative => "nominative",
        Genitive => "genitive",
        Dative => "dative",
        Accusative => "accusative",
        Instrumental => "instrumental",
        Prepositional => "prepositional",
    }
}

string_enum! {
    Number, "number" {
        Singular => "singular",
        Plural => "plural",
    }
}

string_enum! {
    Gender, "gender" {
        Masculine => "masculine",
        Feminine => "feminine",
        Neuter => "neuter",
    }
}

string_enum! {
    Animacy, "animacy" {
        Animate => "animate",
        Inanimate => "inanimate",
    }
}

string_enum! {
    PartOfSpeech, "part of speech" {
        Noun => "noun",
        Adjective => "adjective",
        Pronoun => "pronoun",
    }
}

string_enum! {
    /// Languages the UI, rules and translations are available in.
    Locale, "locale" {
        En => "en",
        Fr => "fr",
        Ru => "ru",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_form_round_trips() {
        for &case in Case::ALL {
            assert_eq!(case.as_str().parse::<Case>(), Ok(case));
        }
        assert_eq!("fr".parse::<Locale>(), Ok(Locale::Fr));
    }

    #[test]
    fn serde_uses_the_same_names_as_the_database() {
        assert_eq!(
            serde_json::to_string(&Case::Prepositional).unwrap(),
            r#""prepositional""#
        );
    }

    #[test]
    fn unknown_values_are_rejected() {
        let err = "vocative".parse::<Case>().unwrap_err();
        assert_eq!(err.to_string(), "unknown case `vocative`");
    }
}
