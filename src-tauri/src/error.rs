use serde::Serialize;

/// Structured error type returned from Tauri commands.
/// Frontend can match on `code` to distinguish error categories.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    ProfileNotFound,
    AuthFailed,
    ConnectionFailed,
    TunnelBindFailed,
    TunnelUnsupported,
    ConfigError,
    CredentialError,
    HostKeyMismatch,
    Internal,
}

impl AppError {
    pub fn profile_not_found(id: &str) -> Self {
        Self {
            code: ErrorCode::ProfileNotFound,
            message: format!("Profile not found: {}", id),
        }
    }

    pub fn auth_failed(msg: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::AuthFailed,
            message: msg.into(),
        }
    }

    pub fn connection_failed(msg: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::ConnectionFailed,
            message: msg.into(),
        }
    }

    pub fn config(msg: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::ConfigError,
            message: msg.into(),
        }
    }

    pub fn credential(msg: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::CredentialError,
            message: msg.into(),
        }
    }

    pub fn host_key_mismatch(msg: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::HostKeyMismatch,
            message: msg.into(),
        }
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::Internal,
            message: msg.into(),
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        Self::internal(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_code_serializes_screaming_snake_case() {
        let json = serde_json::to_string(&ErrorCode::ProfileNotFound).unwrap();
        assert_eq!(json, r#""PROFILE_NOT_FOUND""#);

        let json = serde_json::to_string(&ErrorCode::AuthFailed).unwrap();
        assert_eq!(json, r#""AUTH_FAILED""#);

        let json = serde_json::to_string(&ErrorCode::ConnectionFailed).unwrap();
        assert_eq!(json, r#""CONNECTION_FAILED""#);

        let json = serde_json::to_string(&ErrorCode::TunnelBindFailed).unwrap();
        assert_eq!(json, r#""TUNNEL_BIND_FAILED""#);

        let json = serde_json::to_string(&ErrorCode::ConfigError).unwrap();
        assert_eq!(json, r#""CONFIG_ERROR""#);
    }

    #[test]
    fn app_error_constructors() {
        let e = AppError::profile_not_found("abc-123");
        assert!(matches!(e.code, ErrorCode::ProfileNotFound));
        assert!(e.message.contains("abc-123"));

        let e = AppError::auth_failed("bad password");
        assert!(matches!(e.code, ErrorCode::AuthFailed));
        assert_eq!(e.message, "bad password");

        let e = AppError::connection_failed("timeout");
        assert!(matches!(e.code, ErrorCode::ConnectionFailed));

        let e = AppError::config("parse error");
        assert!(matches!(e.code, ErrorCode::ConfigError));

        let e = AppError::credential("keyring error");
        assert!(matches!(e.code, ErrorCode::CredentialError));

        let e = AppError::internal("unexpected");
        assert!(matches!(e.code, ErrorCode::Internal));
    }

    #[test]
    fn display_impl_returns_message() {
        let e = AppError::auth_failed("wrong credentials");
        assert_eq!(format!("{}", e), "wrong credentials");
    }

    #[test]
    fn from_anyhow_creates_internal() {
        let anyhow_err = anyhow::anyhow!("something went wrong");
        let e = AppError::from(anyhow_err);
        assert!(matches!(e.code, ErrorCode::Internal));
        assert!(e.message.contains("something went wrong"));
    }

    #[test]
    fn app_error_serializes_to_json() {
        let e = AppError::auth_failed("bad key");
        let json: serde_json::Value = serde_json::to_value(&e).unwrap();
        assert_eq!(json["code"], "AUTH_FAILED");
        assert_eq!(json["message"], "bad key");
    }
}
