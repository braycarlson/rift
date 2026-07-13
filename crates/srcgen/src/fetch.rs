use std::fs;
use std::path::Path;

use anyhow::Context;

const RESPONSE_BYTES_MAX: u64 = 64 * 1024 * 1024;
const SPEC_FILE_COUNT_MAX: u32 = 32;

struct SpecFile {
    name_local: &'static str,
    url: &'static str,
}

const SPEC_FILES: [SpecFile; 9] = [
    SpecFile {
        name_local: "champion-summary.json",
        url: "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-summary.json",
    },
    SpecFile {
        name_local: "gameModes.json",
        url: "https://www.mingweisamuel.com/riotapi-schema/enums/gameModes.json",
    },
    SpecFile {
        name_local: "gameTypes.json",
        url: "https://www.mingweisamuel.com/riotapi-schema/enums/gameTypes.json",
    },
    SpecFile {
        name_local: "maps.json",
        url: "https://www.mingweisamuel.com/riotapi-schema/enums/maps.json",
    },
    SpecFile {
        name_local: "openapi.json",
        url: "https://www.mingweisamuel.com/riotapi-schema/openapi-3.0.0.json",
    },
    SpecFile {
        name_local: "queueTypes.json",
        url: "https://www.mingweisamuel.com/riotapi-schema/enums/queueTypes.json",
    },
    SpecFile {
        name_local: "queues.json",
        url: "https://www.mingweisamuel.com/riotapi-schema/enums/queues.json",
    },
    SpecFile {
        name_local: "routesTable.json",
        url: "https://www.mingweisamuel.com/riotapi-schema/routesTable.json",
    },
    SpecFile {
        name_local: "seasons.json",
        url: "https://www.mingweisamuel.com/riotapi-schema/enums/seasons.json",
    },
];

const _: () = assert!(SPEC_FILES.len() <= SPEC_FILE_COUNT_MAX as usize);

pub fn spec_files_fetch(spec_directory: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(spec_directory)
        .with_context(|| format!("directory creation failed: {}", spec_directory.display()))?;

    assert!(
        spec_directory.is_dir(),
        "spec directory must exist: {}",
        spec_directory.display()
    );

    let mut iterations: u32 = 0;

    for spec_file in &SPEC_FILES {
        iterations += 1;

        assert!(
            iterations <= SPEC_FILE_COUNT_MAX,
            "loop exceeded {SPEC_FILE_COUNT_MAX} iterations",
        );

        let body = spec_file_fetch(spec_file.url)?;
        let path_target = spec_directory.join(spec_file.name_local);

        fs::write(&path_target, &body)
            .with_context(|| format!("write failed: {}", path_target.display()))?;

        assert!(
            path_target.is_file(),
            "spec file must exist after write: {}",
            path_target.display(),
        );

        println!("fetched {}", path_target.display());
    }

    assert!(
        iterations as usize == SPEC_FILES.len(),
        "all spec files must be fetched"
    );

    Ok(())
}

fn spec_file_fetch(url: &str) -> anyhow::Result<String> {
    assert!(!url.is_empty(), "url must not be empty");
    assert!(url.starts_with("https://"), "url must use https: {url}");

    let mut response = ureq::get(url)
        .call()
        .with_context(|| format!("request failed: {url}"))?;

    let body = response
        .body_mut()
        .with_config()
        .limit(RESPONSE_BYTES_MAX)
        .read_to_string()
        .with_context(|| format!("body read failed: {url}"))?;

    assert!(!body.is_empty(), "body must not be empty: {url}");

    let _ = serde_json::from_str::<serde_json::Value>(&body)
        .with_context(|| format!("response is not valid JSON: {url}"))?;

    Ok(body)
}
