use std::str::FromStr;

/// A Data Dragon locale, such as `en_US`.
///
/// Mirrors the routing-value pattern used elsewhere: [`Locale::as_str`] yields
/// the on-the-wire code, [`FromStr`] parses it case-insensitively, and
/// [`Locale::ALL`] lists every variant.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Locale {
    /// Czech (Czechia).
    CsCz,
    /// German (Germany).
    DeDe,
    /// Greek (Greece).
    ElGr,
    /// English (Australia).
    EnAu,
    /// English (United Kingdom).
    EnGb,
    /// English (Philippines).
    EnPh,
    /// English (Singapore).
    EnSg,
    /// English (United States).
    EnUs,
    /// Spanish (Argentina).
    EsAr,
    /// Spanish (Spain).
    EsEs,
    /// Spanish (Mexico).
    EsMx,
    /// French (France).
    FrFr,
    /// Hungarian (Hungary).
    HuHu,
    /// Indonesian (Indonesia).
    IdId,
    /// Italian (Italy).
    ItIt,
    /// Japanese (Japan).
    JaJp,
    /// Korean (Korea).
    KoKr,
    /// Polish (Poland).
    PlPl,
    /// Portuguese (Brazil).
    PtBr,
    /// Romanian (Romania).
    RoRo,
    /// Russian (Russia).
    RuRu,
    /// Thai (Thailand).
    ThTh,
    /// Turkish (Turkey).
    TrTr,
    /// Vietnamese (Vietnam).
    ViVn,
    /// Chinese (Mainland China).
    ZhCn,
    /// Chinese (Malaysia).
    ZhMy,
    /// Chinese (Taiwan).
    ZhTw,
}

impl Locale {
    /// Every locale, in declaration order.
    pub const ALL: [Locale; 27] = [
        Locale::CsCz,
        Locale::DeDe,
        Locale::ElGr,
        Locale::EnAu,
        Locale::EnGb,
        Locale::EnPh,
        Locale::EnSg,
        Locale::EnUs,
        Locale::EsAr,
        Locale::EsEs,
        Locale::EsMx,
        Locale::FrFr,
        Locale::HuHu,
        Locale::IdId,
        Locale::ItIt,
        Locale::JaJp,
        Locale::KoKr,
        Locale::PlPl,
        Locale::PtBr,
        Locale::RoRo,
        Locale::RuRu,
        Locale::ThTh,
        Locale::TrTr,
        Locale::ViVn,
        Locale::ZhCn,
        Locale::ZhMy,
        Locale::ZhTw,
    ];

    /// The Data Dragon locale code, e.g. `"en_US"`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Locale::CsCz => "cs_CZ",
            Locale::DeDe => "de_DE",
            Locale::ElGr => "el_GR",
            Locale::EnAu => "en_AU",
            Locale::EnGb => "en_GB",
            Locale::EnPh => "en_PH",
            Locale::EnSg => "en_SG",
            Locale::EnUs => "en_US",
            Locale::EsAr => "es_AR",
            Locale::EsEs => "es_ES",
            Locale::EsMx => "es_MX",
            Locale::FrFr => "fr_FR",
            Locale::HuHu => "hu_HU",
            Locale::IdId => "id_ID",
            Locale::ItIt => "it_IT",
            Locale::JaJp => "ja_JP",
            Locale::KoKr => "ko_KR",
            Locale::PlPl => "pl_PL",
            Locale::PtBr => "pt_BR",
            Locale::RoRo => "ro_RO",
            Locale::RuRu => "ru_RU",
            Locale::ThTh => "th_TH",
            Locale::TrTr => "tr_TR",
            Locale::ViVn => "vi_VN",
            Locale::ZhCn => "zh_CN",
            Locale::ZhMy => "zh_MY",
            Locale::ZhTw => "zh_TW",
        }
    }
}

/// Error returned when a string does not name a known [`Locale`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LocaleParseError;

impl core::fmt::Display for LocaleParseError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("unrecognized locale")
    }
}

impl std::error::Error for LocaleParseError {}

impl core::fmt::Display for Locale {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for Locale {
    type Err = LocaleParseError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        assert!(!Locale::ALL.is_empty(), "locale table must not be empty");

        let mut index: usize = 0;

        while index < Locale::ALL.len() {
            let locale = Locale::ALL[index];

            if text.eq_ignore_ascii_case(locale.as_str()) {
                return Ok(locale);
            }

            index += 1;
        }

        assert!(
            index == Locale::ALL.len(),
            "scan must cover the full locale table"
        );

        Err(LocaleParseError)
    }
}

impl serde::Serialize for Locale {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for Locale {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = <String as serde::Deserialize>::deserialize(deserializer)?;

        text.parse().map_err(serde::de::Error::custom)
    }
}
