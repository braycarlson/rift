use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::Context;
use serde::Deserialize;
use serde::de::DeserializeOwned;

const SPEC_BYTES_MAX: u64 = 64 * 1024 * 1024;

#[derive(Deserialize)]
pub struct ChampionEntry {
    pub alias: String,
    pub id: i16,
    pub name: String,
}

#[derive(Deserialize)]
pub struct Components {
    pub schemas: BTreeMap<String, SchemaObject>,
}

#[derive(Deserialize)]
pub struct ExternalDocs {
    pub url: String,
}

#[derive(Deserialize)]
pub struct MediaType {
    pub schema: SchemaObject,
}

#[derive(Deserialize)]
pub struct NumericEntry<T> {
    #[serde(rename = "x-name")]
    pub name: String,
    #[serde(rename = "x-value")]
    pub value: T,
}

#[derive(Deserialize)]
pub struct OpenApi {
    pub components: Components,
    pub paths: BTreeMap<String, PathItem>,
}

#[derive(Deserialize)]
pub struct Operation {
    pub description: Option<String>,
    #[serde(rename = "externalDocs")]
    pub external_docs: Option<ExternalDocs>,
    #[serde(rename = "operationId")]
    pub id: String,
    #[serde(rename = "x-nullable-404", default)]
    pub nullable_404: bool,
    #[serde(default)]
    pub parameters: Vec<Parameter>,
    #[serde(rename = "requestBody")]
    pub request_body: Option<RequestBody>,
    pub responses: BTreeMap<String, ResponseObject>,
    #[serde(rename = "x-route-enum")]
    pub route_enum: String,
    pub summary: Option<String>,
}

#[derive(Deserialize)]
pub struct Parameter {
    pub description: Option<String>,
    #[serde(rename = "in")]
    pub location: String,
    pub name: String,
    #[serde(default)]
    pub required: bool,
    pub schema: SchemaObject,
}

#[derive(Deserialize)]
pub struct PathItem {
    pub delete: Option<Operation>,
    pub get: Option<Operation>,
    pub patch: Option<Operation>,
    pub post: Option<Operation>,
    pub put: Option<Operation>,
}

#[derive(Deserialize)]
pub struct QueueTypeEntry {
    #[serde(rename = "x-name")]
    pub name: String,
    #[serde(rename = "queueType")]
    pub queue_type: String,
}

#[derive(Deserialize)]
pub struct RequestBody {
    pub content: BTreeMap<String, MediaType>,
}

#[derive(Deserialize)]
pub struct ResponseObject {
    pub content: Option<BTreeMap<String, MediaType>>,
}

#[derive(Deserialize)]
pub struct RouteEntry {
    pub id: u8,
    #[serde(rename = "regionalRoute")]
    pub regional_route: Option<String>,
}

#[derive(Deserialize)]
pub struct RoutesTable {
    pub platform: BTreeMap<String, RouteEntry>,
    pub regional: BTreeMap<String, RouteEntry>,
    #[serde(rename = "val-platform")]
    pub val_platform: BTreeMap<String, RouteEntry>,
}

#[derive(Deserialize)]
pub struct SchemaObject {
    #[serde(rename = "additionalProperties")]
    pub additional_properties: Option<Box<SchemaObject>>,
    pub description: Option<String>,
    pub format: Option<String>,
    pub items: Option<Box<SchemaObject>>,
    pub properties: Option<BTreeMap<String, SchemaObject>>,
    #[serde(rename = "$ref")]
    pub reference: Option<String>,
    pub required: Option<Vec<String>>,
    #[serde(rename = "type")]
    pub type_name: Option<String>,
}

#[derive(Deserialize)]
pub struct StringEntry {
    #[serde(rename = "x-name")]
    pub name: String,
    #[serde(rename = "x-value")]
    pub value: String,
}

pub fn spec_load<T: DeserializeOwned>(spec_directory: &Path, file_name: &str) -> anyhow::Result<T> {
    assert!(!file_name.is_empty(), "file name must not be empty");
    assert!(
        Path::new(file_name)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("json")),
        "file name must end with .json: {file_name}"
    );

    let path = spec_directory.join(file_name);
    let body =
        fs::read_to_string(&path).with_context(|| format!("read failed: {}", path.display()))?;

    assert!(!body.is_empty(), "spec file must not be empty: {file_name}");
    assert!(
        body.len() as u64 <= SPEC_BYTES_MAX,
        "spec file exceeds {SPEC_BYTES_MAX} bytes: {file_name}"
    );

    let value = serde_json::from_str::<T>(&body)
        .with_context(|| format!("parse failed: {}", path.display()))?;

    Ok(value)
}
