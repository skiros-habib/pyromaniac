#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CodeRun {
    code: String,
    input: String,
    lang: pyrod_service::Language,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CodeResult {
    stdout: String,
    stderr: String,
}
