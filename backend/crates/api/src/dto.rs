//! API mirrors of domain enums. The domain stays free of OpenAPI concerns; these
//! add `ToSchema` and convert both ways. Wire names are the same snake_case strings.

use domain::game::LevelId;
use domain::grammar::{Case, Locale, Number};

macro_rules! mirror_enum {
    ($(#[$meta:meta])* $name:ident <=> $domain:ident { $($variant:ident),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq,
            ::serde::Serialize, ::serde::Deserialize, ::utoipa::ToSchema,
        )]
        #[serde(rename_all = "snake_case")]
        pub enum $name {
            $($variant),+
        }

        impl From<$name> for $domain {
            fn from(value: $name) -> Self {
                match value {
                    $($name::$variant => $domain::$variant),+
                }
            }
        }

        impl From<$domain> for $name {
            fn from(value: $domain) -> Self {
                match value {
                    $($domain::$variant => $name::$variant),+
                }
            }
        }
    };
}

mirror_enum! {
    UiLocale <=> Locale { En, Fr, Ru }
}

mirror_enum! {
    GrammaticalCase <=> Case {
        Nominative, Genitive, Dative, Accusative, Instrumental, Prepositional,
    }
}

mirror_enum! {
    GrammaticalNumber <=> Number { Singular, Plural }
}

mirror_enum! {
    /// A difficulty level of the games.
    Level <=> LevelId { BasicNouns, IntermediateNouns, AdvancedNouns }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_names_match_the_domain() {
        // The domain's string form is what the database and the frontend see too.
        for &level in LevelId::ALL {
            let json = serde_json::to_string(&Level::from(level)).unwrap();
            assert_eq!(json, format!("\"{}\"", level.as_str()));
        }
        for &case in Case::ALL {
            let json = serde_json::to_string(&GrammaticalCase::from(case)).unwrap();
            assert_eq!(json, format!("\"{}\"", case.as_str()));
        }
    }
}
