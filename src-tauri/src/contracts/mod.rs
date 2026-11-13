use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{atomic::{AtomicU64, Ordering}, Arc, Mutex},
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorCode {
    Cancelled,
    InvalidInput,
    Unsupported,
    TooLarge,
    Io,
    Busy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractError {
    pub code: ErrorCode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ImageResource {
    pub id: String,
    pub mime_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ImageDocument {
    pub id: String,
    pub source_path: Option<String>,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub frame_count: u32,
    pub size_bytes: u64,
    pub preview: ImageResource,
}

/// An immutable original from Viewer, resolved by document ID inside Rust.
pub struct EditorInput {
    pub document: ImageDocument,
    pub original_bytes: Vec<u8>,
    pub source_path: Option<PathBuf>,
}

type PublishedEdit = (String, Arc<Vec<u8>>);

/// Holds the most recent explicit editor export for the downstream consumer.
#[derive(Default)]
pub struct EditedResources {
    latest: Mutex<Option<PublishedEdit>>,
    next: AtomicU64,
}

impl EditedResources {
    pub fn publish_png(&self, bytes: Vec<u8>) -> Result<ImageResource, ContractError> {
        let mut latest = self.latest.lock().map_err(|_| ContractError { code: ErrorCode::Busy })?;
        let sequence = self.next.fetch_add(1, Ordering::Relaxed);
        let id = format!("editor-export:{sequence}");
        *latest = Some((id.clone(), Arc::new(bytes)));
        Ok(ImageResource { id, mime_type: "image/png".into() })
    }

    pub fn read(&self, id: &str) -> Result<Arc<Vec<u8>>, ContractError> {
        let latest = self.latest.lock().map_err(|_| ContractError { code: ErrorCode::Busy })?;
        latest.as_ref().filter(|(key, _)| key == id).map(|(_, bytes)| Arc::clone(bytes))
            .ok_or(ContractError { code: ErrorCode::InvalidInput })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode { Dark, Light, System }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Locale {
    #[serde(rename = "zh-CN")]
    SimplifiedChinese,
    #[serde(rename = "en")]
    English,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edited_export_requires_the_current_resource_id() {
        let exports = EditedResources::default();
        assert!(exports.read("C:\\example.png").is_err());
        let first = exports.publish_png(vec![1]).unwrap();
        let held = exports.read(&first.id).unwrap();
        let second = exports.publish_png(vec![2]).unwrap();
        assert_eq!(second.mime_type, "image/png");
        assert_eq!(exports.read(&second.id).unwrap().as_slice(), &[2]);
        assert!(exports.read(&first.id).is_err());
        assert_eq!(held.as_slice(), &[1]);
    }

    #[test]
    fn document_round_trips_the_shared_wire_fixture() {
        let json: serde_json::Value = serde_json::from_str(include_str!("../../../src/shared/image-document.fixture.json")).unwrap();
        let document: ImageDocument = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(document.frame_count, 1);
        assert_eq!(document.preview.mime_type, "image/jpeg");
        assert_eq!(serde_json::to_value(document).unwrap(), json);
    }

    #[test]
    fn preferences_use_the_same_wire_values_as_the_frontend() {
        assert_eq!(serde_json::to_string(&Locale::SimplifiedChinese).unwrap(), "\"zh-CN\"");
        assert_eq!(serde_json::to_string(&ThemeMode::System).unwrap(), "\"system\"");
        assert_eq!(serde_json::to_string(&ContractError { code: ErrorCode::TooLarge }).unwrap(), "{\"code\":\"too-large\"}");
    }
}
