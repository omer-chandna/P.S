use crate::error::{PsError, PsResult};
use url::Url;

pub const DEFAULT_OLLAMA_ENDPOINT: &str = "http://127.0.0.1:11434";

/// Accept only loopback HTTP(S) origins. Paths, credentials, and public hosts are rejected
/// so inference cannot be pointed at a remote service.
pub fn normalize_local_endpoint(raw: &str) -> PsResult<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(PsError::InvalidEndpoint {
            detail: "Enter the local Ollama address.".into(),
        });
    }

    let url = Url::parse(raw).map_err(|_| PsError::InvalidEndpoint {
        detail: "The Ollama address is not a valid URL.".into(),
    })?;

    if url.scheme() != "http" && url.scheme() != "https" {
        return Err(PsError::InvalidEndpoint {
            detail: "The Ollama address must use http or https.".into(),
        });
    }

    if !url.username().is_empty() || url.password().is_some() {
        return Err(PsError::InvalidEndpoint {
            detail: "The Ollama address cannot include a username or password.".into(),
        });
    }

    let host = url.host_str().ok_or_else(|| PsError::InvalidEndpoint {
        detail: "The Ollama address needs a host.".into(),
    })?;
    let host_bare = host.trim_matches(['[', ']']).to_ascii_lowercase();
    if !is_loopback_host(&host_bare) {
        return Err(PsError::InvalidEndpoint {
            detail: "Inference is restricted to a local Ollama endpoint (localhost or 127.0.0.1)."
                .into(),
        });
    }

    if url.path() != "/" && !url.path().is_empty() {
        return Err(PsError::InvalidEndpoint {
            detail: "The Ollama address must be the server origin, without a path.".into(),
        });
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err(PsError::InvalidEndpoint {
            detail: "The Ollama address cannot include a query or fragment.".into(),
        });
    }

    let mut normalized = format!("{}://{}", url.scheme(), host_bare);
    if host_bare == "::1" {
        normalized = format!("{}://[::1]", url.scheme());
    }
    if let Some(port) = url.port() {
        normalized.push(':');
        normalized.push_str(&port.to_string());
    }
    Ok(normalized)
}

pub fn is_loopback_host(host: &str) -> bool {
    let host = host.trim_matches(['[', ']']).to_ascii_lowercase();
    matches!(host.as_str(), "localhost" | "127.0.0.1" | "::1")
}

pub fn is_loopback_url(url: &Url) -> bool {
    url.host_str().map(is_loopback_host).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_default_and_localhost_variants() {
        assert_eq!(
            normalize_local_endpoint("http://127.0.0.1:11434").unwrap(),
            "http://127.0.0.1:11434"
        );
        assert_eq!(
            normalize_local_endpoint("http://localhost:11434/").unwrap(),
            "http://localhost:11434"
        );
        assert_eq!(
            normalize_local_endpoint("http://[::1]:11434").unwrap(),
            "http://[::1]:11434"
        );
    }

    #[test]
    fn rejects_remote_and_decorated_urls() {
        assert!(normalize_local_endpoint("https://ollama.com").is_err());
        assert!(normalize_local_endpoint("http://192.168.1.20:11434").is_err());
        assert!(normalize_local_endpoint("http://10.0.0.2:11434").is_err());
        assert!(normalize_local_endpoint("file:///tmp/ollama").is_err());
        assert!(normalize_local_endpoint("http://user:pass@127.0.0.1:11434").is_err());
        assert!(normalize_local_endpoint("http://127.0.0.1:11434/api").is_err());
        assert!(normalize_local_endpoint("http://127.0.0.1:11434?x=1").is_err());
    }
}
