mod consts;
mod docs;
mod endpoints;
mod meta;
mod models;
mod overlay;
mod routes;

use std::fs;
use std::path::Path;

use anyhow::Context;

use crate::spec::{
    self, ChampionEntry, NumericEntry, OpenApi, QueueTypeEntry, RoutesTable, StringEntry,
};

const GENERATED_FILE_COUNT: u32 = 6;

const MOD_SOURCE: &str =
    "//! Generated code: routes, models, endpoint methods, constants, and metadata.
//!
//! This module and all of its children are produced by `srcgen` from the Riot
//! API `OpenAPI` specification. Do not edit by hand.
#![allow(clippy::pedantic)]

/// Champion, queue, map, season, and game-mode constants.
pub mod consts;
/// Endpoint methods generated onto [`crate::RiotApi`].
pub mod endpoints;
/// Static metadata describing every generated endpoint.
pub mod meta;
/// Data-transfer objects returned by the Riot API.
pub mod models;
/// Platform, regional, and Valorant routing values.
pub mod routes;
";

pub fn generate_all(
    spec_directory: &Path,
    spec_manual_directory: &Path,
    target_directory: &Path,
) -> anyhow::Result<()> {
    assert!(
        spec_directory.is_dir(),
        "spec directory must exist: {}",
        spec_directory.display()
    );

    let mut open_api: OpenApi = spec::spec_load(spec_directory, "openapi.json")?;

    overlay::overlays_apply(&mut open_api, spec_manual_directory)?;

    let champions: Vec<ChampionEntry> = spec::spec_load(spec_directory, "champion-summary.json")?;
    let game_modes: Vec<StringEntry> = spec::spec_load(spec_directory, "gameModes.json")?;
    let game_types: Vec<StringEntry> = spec::spec_load(spec_directory, "gameTypes.json")?;
    let maps: Vec<NumericEntry<u16>> = spec::spec_load(spec_directory, "maps.json")?;
    let queue_types: Vec<QueueTypeEntry> = spec::spec_load(spec_directory, "queueTypes.json")?;
    let queues: Vec<NumericEntry<u16>> = spec::spec_load(spec_directory, "queues.json")?;
    let routes_table: RoutesTable = spec::spec_load(spec_directory, "routesTable.json")?;
    let seasons: Vec<NumericEntry<u8>> = spec::spec_load(spec_directory, "seasons.json")?;

    assert!(!champions.is_empty(), "champion spec must not be empty");
    assert!(!game_modes.is_empty(), "game mode spec must not be empty");
    assert!(!game_types.is_empty(), "game type spec must not be empty");
    assert!(!maps.is_empty(), "map spec must not be empty");
    assert!(!queue_types.is_empty(), "queue type spec must not be empty");
    assert!(!queues.is_empty(), "queue spec must not be empty");
    assert!(!seasons.is_empty(), "season spec must not be empty");

    let consts_input = consts::ConstsInput {
        champions: &champions,
        game_modes: &game_modes,
        game_types: &game_types,
        maps: &maps,
        queue_types: &queue_types,
        queues: &queues,
        seasons: &seasons,
    };

    let files: [(&str, String); GENERATED_FILE_COUNT as usize] = [
        ("consts.rs", consts::consts_generate(&consts_input)),
        ("endpoints.rs", endpoints::endpoints_generate(&open_api)?),
        ("meta.rs", meta::meta_generate(&open_api)?),
        ("mod.rs", MOD_SOURCE.to_string()),
        (
            "models.rs",
            models::models_generate(&open_api.components.schemas)?,
        ),
        ("routes.rs", routes::routes_generate(&routes_table)?),
    ];

    files_write(target_directory, &files)?;

    Ok(())
}

fn files_write(target_directory: &Path, files: &[(&str, String)]) -> anyhow::Result<()> {
    fs::create_dir_all(target_directory)
        .with_context(|| format!("directory creation failed: {}", target_directory.display()))?;

    assert!(
        target_directory.is_dir(),
        "target directory must exist: {}",
        target_directory.display()
    );

    let mut iterations: u32 = 0;

    for (file_name, source) in files {
        iterations += 1;

        assert!(
            iterations <= GENERATED_FILE_COUNT,
            "file count exceeds {GENERATED_FILE_COUNT}"
        );
        assert!(
            !source.is_empty(),
            "generated source must not be empty: {file_name}"
        );

        let path_target = target_directory.join(file_name);

        fs::write(&path_target, source)
            .with_context(|| format!("write failed: {}", path_target.display()))?;

        assert!(
            path_target.is_file(),
            "generated file must exist after write: {}",
            path_target.display(),
        );

        println!("generated {}", path_target.display());
    }

    assert!(
        iterations == GENERATED_FILE_COUNT,
        "all generated files must be written"
    );

    Ok(())
}
