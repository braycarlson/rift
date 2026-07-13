use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::Context;
use serde::Deserialize;

use crate::spec::{OpenApi, PathItem, SchemaObject};

const OVERLAY_BYTES_MAX: u64 = 1024 * 1024;
const OVERLAY_ENTRY_COUNT_MAX: u32 = OVERLAY_FILE_COUNT_MAX * 64;
const OVERLAY_FILE_COUNT_MAX: u32 = 32;

#[derive(Deserialize)]
struct Overlay {
    #[serde(default)]
    components: OverlayComponents,
    #[serde(default)]
    paths: BTreeMap<String, PathItem>,
}

#[derive(Default, Deserialize)]
struct OverlayComponents {
    #[serde(default)]
    schemas: BTreeMap<String, SchemaObject>,
}

pub fn overlays_apply(open_api: &mut OpenApi, spec_manual_directory: &Path) -> anyhow::Result<()> {
    if !spec_manual_directory.is_dir() {
        return Ok(());
    }

    let file_paths = overlay_paths_collect(spec_manual_directory)?;
    let mut iterations: u32 = 0;

    for file_path in &file_paths {
        iterations += 1;

        assert!(
            iterations <= OVERLAY_FILE_COUNT_MAX,
            "overlay file count exceeds {OVERLAY_FILE_COUNT_MAX}"
        );

        let body = fs::read_to_string(file_path)
            .with_context(|| format!("overlay read failed: {}", file_path.display()))?;

        assert!(
            body.len() as u64 <= OVERLAY_BYTES_MAX,
            "overlay file exceeds {OVERLAY_BYTES_MAX} bytes: {}",
            file_path.display(),
        );

        let overlay = serde_json::from_str::<Overlay>(&body)
            .with_context(|| format!("overlay parse failed: {}", file_path.display()))?;

        overlay_merge(open_api, overlay);

        println!("applied overlay {}", file_path.display());
    }

    Ok(())
}

fn overlay_paths_collect(spec_manual_directory: &Path) -> anyhow::Result<Vec<std::path::PathBuf>> {
    let entries = fs::read_dir(spec_manual_directory).with_context(|| {
        format!(
            "overlay directory read failed: {}",
            spec_manual_directory.display()
        )
    })?;

    let mut file_paths = Vec::with_capacity(OVERLAY_FILE_COUNT_MAX as usize);
    let mut iterations: u32 = 0;

    for entry in entries {
        iterations += 1;

        assert!(
            iterations <= OVERLAY_FILE_COUNT_MAX,
            "overlay directory entry count exceeds {OVERLAY_FILE_COUNT_MAX}"
        );

        let path = entry?.path();

        let is_json = path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("json"));

        if is_json {
            file_paths.push(path);
        }
    }

    file_paths.sort();

    assert!(
        file_paths.len() <= OVERLAY_FILE_COUNT_MAX as usize,
        "overlay file count exceeds {OVERLAY_FILE_COUNT_MAX}"
    );

    Ok(file_paths)
}

fn overlay_merge(open_api: &mut OpenApi, overlay: Overlay) {
    map_merge(&mut open_api.paths, overlay.paths, "path");

    map_merge(
        &mut open_api.components.schemas,
        overlay.components.schemas,
        "schema",
    );
}

fn map_merge<V>(base: &mut BTreeMap<String, V>, extra: BTreeMap<String, V>, kind: &str) {
    let mut iterations: u32 = 0;

    for (key, value) in extra {
        iterations += 1;

        assert!(
            iterations <= OVERLAY_ENTRY_COUNT_MAX,
            "overlay {kind} entry count exceeds {OVERLAY_ENTRY_COUNT_MAX}"
        );
        assert!(
            !base.contains_key(&key),
            "overlay {kind} collides with existing: {key}"
        );

        base.insert(key, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_merge_inserts_new_keys() {
        let mut base: BTreeMap<String, u8> = BTreeMap::new();

        base.insert("a".to_string(), 1);

        let mut extra: BTreeMap<String, u8> = BTreeMap::new();

        extra.insert("b".to_string(), 2);

        map_merge(&mut base, extra, "test");

        assert!(base.len() == 2, "both keys must be present");
        assert!(base["b"] == 2, "new value must be inserted");
    }

    #[test]
    #[should_panic(expected = "collides with existing")]
    fn map_merge_rejects_collision() {
        let mut base: BTreeMap<String, u8> = BTreeMap::new();

        base.insert("a".to_string(), 1);

        let mut extra: BTreeMap<String, u8> = BTreeMap::new();

        extra.insert("a".to_string(), 2);

        map_merge(&mut base, extra, "test");
    }
}
