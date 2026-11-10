use ps_core::ollama::OllamaClient;
use ps_core::optimizer::{optimize_prompt, OptimizationStatus, OptimizeConfig};
use ps_core::settings::AppSettings;
use ps_core::PsError;
use serde_json::json;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn config() -> OptimizeConfig {
    OptimizeConfig::from(&AppSettings::default())
}

#[tokio::test]
async fn chat_round_trip_uses_structured_output_and_preserves_constraints() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .and(body_partial_json(json!({
            "stream": false,
            "think": false,
            "model": "qwen3.5:4b"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "model": "qwen3.5:4b",
            "done": true,
            "done_reason": "stop",
            "prompt_eval_count": 42,
            "eval_count": 30,
            "message": {
                "role": "assistant",
                "content": "{\"status\":\"rewritten\",\"rewritten_prompt\":\"Update src/auth/session.ts. Keep the public API unchanged and preserve compatibility with Node 22.\"}"
            }
        })))
        .mount(&server)
        .await;

    let client = OllamaClient::new(&server.uri()).unwrap();
    let analysis = optimize_prompt(
        &client,
        "qwen3.5:4b",
        "Update src/auth/session.ts without changing the public API. Keep compatibility with Node 22.",
        &config(),
        &CancellationToken::new(),
    )
    .await
    .unwrap();

    let requests = server.received_requests().await.unwrap();
    let body = String::from_utf8_lossy(&requests[0].body);
    assert_eq!(requests.len(), 1);
    assert!(body.contains("Clarify"));
    assert!(body.contains("Condense"));
    assert!(body.contains("MINIMUM SUFFICIENT PROMPT"));
    assert!(!body.contains("MODE: IMPROVE"));
    assert!(!body.contains("MODE: COMPACT"));
    assert!(!body.contains("needs_clarification"));

    assert_eq!(analysis.status, OptimizationStatus::Rewritten);
    assert!(
        !analysis.suspicious,
        "{:?}",
        analysis.preservation.warnings()
    );
    assert!(analysis.rewritten_prompt.contains("src/auth/session.ts"));
    assert!(analysis.rewritten_prompt.contains("Node 22"));
    assert_eq!(analysis.prompt_tokens, Some(42));
    assert_eq!(analysis.completion_tokens, Some(30));
    assert_eq!(analysis.prompt_version, "1.3.0");
}

#[tokio::test]
async fn unchanged_fixture_keeps_the_exact_original() {
    let original =
        "Add a loading spinner to the Submit button while the existing request is in progress.";
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "done": true,
            "done_reason": "stop",
            "message": {
                "role": "assistant",
                "content": format!(
                    "{{\"status\":\"unchanged\",\"rewritten_prompt\":{}}}",
                    serde_json::to_string(original).unwrap()
                )
            }
        })))
        .mount(&server)
        .await;

    let client = OllamaClient::new(&server.uri()).unwrap();
    let analysis = optimize_prompt(
        &client,
        "qwen3.5:4b",
        original,
        &config(),
        &CancellationToken::new(),
    )
    .await
    .unwrap();
    assert_eq!(analysis.status, OptimizationStatus::Unchanged);
    assert_eq!(analysis.rewritten_prompt, original);
    assert!(!analysis.suspicious);
}

#[tokio::test]
async fn a_question_only_reply_leaves_the_original() {
    let original =
        "Implement user deletion, but I haven't decided whether it should be permanent or reversible.";
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "done": true,
            "message": {
                "role": "assistant",
                "content": "{\"status\":\"needs_clarification\",\"rewritten_prompt\":\"\",\"clarification_question\":\"Should user deletion be permanent or reversible?\"}"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;
    let client = OllamaClient::new(&server.uri()).unwrap();
    let analysis = optimize_prompt(
        &client,
        "qwen3.5:4b",
        original,
        &config(),
        &CancellationToken::new(),
    )
    .await
    .unwrap();
    assert_eq!(analysis.status, OptimizationStatus::Unchanged);
    assert_eq!(analysis.rewritten_prompt, original);
}

#[tokio::test]
async fn a_legacy_question_with_a_rewrite_keeps_the_rewrite() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "done": true,
            "done_reason": "stop",
            "message": {
                "role": "assistant",
                "content": "{\"status\":\"needs_clarification\",\"rewritten_prompt\":\"Investigate the login failure after a page refresh. Identify the root cause, implement a focused fix, and verify that the expected login state is preserved.\",\"clarification_question\":\"Which file contains the login bug?\"}"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = OllamaClient::new(&server.uri()).unwrap();
    let analysis = optimize_prompt(
        &client,
        "qwen3.5:4b",
        "fix login it breaks on refresh",
        &config(),
        &CancellationToken::new(),
    )
    .await
    .unwrap();
    assert_eq!(analysis.status, OptimizationStatus::Rewritten);
    assert!(analysis.rewritten_prompt.contains("login failure"));
}

#[tokio::test]
async fn lists_models_and_reports_timeouts() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "models": [
                {"name": "qwen3.5:2b", "size": 10},
                {"name": "remote:cloud", "size": 1, "remote_model": "remote"}
            ]
        })))
        .mount(&server)
        .await;
    let client = OllamaClient::new(&server.uri()).unwrap();
    let models = client.list_models().await.unwrap();
    assert_eq!(models.len(), 2);
    assert!(models[0].runnable);
    assert!(!models[1].runnable);

    let slow = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/tags"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(6)))
        .mount(&slow)
        .await;
    let client = OllamaClient::new(&slow.uri()).unwrap();
    let error = client.list_models().await.unwrap_err();
    assert!(matches!(error, PsError::Timeout), "{error:?}");
}

#[tokio::test]
async fn pull_stream_reports_progress_and_cancellation() {
    let server = MockServer::start().await;
    let body = concat!(
        "{\"status\":\"pulling manifest\"}\n",
        "{\"status\":\"downloading\",\"total\":100,\"completed\":40}\n",
        "{\"status\":\"success\"}\n",
    );
    Mock::given(method("POST"))
        .and(path("/api/pull"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    let client = OllamaClient::new(&server.uri()).unwrap();
    let mut seen = Vec::new();
    client
        .pull("qwen3.5:4b", &CancellationToken::new(), |progress| {
            seen.push(progress.status);
        })
        .await
        .unwrap();
    assert_eq!(seen, vec!["pulling manifest", "downloading", "success"]);

    let stalled = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/pull"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("{\"status\":\"downloading\",\"total\":100,\"completed\":1}\n")
                .set_delay(Duration::from_secs(30)),
        )
        .mount(&stalled)
        .await;
    let client = OllamaClient::new(&stalled.uri()).unwrap();
    let cancel = CancellationToken::new();
    cancel.cancel();
    let error = client
        .pull("qwen3.5:4b", &cancel, |_| {})
        .await
        .unwrap_err();
    assert!(matches!(error, PsError::Cancelled));
}

#[tokio::test]
async fn delete_missing_model_is_a_model_error() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/delete"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({"error": "model not found"})))
        .mount(&server)
        .await;
    let client = OllamaClient::new(&server.uri()).unwrap();
    let error = client.delete_model("qwen3.5:4b").await.unwrap_err();
    assert!(matches!(error, PsError::ModelUnavailable { .. }));
}

#[tokio::test]
async fn remote_endpoints_never_create_a_client() {
    assert!(matches!(
        OllamaClient::new("https://ollama.com"),
        Err(PsError::InvalidEndpoint { .. })
    ));
}
