use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub(crate) struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
}

/// Operating system of a binary target. Serializes to match the server's
/// `OperatingSystem` enum (lowercase: `windows`, `linux`, `macos`).
#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Os {
    Windows,
    Linux,
    Macos,
}

/// CPU architecture of a binary target. Serializes to match the server's
/// `Architecture` enum (lowercase: `x86_64`, `arm64`).
#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Arch {
    X86_64,
    Arm64,
}

/// A single compiled binary in a `binaries` code version.
#[derive(Debug, Serialize, Clone)]
pub struct PublishBinaryRequest {
    pub os: Os,
    pub architecture: Arch,
    pub checksum: String,
    pub size: u64,
}

/// The single source archive in a `source` code version.
#[derive(Debug, Serialize, Clone)]
pub struct PublishSourceRequest {
    pub checksum: String,
    pub size: u64,
}

/// The artifact a code version describes: either a set of compiled binaries
/// (each tagged with its OS/arch) or a single source archive. Serialized with an
/// internal `type` tag to match the server's `PublishArtifactRequest`.
#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PublishArtifactRequest {
    Binaries { binaries: Vec<PublishBinaryRequest> },
    Source { source: PublishSourceRequest },
}

/// Body of `POST /projects/{owner}/{project}/code/upload`.
#[derive(Debug, Serialize, Clone)]
pub struct PublishProjectVersionRequest {
    pub digest: String,
    pub artifact: PublishArtifactRequest,
}

/// Optional body of `POST .../code/{id}/complete`: the multipart uploads the
/// client finished, so the server can finalize the exact uploads it wrote to.
#[derive(Debug, Serialize, Clone)]
pub struct CompleteCodeUploadRequest {
    pub artifacts: Vec<CompletedCodeArtifact>,
}

#[derive(Debug, Serialize, Clone)]
pub struct CompletedCodeArtifact {
    pub key: String,
    pub upload_id: String,
    pub parts: Vec<CompletedCodePart>,
}

#[derive(Debug, Serialize, Clone)]
pub struct CompletedCodePart {
    pub part_number: i32,
    pub etag: String,
}
