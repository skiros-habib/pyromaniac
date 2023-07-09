use std::{sync::Arc, time::Duration};

use crate::runner::run_code;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Result},
    routing::post,
    Json, Router,
};
use tokio::sync::Semaphore;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CodeRun {
    code: String,
    input: Option<String>,
    lang: pyrod_service::Language,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CodeOutput {
    stdout: String,
    stderr: String,
}

#[tracing::instrument(ret)]
async fn run(
    State(semaphore): State<Arc<Semaphore>>,
    Json(req): Json<CodeRun>,
) -> Result<Json<CodeOutput>> {
    //there's two things done here to bound the number of VMs running

    //Tasks are spawned with a timeout of 30 seconds.
    //after that the task is dropped, therefore machine dropped and process killed

    //We also have a Semaphore. Tasks must acquire a permit from the semaphore
    //before being allowed to run. This limits the number of VMs active at any time.

    //this does mean requests may take a while, so we need to set HTTP timeouts appropriately

    let _permit = semaphore
        .acquire_owned()
        .await
        .map_err(|e| JsonError::from(e).into_response())?;

    let (stdout, stderr) = tokio::time::timeout(
        Duration::from_secs(30),
        run_code(req.lang, req.code, req.input.unwrap_or("".to_string())),
    )
    .await
    .map_err(|e| JsonError::from(e).into_response())? // Result::flatten is nightly still
    .map_err(|e| JsonError::from(e).into_response())?;

    Ok(Json(CodeOutput { stdout, stderr }))
}

pub fn app() -> Router {
    //you really expect anyone to handle this error?
    let n_cpus = std::thread::available_parallelism()
        .expect("Failed to determine number of CPUs")
        .get();
    let max_vms = n_cpus * 2;

    Router::new()
        .route("/api/run", post(run))
        .with_state(Arc::new(Semaphore::new(max_vms)))
        .fallback(|| async { (StatusCode::NOT_FOUND, "Not Found\n") })
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct JsonError {
    error: String,
}

impl<E: ToString> From<E> for JsonError {
    fn from(value: E) -> Self {
        Self {
            error: value.to_string(),
        }
    }
}

impl IntoResponse for JsonError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(self)).into_response()
    }
}
