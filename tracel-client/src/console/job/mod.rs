pub mod response;

mod body;

use crate::{
    ClientError,
    console::{
        Client,
        job::{
            body::ComputeProviderQueueJobRequest,
            response::{JobLogResponse, JobResponse, ProjectJobsResponse, QueuedJobResponse},
        },
    },
};

impl Client {
    /// Queue a job that runs a code version on a compute provider group.
    ///
    /// The client must be logged in before calling this method.
    pub fn start_remote_job(
        &self,
        compute_provider_group_name: &str,
        owner_name: &str,
        project_name: &str,
        digest: &str,
        command: &str,
    ) -> Result<QueuedJobResponse, ClientError> {
        let body = ComputeProviderQueueJobRequest {
            compute_provider_group_name: compute_provider_group_name.to_string(),
            digest: digest.to_string(),
            command: command.to_string(),
        };

        self.transport.post_json(
            format!("projects/{owner_name}/{project_name}/jobs/queue"),
            Some(body),
        )
    }

    /// List the jobs of a project.
    ///
    /// The client must be logged in before calling this method.
    pub fn list_jobs(
        &self,
        namespace: &str,
        project_name: &str,
    ) -> Result<ProjectJobsResponse, ClientError> {
        self.transport
            .get_json(format!("projects/{namespace}/{project_name}/jobs"))
    }

    /// Get a job by its project-scoped number.
    ///
    /// The client must be logged in before calling this method.
    pub fn get_job(
        &self,
        namespace: &str,
        project_name: &str,
        job_num: i32,
    ) -> Result<JobResponse, ClientError> {
        self.transport.get_json(format!(
            "projects/{namespace}/{project_name}/jobs/{job_num}"
        ))
    }

    /// Read a page of a job's log file from byte offset `start`, or from the beginning.
    ///
    /// The client must be logged in before calling this method.
    pub fn get_job_logs(
        &self,
        namespace: &str,
        project_name: &str,
        job_num: i32,
        start: Option<u64>,
    ) -> Result<JobLogResponse, ClientError> {
        let mut url = self.transport.join(&format!(
            "projects/{namespace}/{project_name}/jobs/{job_num}/logs"
        ));
        if let Some(start) = start {
            url.query_pairs_mut()
                .append_pair("start", &start.to_string());
        }

        self.transport.get_json(url)
    }

    /// Cancel a new, queued or running job.
    ///
    /// The client must be logged in before calling this method.
    pub fn cancel_job(
        &self,
        namespace: &str,
        project_name: &str,
        job_num: i32,
    ) -> Result<(), ClientError> {
        self.transport.put(
            format!("projects/{namespace}/{project_name}/jobs/{job_num}/cancel"),
            None::<()>,
        )
    }
}
