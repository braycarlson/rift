use std::path::Path;

const HOST_CDRAGON: &str = "https://raw.communitydragon.org";
const HOST_DDRAGON: &str = "https://ddragon.leagueoflegends.com";

const CDRAGON_CHAMPION_SUMMARY_PATH: &str =
    "/plugins/rcp-be-lol-game-data/global/default/v1/champion-summary.json";

const CHAMPION_ID_LENGTH_MAX: usize = 64;
const FILE_NAME_LENGTH_MAX: usize = 64;
const IMAGE_NAME_LENGTH_MAX: usize = 128;
const LANGUAGE_LENGTH_MAX: usize = 16;
const PATH_LENGTH_MAX: usize = 256;
const SKIN_INDEX_MAX: u16 = 1_000;
const VERSION_LENGTH_MAX: usize = 24;

const REGION_LENGTH_MAX: usize = 16;

pub(crate) const LANGUAGES_URL: &str = "https://ddragon.leagueoflegends.com/cdn/languages.json";
pub(crate) const VERSIONS_URL: &str = "https://ddragon.leagueoflegends.com/api/versions.json";

#[must_use]
pub fn realm_url(region: &str) -> String {
    region_assert(region);

    format!("{HOST_DDRAGON}/realms/{region}.json")
}

#[must_use]
pub fn passive_image_url(version: &str, image_name: &str) -> String {
    version_assert(version);
    image_name_assert(image_name);

    format!("{HOST_DDRAGON}/cdn/{version}/img/passive/{image_name}")
}

#[must_use]
pub fn sprite_url(version: &str, sprite_name: &str) -> String {
    version_assert(version);
    image_name_assert(sprite_name);

    format!("{HOST_DDRAGON}/cdn/{version}/img/sprite/{sprite_name}")
}

pub(crate) fn champion_detail_url(version: &str, language: &str, key: &str) -> String {
    version_assert(version);
    language_assert(language);
    assert!(!key.is_empty(), "champion key must not be empty");
    assert!(
        key.len() <= CHAMPION_ID_LENGTH_MAX,
        "champion key exceeds {CHAMPION_ID_LENGTH_MAX} bytes"
    );
    assert!(
        key.chars().all(|c| c.is_ascii_alphanumeric()),
        "champion key must be ascii alphanumeric: {key}"
    );

    format!("{HOST_DDRAGON}/cdn/{version}/data/{language}/champion/{key}.json")
}

pub(crate) fn cdragon_champion_summary_url(patch: &str) -> String {
    version_assert(patch);

    format!("{HOST_CDRAGON}/{patch}{CDRAGON_CHAMPION_SUMMARY_PATH}")
}

#[must_use]
pub fn cdragon_file_url(patch: &str, file_path: &str) -> String {
    version_assert(patch);
    assert!(!file_path.is_empty(), "file_path must not be empty");
    assert!(
        file_path.len() <= PATH_LENGTH_MAX,
        "file_path exceeds {PATH_LENGTH_MAX} bytes"
    );
    assert!(
        file_path.starts_with('/'),
        "file_path must start with '/': {file_path}"
    );

    format!("{HOST_CDRAGON}/{patch}{file_path}")
}

#[must_use]
pub fn champion_loading_url(champion_id: &str, skin_index: u16) -> String {
    champion_id_assert(champion_id);
    assert!(
        skin_index <= SKIN_INDEX_MAX,
        "skin_index exceeds {SKIN_INDEX_MAX}"
    );

    format!("{HOST_DDRAGON}/cdn/img/champion/loading/{champion_id}_{skin_index}.jpg")
}

#[must_use]
pub fn champion_splash_url(champion_id: &str, skin_index: u16) -> String {
    champion_id_assert(champion_id);
    assert!(
        skin_index <= SKIN_INDEX_MAX,
        "skin_index exceeds {SKIN_INDEX_MAX}"
    );

    format!("{HOST_DDRAGON}/cdn/img/champion/splash/{champion_id}_{skin_index}.jpg")
}

#[must_use]
pub fn champion_square_url(version: &str, image_name: &str) -> String {
    version_assert(version);
    image_name_assert(image_name);

    format!("{HOST_DDRAGON}/cdn/{version}/img/champion/{image_name}")
}

