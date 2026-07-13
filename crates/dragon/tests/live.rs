use dragon::{DragonApi, DragonApiConfig, Error, Locale};

const CHAMPION_ID_AATROX: &str = "Aatrox";
const CHAMPION_KEY_AATROX: &str = "266";
const CHAMPION_SPELL_COUNT: usize = 4;
const ITEM_ID_BOOTS: &str = "1001";
const LANGUAGE: &str = "en_US";
const RUNE_TREE_COUNT: usize = 5;
const SUMMONER_SPELL_ID_FLASH: &str = "SummonerFlash";
const VERSION_UNKNOWN: &str = "0.0.0";

fn api_new() -> DragonApi {
    DragonApi::new(DragonApiConfig::new()).expect("client construction must succeed")
}

async fn version_latest_fetch(api: &DragonApi) -> String {
    api.version_latest_fetch()
        .await
        .expect("latest version fetch must succeed")
}

#[tokio::test]
async fn versions_return_latest_first() {
    let api = api_new();

    let versions = api
        .versions_fetch()
        .await
        .expect("versions fetch must succeed");

    assert!(!versions.is_empty(), "versions must not be empty");

    let latest = &versions[0];

    assert!(!latest.is_empty(), "latest version must not be empty");
    assert!(
        latest.chars().next().is_some_and(|c| c.is_ascii_digit()),
        "latest version must start with a digit: {latest}"
    );
}

#[tokio::test]
async fn languages_contain_english() {
    let api = api_new();

    let languages = api
        .languages_fetch()
        .await
        .expect("languages fetch must succeed");

    assert!(!languages.is_empty(), "languages must not be empty");
    assert!(
        languages.iter().any(|language| language == LANGUAGE),
        "languages must contain {LANGUAGE}"
    );
}

#[tokio::test]
async fn champions_contain_aatrox() {
    let api = api_new();
    let version = version_latest_fetch(&api).await;

    let champions = api
        .champions_fetch(&version, LANGUAGE)
        .await
        .expect("champions fetch must succeed");

    assert!(!champions.data.is_empty(), "champions must not be empty");

    let aatrox = champions
        .data
        .get(CHAMPION_ID_AATROX)
        .expect("Aatrox must exist");

    assert!(
        aatrox.key.as_ref() == CHAMPION_KEY_AATROX,
        "key must be 266"
    );
    assert!(!aatrox.name.is_empty(), "name must not be empty");
    assert!(!aatrox.image.full.is_empty(), "image must not be empty");
}

#[tokio::test]
async fn items_contain_boots() {
    let api = api_new();
    let version = version_latest_fetch(&api).await;

    let items = api
        .items_fetch(&version, LANGUAGE)
        .await
        .expect("items fetch must succeed");

    assert!(!items.data.is_empty(), "items must not be empty");

    let boots = items.data.get(ITEM_ID_BOOTS).expect("Boots must exist");

    assert!(boots.gold.total > 0, "gold total must be positive");
    assert!(!boots.name.is_empty(), "name must not be empty");
    assert!(!boots.image.full.is_empty(), "image must not be empty");
}

#[tokio::test]
async fn items_unknown_version_returns_not_found() {
    let api = api_new();

    let result = api.items_fetch(VERSION_UNKNOWN, LANGUAGE).await;

    assert!(
        matches!(result, Err(Error::NotFound { .. })),
        "expected NotFound, got {result:?}"
    );
}

#[tokio::test]
async fn runes_contain_five_trees() {
    let api = api_new();
    let version = version_latest_fetch(&api).await;

    let trees = api
        .runes_fetch(&version, LANGUAGE)
        .await
        .expect("runes fetch must succeed");

    assert!(trees.len() == RUNE_TREE_COUNT, "must have five rune trees");

    for tree in &trees {
        assert!(!tree.slots.is_empty(), "tree slots must not be empty");
        assert!(!tree.icon.is_empty(), "tree icon must not be empty");
    }
}

#[tokio::test]
async fn summoner_spells_contain_flash() {
    let api = api_new();
    let version = version_latest_fetch(&api).await;

    let spells = api
        .summoner_spells_fetch(&version, LANGUAGE)
        .await
        .expect("summoner spells fetch must succeed");

    assert!(!spells.data.is_empty(), "spells must not be empty");

    let flash = spells
        .data
        .get(SUMMONER_SPELL_ID_FLASH)
        .expect("Flash must exist");

    assert!(!flash.name.is_empty(), "name must not be empty");
    assert!(!flash.modes.is_empty(), "modes must not be empty");
}

#[tokio::test]
async fn profile_icons_deserialize() {
    let api = api_new();
    let version = version_latest_fetch(&api).await;

    let icons = api
        .profile_icons_fetch(&version, LANGUAGE)
        .await
        .expect("profile icons fetch must succeed");

    assert!(!icons.data.is_empty(), "icons must not be empty");
    assert!(!icons.version.is_empty(), "version must not be empty");
}

#[tokio::test]
async fn champion_detail_has_spells() {
    let api = api_new();
    let version = version_latest_fetch(&api).await;

    let file = api
        .champion_fetch(&version, Locale::EnUs.as_str(), CHAMPION_ID_AATROX)
        .await
        .expect("champion detail fetch must succeed");

    let aatrox = file
        .data
        .get(CHAMPION_ID_AATROX)
        .expect("Aatrox detail must exist");

    assert!(
        aatrox.key.as_ref() == CHAMPION_KEY_AATROX,
        "key must be 266"
    );
    assert!(
        aatrox.spells.len() == CHAMPION_SPELL_COUNT,
        "a champion has four spells"
    );
    assert!(!aatrox.passive.name.is_empty(), "passive must have a name");
    assert!(!aatrox.lore.is_empty(), "lore must not be empty");
}

#[tokio::test]
async fn realms_expose_current_version() {
    let api = api_new();

    let realm = api
        .realms_fetch("na")
        .await
        .expect("realms fetch must succeed");

    assert!(!realm.version.is_empty(), "realm version must not be empty");
    assert!(
        realm
            .version
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_digit()),
        "realm version must start with a digit: {}",
        realm.version
    );
}

#[tokio::test]
async fn champion_unknown_id_returns_not_found() {
    let api = api_new();
    let version = version_latest_fetch(&api).await;

    let result = api
        .champion_fetch(&version, LANGUAGE, "NotARealChampion")
        .await;

    assert!(
        matches!(result, Err(Error::NotFound { .. })),
        "expected NotFound, got {result:?}"
    );
}

#[tokio::test]
async fn realms_unknown_region_returns_not_found() {
    let api = api_new();

    let result = api.realms_fetch("zz").await;

    assert!(
        matches!(result, Err(Error::NotFound { .. })),
        "expected NotFound, got {result:?}"
    );
}

#[tokio::test]
async fn repeated_versions_fetch_stays_consistent() {
    let api = api_new();

    let first = api
        .versions_fetch()
        .await
        .expect("first versions fetch must succeed");
    let second = api
        .versions_fetch()
        .await
        .expect("second versions fetch must succeed");

    assert!(
        first == second,
        "repeated versions fetch must return the same list"
    );
}
