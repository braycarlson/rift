#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Champion {
    pub blurb: Box<str>,
    pub id: Box<str>,
    pub image: Image,
    pub info: ChampionInfo,
    pub key: Box<str>,
    pub name: Box<str>,
    #[serde(rename = "partype")]
    pub par_type: Box<str>,
    pub stats: rustc_hash::FxHashMap<String, f64>,
    pub tags: Box<[String]>,
    pub title: Box<str>,
    pub version: Box<str>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ChampionInfo {
    pub attack: u8,
    pub defense: u8,
    pub difficulty: u8,
    pub magic: u8,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ChampionsFile {
    pub data: rustc_hash::FxHashMap<String, Champion>,
    pub version: Box<str>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Image {
    pub full: Box<str>,
    pub group: Box<str>,
    #[serde(rename = "h")]
    pub height: u16,
    pub sprite: Box<str>,
    #[serde(rename = "w")]
    pub width: u16,
    pub x: u16,
    pub y: u16,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Item {
    #[serde(rename = "from", skip_serializing_if = "Option::is_none")]
    pub builds_from: Option<Box<[String]>>,
    #[serde(rename = "into", skip_serializing_if = "Option::is_none")]
    pub builds_into: Option<Box<[String]>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colloq: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<u8>,
    pub description: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect: Option<rustc_hash::FxHashMap<String, String>>,
    pub gold: ItemGold,
    #[serde(rename = "hideFromAll", skip_serializing_if = "Option::is_none")]
    pub hide_from_all: Option<bool>,
    pub image: Image,
    #[serde(rename = "inStore", skip_serializing_if = "Option::is_none")]
    pub in_store: Option<bool>,
    pub maps: rustc_hash::FxHashMap<String, bool>,
    pub name: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plaintext: Option<Box<str>>,
    #[serde(rename = "requiredAlly", skip_serializing_if = "Option::is_none")]
    pub required_ally: Option<Box<str>>,
    #[serde(rename = "requiredChampion", skip_serializing_if = "Option::is_none")]
    pub required_champion: Option<Box<str>>,
    #[serde(rename = "specialRecipe", skip_serializing_if = "Option::is_none")]
    pub special_recipe: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stacks: Option<u32>,
    pub stats: rustc_hash::FxHashMap<String, f64>,
    pub tags: Box<[String]>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ItemGold {
    pub base: u32,
    pub purchasable: bool,
    pub sell: u32,
    pub total: u32,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ItemsFile {
    pub data: rustc_hash::FxHashMap<String, Item>,
    pub version: Box<str>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ProfileIcon {
    #[serde(deserialize_with = "profile_icon_id_deserialize")]
    pub id: u32,
    pub image: Image,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ProfileIconsFile {
    pub data: rustc_hash::FxHashMap<String, ProfileIcon>,
    pub version: Box<str>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Rune {
    #[serde(rename = "longDesc")]
    pub description_long: Box<str>,
    #[serde(rename = "shortDesc")]
    pub description_short: Box<str>,
    pub icon: Box<str>,
    pub id: u32,
    pub key: Box<str>,
    pub name: Box<str>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct RuneSlot {
    pub runes: Box<[Rune]>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct RuneTree {
    pub icon: Box<str>,
    pub id: u32,
    pub key: Box<str>,
    pub name: Box<str>,
    pub slots: Box<[RuneSlot]>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct SummonerSpell {
    pub cooldown: Box<[f64]>,
    pub description: Box<str>,
    pub id: Box<str>,
    pub image: Image,
    pub key: Box<str>,
    pub modes: Box<[String]>,
    pub name: Box<str>,
    #[serde(rename = "summonerLevel")]
    pub summoner_level: u16,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct SummonerSpellsFile {
    pub data: rustc_hash::FxHashMap<String, SummonerSpell>,
    pub version: Box<str>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Realm {
    #[serde(rename = "cdn")]
    pub cdn_base: Box<str>,
    #[serde(rename = "css")]
    pub css_version: Box<str>,
    #[serde(rename = "dd")]
    pub data_dragon_version: Box<str>,
    #[serde(rename = "l")]
    pub language: Box<str>,
    #[serde(rename = "lg")]
    pub legacy_version: Box<str>,
    #[serde(rename = "profileiconmax")]
    pub profile_icon_id_max: u32,
    #[serde(rename = "n")]
    pub type_versions: rustc_hash::FxHashMap<String, String>,
    #[serde(rename = "v")]
    pub version: Box<str>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ChampionDetailFile {
    pub data: rustc_hash::FxHashMap<String, ChampionDetail>,
    pub version: Box<str>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ChampionDetail {
    #[serde(rename = "allytips", default)]
    pub ally_tips: Box<[String]>,
    pub blurb: Box<str>,
    #[serde(rename = "enemytips", default)]
    pub enemy_tips: Box<[String]>,
    pub id: Box<str>,
    pub image: Image,
    pub info: ChampionInfo,
    pub key: Box<str>,
    pub lore: Box<str>,
    pub name: Box<str>,
    #[serde(rename = "partype")]
    pub par_type: Box<str>,
    pub passive: Passive,
    pub skins: Box<[Skin]>,
    pub spells: Box<[Spell]>,
    pub stats: rustc_hash::FxHashMap<String, f64>,
    pub tags: Box<[String]>,
    pub title: Box<str>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Passive {
    pub description: Box<str>,
    pub image: Image,
    pub name: Box<str>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Skin {
    #[serde(default)]
    pub chromas: bool,
    pub id: Box<str>,
    pub name: Box<str>,
    pub num: u16,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Spell {
    pub cooldown: Box<[f64]>,
    pub cost: Box<[f64]>,
    pub description: Box<str>,
    pub id: Box<str>,
    pub image: Image,
    #[serde(rename = "maxrank", default)]
    pub rank_max: u8,
    pub name: Box<str>,
    pub range: Box<[f64]>,
    pub tooltip: Box<str>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct CDragonChampionSummary {
    pub alias: Box<str>,
    pub id: i32,
    pub name: Box<str>,
    #[serde(default)]
    pub roles: Box<[String]>,
    #[serde(rename = "squarePortraitPath", skip_serializing_if = "Option::is_none")]
    pub square_portrait_path: Option<Box<str>>,
}

fn profile_icon_id_deserialize<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;

    match value {
        serde_json::Value::Number(number) => number
            .as_u64()
            .and_then(|id| u32::try_from(id).ok())
            .ok_or_else(|| serde::de::Error::custom("profile icon id must fit u32")),
        serde_json::Value::String(text) => text
            .parse::<u32>()
            .map_err(|_| serde::de::Error::custom("profile icon id must parse as u32")),
        _ => Err(serde::de::Error::custom(
            "profile icon id must be a number or string",
        )),
    }
}