pub(crate) fn data_file_url(version: &str, language: &str, file_name: &str) -> String {
    version_assert(version);
    language_assert(language);
    assert!(!file_name.is_empty(), "file_name must not be empty");
    assert!(
        file_name.len() <= FILE_NAME_LENGTH_MAX,
        "file_name exceeds {FILE_NAME_LENGTH_MAX} bytes"
    );
    assert!(
        Path::new(file_name)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("json")),
        "file_name must end with .json: {file_name}"
    );

    format!("{HOST_DDRAGON}/cdn/{version}/data/{language}/{file_name}")
}

#[must_use]
pub fn dragontail_url(version: &str) -> String {
    version_assert(version);
    assert!(
        version.chars().next().is_some_and(|c| c.is_ascii_digit()),
        "dragontail version must start with a digit: {version}"
    );

    format!("{HOST_DDRAGON}/cdn/dragontail-{version}.tgz")
}

#[must_use]
pub fn item_image_url(version: &str, image_name: &str) -> String {
    version_assert(version);
    image_name_assert(image_name);

    format!("{HOST_DDRAGON}/cdn/{version}/img/item/{image_name}")
}

#[must_use]
pub fn profile_icon_url(version: &str, image_name: &str) -> String {
    version_assert(version);
    image_name_assert(image_name);

    format!("{HOST_DDRAGON}/cdn/{version}/img/profileicon/{image_name}")
}

#[must_use]
pub fn rune_icon_url(icon_path: &str) -> String {
    assert!(!icon_path.is_empty(), "icon_path must not be empty");
    assert!(
        icon_path.len() <= PATH_LENGTH_MAX,
        "icon_path exceeds {PATH_LENGTH_MAX} bytes"
    );
    assert!(
        !icon_path.starts_with('/'),
        "icon_path must be relative: {icon_path}"
    );

    format!("{HOST_DDRAGON}/cdn/img/{icon_path}")
}

#[must_use]
pub fn summoner_spell_image_url(version: &str, image_name: &str) -> String {
    version_assert(version);
    image_name_assert(image_name);

    format!("{HOST_DDRAGON}/cdn/{version}/img/spell/{image_name}")
}

fn champion_id_assert(champion_id: &str) {
    assert!(!champion_id.is_empty(), "champion_id must not be empty");
    assert!(
        champion_id.len() <= CHAMPION_ID_LENGTH_MAX,
        "champion_id exceeds {CHAMPION_ID_LENGTH_MAX} bytes"
    );
    assert!(
        champion_id.chars().all(|c| c.is_ascii_alphanumeric()),
        "champion_id must be ascii alphanumeric: {champion_id}"
    );
}

fn image_name_assert(image_name: &str) {
    assert!(!image_name.is_empty(), "image_name must not be empty");
    assert!(
        image_name.len() <= IMAGE_NAME_LENGTH_MAX,
        "image_name exceeds {IMAGE_NAME_LENGTH_MAX} bytes"
    );
    assert!(
        !image_name.contains('/'),
        "image_name must not contain '/': {image_name}"
    );
    assert!(
        image_name.contains('.'),
        "image_name must have an extension: {image_name}"
    );
}

fn region_assert(region: &str) {
    assert!(!region.is_empty(), "region must not be empty");
    assert!(
        region.len() <= REGION_LENGTH_MAX,
        "region exceeds {REGION_LENGTH_MAX} bytes"
    );
    assert!(
        region.chars().all(|c| c.is_ascii_alphanumeric()),
        "region must be ascii alphanumeric: {region}"
    );
}

fn language_assert(language: &str) {
    assert!(!language.is_empty(), "language must not be empty");
    assert!(
        language.len() <= LANGUAGE_LENGTH_MAX,
        "language exceeds {LANGUAGE_LENGTH_MAX} bytes"
    );
    assert!(
        language
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_'),
        "language must be ascii alphanumeric or underscore: {language}"
    );
}

fn version_assert(version: &str) {
    assert!(!version.is_empty(), "version must not be empty");
    assert!(
        version.len() <= VERSION_LENGTH_MAX,
        "version exceeds {VERSION_LENGTH_MAX} bytes"
    );
    assert!(
        version.chars().all(|c| c.is_ascii_graphic() && c != '/'),
        "version must be ascii graphic without '/': {version}"
    );
}
