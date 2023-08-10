//! src/domain/subscriber_name.rs

use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    // I might rename this to from_string
    pub fn parse(input: String) -> Result<SubscriberName, String> {
        let is_empty_or_whitespace = input.trim().is_empty();

        let is_too_long = input.graphemes(true).count() > 256;

        let forbiden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbiden_characters = input
            .chars()
            .any(|grapheme| forbiden_characters.contains(&grapheme));

        if is_empty_or_whitespace || is_too_long || contains_forbiden_characters {
            Err(format!("{input} is not a valid subscriber name."))
        } else {
            Ok(Self(input))
        }
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberName;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "ё".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_rejected() {
        let name = "a".repeat(257);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn white_space_only_names_are_rejected() {
        let name = " ".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn a_valid_name_is_parsed_successfully() {
        let name = "Ursula Le Guin".to_string();
        assert_ok!(SubscriberName::parse(name));
    }
}
