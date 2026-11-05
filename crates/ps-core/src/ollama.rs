use crate::endpoint::{is_loopback_url, normalize_local_endpoint};
use crate::error::{PsError, PsResult};
use crate::models::{classify_model, validate_model_name, ListedModel, ModelParts};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
pub struct OllamaClient {
    endpoint: String,
    http: reqwest::Client,
}

impl OllamaClient {
    pub fn new(endpoint: &str) -> PsResult<Self> {
        let endpoint = normalize_local_endpoint(endpoint)?;
        let http = reqwest::Client::builder()
            .redirect(local_redirect_policy())
            .user_agent("ps-prompt-optimizer")
            .build()
            .map_err(|_| PsError::OllamaUnavailable {
                endpoint: endpoint.clone(),
                detail: "The local HTTP client could not be created.".into(),
            })?;
        Ok(Self { endpoint, http })
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub async fn list_models(&self) -> PsResult<Vec<ListedModel>> {
        let url = format!("{}/api/tags", self.endpoint);
        let response = self
            .http
            .get(url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|error| map_transport(error, &self.endpoint))?;
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|_| PsError::OllamaUnavailable {
                endpoint: self.endpoint.clone(),
                detail: "Ollama sent an unreadable model list.".into(),
            })?;
        if !status.is_success() {
            return Err(map_http_status(status, &body, &self.endpoint, ""));
        }
        parse_tags(&body)
    }

    pub async fn show_model(&self, model: &str) -> PsResult<ModelMetadata> {
        validate_model_name(model)?;
        let url = format!("{}/api/show", self.endpoint);
        let response = self
            .http
            .post(url)
            .timeout(Duration::from_secs(15))
            .json(&serde_json::json!({ "model": model }))
            .send()
            .await
            .map_err(|error| map_transport(error, &self.endpoint))?;
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|_| PsError::MalformedOutput {
                detail: "Ollama model details could not be read.".into(),
            })?;
        if !status.is_success() {
            return Err(map_http_status(status, &body, &self.endpoint, model));
        }
        parse_show(model, &body)
    }

    pub async fn chat(
        &self,
        request: &ChatRequest,
        timeout: Duration,
        cancel: &CancellationToken,
    ) -> PsResult<ChatResponse> {
        if cancel.is_cancelled() {
            return Err(PsError::Cancelled);
        }
        validate_model_name(&request.model)?;
        let url = format!("{}/api/chat", self.endpoint);
        let pending = self.http.post(url).timeout(timeout).json(request).send();
        tokio::pin!(pending);
        let response = tokio::select! {
            _ = cancel.cancelled() => return Err(PsError::Cancelled),
            result = &mut pending => result.map_err(|error| map_transport(error, &self.endpoint))?,
        };
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|_| PsError::MalformedOutput {
                detail: "Ollama returned an unreadable chat response.".into(),
            })?;
        if cancel.is_cancelled() {
            return Err(PsError::Cancelled);
        }
        if !status.is_success() {
            return Err(map_http_status(
                status,
                &body,
                &self.endpoint,
                &request.model,
            ));
        }
        serde_json::from_str(&body).map_err(|_| PsError::MalformedOutput {
            detail: "Ollama returned a chat response that was not valid JSON.".into(),
        })
    }

    pub async fn delete_model(&self, model: &str) -> PsResult<()> {
        validate_model_name(model)?;
        let url = format!("{}/api/delete", self.endpoint);
        let response = self
            .http
            .delete(url)
            .timeout(Duration::from_secs(60))
            .json(&serde_json::json!({ "model": model }))
            .send()
            .await
            .map_err(|error| map_transport(error, &self.endpoint))?;
        let status = response.status();
        if status.is_success() {
            return Ok(());
        }
        let body = response.text().await.unwrap_or_default();
        Err(map_http_status(status, &body, &self.endpoint, model))
    }

    pub async fn pull<F>(
        &self,
        model: &str,
        cancel: &CancellationToken,
        mut on_progress: F,
    ) -> PsResult<()>
    where
        F: FnMut(PullProgress),
    {
        if cancel.is_cancelled() {
            return Err(PsError::Cancelled);
        }
        validate_model_name(model)?;
        let url = format!("{}/api/pull", self.endpoint);
        let pending = self
            .http
            .post(url)
            .json(&serde_json::json!({ "model": model, "stream": true }))
            .send();
        tokio::pin!(pending);
        let response = tokio::select! {
            _ = cancel.cancelled() => return Err(PsError::Cancelled),
            result = &mut pending => result.map_err(|error| map_transport(error, &self.endpoint))?,
        };
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(map_http_status(status, &body, &self.endpoint, model));
        }

        let mut stream = response.bytes_stream();
        let mut buffer: Vec<u8> = Vec::new();
        loop {
            tokio::select! {
                _ = cancel.cancelled() => return Err(PsError::Cancelled),
                item = stream.next() => {
                    match item {
                        None => break,
                        Some(Err(_)) => {
                            return Err(PsError::ModelDownload {
                                detail: "The model download stream ended unexpectedly.".into(),
                            });
                        }
                        Some(Ok(chunk)) => buffer.extend_from_slice(&chunk),
                    }
                }
            }
            while let Some(index) = buffer.iter().position(|byte| *byte == b'\n') {
                let line: Vec<u8> = buffer.drain(..=index).collect();
                let text = String::from_utf8_lossy(&line);
                if let Some(progress) = parse_pull_line(&text)? {
                    let failed = progress.error.clone();
                    let done = progress.done;
                    on_progress(progress);
                    if let Some(message) = failed {
                        return Err(PsError::ModelDownload { detail: message });
                    }
                    if done {
                        return Ok(());
                    }
                }
            }
        }
        if !buffer.is_empty() {
            let text = String::from_utf8_lossy(&buffer);
            if let Some(progress) = parse_pull_line(&text)? {
                let failed = progress.error.clone();
                let done = progress.done;
                on_progress(progress);
                if let Some(message) = failed {
                    return Err(PsError::ModelDownload { detail: message });
                }
                if done {
                    return Ok(());
                }
            }
        }
        Err(PsError::ModelDownload {
            detail: "The download finished without a success status.".into(),
        })
    }
}

