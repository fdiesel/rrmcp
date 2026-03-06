use thiserror::Error;

#[derive(Debug, Error)]
pub enum RedmineError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Redmine API error ({status}): {message}")]
    Api { status: u16, message: String },

    #[error("Not found")]
    NotFound,

    #[error("Unauthorized — check your Redmine API key")]
    Unauthorized,

    #[error("Validation errors: {0}")]
    Validation(String),

    #[error("Unexpected response: {0}")]
    UnexpectedResponse(String),
}

impl RedmineError {
    pub fn from_status(status: u16, body: &str) -> Self {
        match status {
            401 | 403 => RedmineError::Unauthorized,
            404 => RedmineError::NotFound,
            422 => {
                // Try to extract Redmine validation errors
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(body) {
                    if let Some(errors) = v.get("errors").and_then(|e| e.as_array()) {
                        let msg = errors
                            .iter()
                            .filter_map(|e| e.as_str())
                            .collect::<Vec<_>>()
                            .join(", ");
                        return RedmineError::Validation(msg);
                    }
                }
                RedmineError::Validation(body.to_string())
            }
            _ => RedmineError::Api {
                status,
                message: body.to_string(),
            },
        }
    }
}
