use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct CreatedByUserResponse {
    pub id: i32,
    pub username: String,
    pub namespace: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ModelVersionResponse {
    pub id: String,
    pub experiment: Option<ExperimentSourceResponse>,
    pub version: u32,
    pub size: u64,
    pub checksum: String,
    pub created_by: CreatedByUserResponse,
    pub created_at: String,
    pub manifest: serde_json::Value,
    /// Opaque, app-defined blob stored with the version — `Null` when the
    /// publisher set none.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ExperimentSourceResponse {
    pub id: i32,
    pub experiment_num: i32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ModelResponse {
    pub id: String,
    pub project_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_by: CreatedByUserResponse,
    pub created_at: String,
    pub version_count: u64,
    /// Highest published version, absent while the model has none.
    #[serde(default)]
    pub latest_version: Option<u32>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ModelListResponse {
    pub items: Vec<ModelResponse>,
    pub total: usize,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ModelVersionListResponse {
    pub items: Vec<ModelVersionResponse>,
    pub total: usize,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ModelDownloadResponse {
    pub files: Vec<PresignedModelFileUrlResponse>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct PresignedModelFileUrlResponse {
    pub rel_path: String,
    pub url: String,
    pub size_bytes: u64,
    pub checksum: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct PresignedModelFileUploadUrlsResponse {
    pub rel_path: String,
    pub urls: crate::artifact::response::MultipartUploadResponse,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RequestModelVersionUploadResponse {
    pub version: u32,
    pub files: Vec<PresignedModelFileUploadUrlsResponse>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_list_matches_backend_contract() {
        let response: ModelListResponse = serde_json::from_str(
            r#"{
                "items": [{
                    "id": "0198f0a1-0000-7000-8000-000000000000",
                    "project_id": 3,
                    "name": "resnet",
                    "description": null,
                    "created_by": {"id": 7, "username": "ada", "namespace": "ada"},
                    "created_at": "2026-03-05 18:45:43.397",
                    "version_count": 2,
                    "latest_version": 2
                }],
                "total": 1
            }"#,
        )
        .unwrap();

        assert_eq!(response.total, 1);
        let model = &response.items[0];
        assert_eq!(model.name, "resnet");
        assert_eq!(model.description, None);
        assert_eq!(model.version_count, 2);
        assert_eq!(model.latest_version, Some(2));
        assert_eq!(model.created_by.username, "ada");
    }

    /// A model without a published version reports a null latest version.
    #[test]
    fn model_without_a_version_has_no_latest_version() {
        let model: ModelResponse = serde_json::from_str(
            r#"{
                "id": "0198f0a1-0000-7000-8000-000000000000",
                "project_id": 3,
                "name": "resnet",
                "description": "a model",
                "created_by": {"id": 7, "username": "ada", "namespace": "ada"},
                "created_at": "2026-03-05 18:45:43.397",
                "version_count": 0,
                "latest_version": null
            }"#,
        )
        .unwrap();

        assert_eq!(model.latest_version, None);
    }

    /// The presigned files carry the size and checksum a download is verified against.
    #[test]
    fn model_download_matches_backend_contract() {
        let response: ModelDownloadResponse = serde_json::from_str(
            r#"{
                "files": [{
                    "rel_path": "model.mpk",
                    "url": "https://blobs.example.com/model.mpk?signature=x",
                    "size_bytes": 1048576,
                    "checksum": "9f86d0818"
                }]
            }"#,
        )
        .unwrap();

        let file = &response.files[0];
        assert_eq!(file.rel_path, "model.mpk");
        assert_eq!(file.size_bytes, 1048576);
        assert_eq!(file.checksum, "9f86d0818");
    }

    #[test]
    fn model_version_list_matches_backend_contract() {
        let response: ModelVersionListResponse = serde_json::from_str(
            r#"{
                "items": [{
                    "id": "0198f0a1-0000-7000-8000-000000000001",
                    "experiment": {"id": 12, "experiment_num": 4},
                    "version": 1,
                    "size": 2048,
                    "checksum": "abc",
                    "created_by": {"id": 7, "username": "ada", "namespace": "ada"},
                    "created_at": "2026-03-05 18:45:43.397",
                    "manifest": {"files": []},
                    "metadata": null
                }],
                "total": 1
            }"#,
        )
        .unwrap();

        assert_eq!(response.total, 1);
        let version = &response.items[0];
        assert_eq!(version.version, 1);
        assert_eq!(version.size, 2048);
        assert_eq!(version.experiment.as_ref().unwrap().experiment_num, 4);
        assert!(version.metadata.is_null());
    }

    /// The metadata blob comes back exactly as the publisher stored it, and a
    /// server that predates the field reads as no metadata rather than an error.
    #[test]
    fn version_metadata_round_trips_verbatim() {
        let version: ModelVersionResponse = serde_json::from_str(
            r#"{
                "id": "0198f0a1-0000-7000-8000-000000000001",
                "experiment": null,
                "version": 2,
                "size": 2048,
                "checksum": "abc",
                "created_by": {"id": 7, "username": "ada", "namespace": "ada"},
                "created_at": "2026-03-05 18:45:43.397",
                "manifest": {"files": []},
                "metadata": {"metabolic": {"repo": "Qwen/Qwen3-0.6B"}}
            }"#,
        )
        .unwrap();

        assert_eq!(version.metadata["metabolic"]["repo"], "Qwen/Qwen3-0.6B");

        let without_field = r#"{
            "id": "0198f0a1-0000-7000-8000-000000000001",
            "experiment": null,
            "version": 2,
            "size": 2048,
            "checksum": "abc",
            "created_by": {"id": 7, "username": "ada", "namespace": "ada"},
            "created_at": "2026-03-05 18:45:43.397",
            "manifest": {"files": []}
        }"#;
        let version: ModelVersionResponse = serde_json::from_str(without_field).unwrap();
        assert!(version.metadata.is_null());
    }
}