fn local_redirect_policy() -> reqwest::redirect::Policy {
    reqwest::redirect::Policy::custom(|attempt| {
        if attempt.previous().len() >= 3 || !is_loopback_url(attempt.url()) {
            attempt.stop()
        } else {
            attempt.follow()
        }
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
    pub format: serde_json::Value,
    pub keep_alive: String,
    pub think: bool,
    pub options: ChatOptions,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatOptions {
    pub temperature: f32,
    pub top_p: f32,
    pub num_predict: u32,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ChatResponse {
    #[serde(default)]
    pub message: Option<ChatMessageBody>,
    #[serde(default)]
    pub done: bool,
    #[serde(default)]
    pub done_reason: Option<String>,
    #[serde(default)]
    pub prompt_eval_count: Option<u64>,
    #[serde(default)]
    pub eval_count: Option<u64>,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ChatMessageBody {
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub thinking: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PullProgress {
    pub status: String,
    pub digest: Option<String>,
    pub total: Option<u64>,
    pub completed: Option<u64>,
    pub percent: Option<u8>,
    pub error: Option<String>,
    pub done: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelMetadata {
    pub name: String,
    pub parameter_size: Option<String>,
    pub quantization: Option<String>,
    pub family: Option<String>,
    pub capabilities: Vec<String>,
    pub cloud: bool,
}

pub fn parse_pull_line(line: &str) -> PsResult<Option<PullProgress>> {
    let line = line.trim();
    if line.is_empty() {
        return Ok(None);
    }
    let value: serde_json::Value =
        serde_json::from_str(line).map_err(|_| PsError::ModelDownload {
            detail: "Ollama sent a download update that was not valid JSON.".into(),
        })?;
    if let Some(message) = value.get("error").and_then(|item| item.as_str()) {
        return Ok(Some(PullProgress {
            status: "error".into(),
            digest: None,
            total: None,
            completed: None,
            percent: None,
            error: Some(bounded_detail(message)),
            done: true,
        }));
    }
    let status = value
        .get("status")
        .and_then(|item| item.as_str())
        .unwrap_or("working")
        .to_string();
    let total = json_u64(&value, "total");
    let completed = json_u64(&value, "completed");
    let percent = match (completed, total) {
        (Some(done), Some(all)) if all > 0 => {
            Some(((done.saturating_mul(100)) / all).min(100) as u8)
        }
        _ => None,
    };
    Ok(Some(PullProgress {
        done: status.eq_ignore_ascii_case("success"),
        status,
        digest: value
            .get("digest")
            .and_then(|item| item.as_str())
            .map(ToOwned::to_owned),
        total,
        completed,
        percent,
        error: None,
    }))
}

pub fn parse_tags(body: &str) -> PsResult<Vec<ListedModel>> {
    let value: serde_json::Value =
        serde_json::from_str(body).map_err(|_| PsError::MalformedOutput {
            detail: "The model list was not valid JSON.".into(),
        })?;
    let Some(entries) = value.get("models").and_then(|item| item.as_array()) else {
        return Err(PsError::MalformedOutput {
            detail: "The model list did not include a models array.".into(),
        });
    };
    let mut models = Vec::new();
    for entry in entries {
        let name = entry
            .get("name")
            .or_else(|| entry.get("model"))
            .and_then(|item| item.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if name.is_empty() {
            continue;
        }
        let details = entry.get("details");
        models.push(classify_model(ModelParts {
            name,
            size_bytes: json_u64(entry, "size").unwrap_or(0),
            digest: entry
                .get("digest")
                .and_then(|item| item.as_str())
                .map(ToOwned::to_owned),
            parameter_size: details
                .and_then(|item| item.get("parameter_size"))
                .and_then(|item| item.as_str())
                .map(ToOwned::to_owned),
            quantization: details
                .and_then(|item| item.get("quantization_level"))
                .and_then(|item| item.as_str())
                .map(ToOwned::to_owned),
            family: details
                .and_then(|item| item.get("family"))
                .and_then(|item| item.as_str())
                .map(ToOwned::to_owned),
            remote_host: entry
                .get("remote_host")
                .and_then(|item| item.as_str())
                .map(ToOwned::to_owned),
            remote_model: entry
                .get("remote_model")
                .and_then(|item| item.as_str())
                .map(ToOwned::to_owned),
        }));
    }
    Ok(models)
}

fn parse_show(requested: &str, body: &str) -> PsResult<ModelMetadata> {
    let value: serde_json::Value =
        serde_json::from_str(body).map_err(|_| PsError::MalformedOutput {
            detail: "Model details were not valid JSON.".into(),
        })?;
    let details = value.get("details");
    let remote_host = value
        .get("remote_host")
        .and_then(|item| item.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let remote_model = value
        .get("remote_model")
        .and_then(|item| item.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let capabilities = value
        .get("capabilities")
        .and_then(|item| item.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(ToOwned::to_owned))
                .collect()
        })
        .unwrap_or_default();
    let classified = classify_model(ModelParts {
        name: requested.to_string(),
        remote_host,
        remote_model,
        ..ModelParts::default()
    });
    Ok(ModelMetadata {
        name: requested.to_string(),
        parameter_size: details
            .and_then(|item| item.get("parameter_size"))
            .and_then(|item| item.as_str())
            .map(ToOwned::to_owned),
        quantization: details
            .and_then(|item| item.get("quantization_level"))
            .and_then(|item| item.as_str())
            .map(ToOwned::to_owned),
        family: details
            .and_then(|item| item.get("family"))
            .and_then(|item| item.as_str())
            .map(ToOwned::to_owned),
        capabilities,
        cloud: classified.cloud,
    })
}

fn json_u64(value: &serde_json::Value, key: &str) -> Option<u64> {
    value.get(key).and_then(|item| item.as_u64())
}

fn map_transport(error: reqwest::Error, endpoint: &str) -> PsError {
    if error.is_timeout() {
        return PsError::Timeout;
    }
    PsError::OllamaUnavailable {
        endpoint: endpoint.to_string(),
        detail: "Start Ollama locally, then retry the connection.".into(),
    }
}

fn map_http_status(
    status: reqwest::StatusCode,
    body: &str,
    endpoint: &str,
    model: &str,
) -> PsError {
    let lower = body.to_ascii_lowercase();
    if status.as_u16() == 404 || lower.contains("not found") {
        return PsError::ModelUnavailable {
            model: if model.is_empty() {
                "requested model".into()
            } else {
                model.to_string()
            },
        };
    }
    if lower.contains("cloud") {
        return PsError::CloudModelRejected {
            model: model.to_string(),
        };
    }
    if !model.is_empty() && lower.contains("pull") {
        return PsError::ModelDownload {
            detail: format!(
                "Ollama could not download the model (HTTP {}).",
                status.as_u16()
            ),
        };
    }
    PsError::OllamaUnavailable {
        endpoint: endpoint.to_string(),
        detail: format!("Ollama returned HTTP {}.", status.as_u16()),
    }
}

fn bounded_detail(message: &str) -> String {
    let mut owned = message.trim().to_string();
    if owned.chars().count() > 240 {
        owned = owned.chars().take(240).collect();
        owned.push('…');
    }
    owned
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_download_progress_success_and_errors() {
        let progress = parse_pull_line(
            r#"{"status":"downloading","digest":"abc","total":200,"completed":50}"#,
        )
        .unwrap()
        .unwrap();
        assert_eq!(progress.percent, Some(25));
        assert!(!progress.done);

        let success = parse_pull_line(r#"{"status":"success"}"#).unwrap().unwrap();
        assert!(success.done);

        let error = parse_pull_line(r#"{"error":"manifest not found"}"#)
            .unwrap()
            .unwrap();
        assert_eq!(error.error.as_deref(), Some("manifest not found"));
        assert!(parse_pull_line("").unwrap().is_none());
        assert!(parse_pull_line("{").is_err());
    }

    #[test]
    fn tags_parser_separates_local_and_cloud_models() {
        let body = r#"{
            "models": [
                {
                    "name": "qwen3.5:4b",
                    "size": 1200,
                    "digest": "abc",
                    "details": {"family": "qwen3", "parameter_size": "4B", "quantization_level": "Q4_K_M"}
                },
                {"name": "gpt-oss:120b-cloud", "size": 0, "remote_host": "https://ollama.com"}
            ]
        }"#;
        let models = parse_tags(body).unwrap();
        assert!(models[0].runnable);
        assert_eq!(models[0].parameter_size.as_deref(), Some("4B"));
        assert!(models[1].cloud);
        assert!(!models[1].runnable);
    }
}
