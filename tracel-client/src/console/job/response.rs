use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QueuedJobResponse {
    pub job_id: String,
    pub compute_provider_group_id: String,
    /// The project-scoped job number, or `None` from a server that does not send it.
    #[serde(default)]
    pub job_num: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProjectJobsResponse {
    pub jobs: Vec<ProjectJobResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProjectJobResponse {
    pub job_num: i32,
    pub job_project_version: String,
    pub job_command: String,
    /// `new`, `queued`, `running`, `pending_cancellation`, `completed`, `failed` or `cancelled`.
    pub status: String,
    pub execution_context: Option<ExecutionContextResponse>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JobResponse {
    pub job_num: i32,
    pub compute_provider_id: Option<i32>,
    pub job_project: JobProjectResponse,
    pub job_project_version: String,
    pub job_command: String,
    /// `new`, `queued`, `running`, `pending_cancellation`, `completed`, `failed` or `cancelled`.
    pub status: String,
    pub status_message: String,
    pub execution_context: Option<ExecutionContextResponse>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JobProjectResponse {
    pub project_name: String,
    pub namespace_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum ExecutionContextResponse {
    Managed {
        compute_provider_name: String,
        memory_gb: i32,
        cpu_cores: i32,
        num_gpus: Option<i32>,
        lock_price_per_hour: MoneyResponse,
        final_cost: Option<MoneyResponse>,
        estimated_cost: Option<MoneyResponse>,
    },
    SelfManaged {
        compute_provider_name: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MoneyResponse {
    /// A decimal amount.
    pub amount: String,
    pub currency: String,
}

/// A page of a job's log, in line order.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JobLogResponse {
    pub lines: Vec<JobLogLine>,
    /// More lines lie past the page in the direction it was read.
    pub has_more: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct JobLogLine {
    /// The line's number in the job's log, counted from 0.
    pub seq: u64,
    pub stream: JobLogStream,
    pub emitted_at: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobLogStream {
    Stdout,
    Stderr,
}

/// A message of a job's live log WebSocket.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum JobLogStreamMessage {
    /// A line the job's compute provider sent, with its newline.
    Log(String),
}
