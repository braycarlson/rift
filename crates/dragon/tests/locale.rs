use std::str::FromStr;

use dragon::Locale;

#[test]
fn as_str_round_trips_through_from_str() {
    for locale in Locale::ALL {
        let parsed = Locale::from_str(locale.as_str()).expect("round trip");

        assert!(parsed == locale, "round trip mismatch for {locale}");
    }
}

#[test]
fn from_str_is_case_insensitive() {
    assert!(
        Locale::from_str("EN_us") == Ok(Locale::EnUs),
        "parse must ignore case"
    );
    assert!(Locale::from_str("xx_YY").is_err(), "unknown must error");
}
