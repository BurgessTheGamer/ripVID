use thiserror::Error;

/// Custom error types for the download application
#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Process failed: {0}")]
    ProcessFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Sidecar error: {0}")]
    Sidecar(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("Download cancelled by user")]
    Cancelled,

    #[error("Quality not available: {0}")]
    QualityNotAvailable(String),

    #[error("Browser not found: {0}")]
    BrowserNotFound(String),

    #[error("Failed to parse output: {0}")]
    ParseError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<DownloadError> for String {
    fn from(error: DownloadError) -> Self {
        error.to_string()
    }
}

/// Determine if an error is retryable
pub fn is_retryable_error(error: &DownloadError) -> bool {
    matches!(
        error,
        DownloadError::Network(_) | DownloadError::RateLimit(_) | DownloadError::ProcessFailed(_)
    )
}

/// Determine if an error is a network error
pub fn is_network_error(stderr: &str) -> bool {
    stderr.contains("Unable to download")
        || stderr.contains("HTTP Error")
        || stderr.contains("Connection")
        || stderr.contains("timeout")
        || stderr.contains("network")
}

/// Determine if an error is a rate limit error
pub fn is_rate_limit_error(stderr: &str) -> bool {
    stderr.contains("rate limit") || stderr.contains("429") || stderr.contains("Too Many Requests")
}

/// Determine if an error is an authentication error
pub fn is_auth_error(stderr: &str) -> bool {
    stderr.contains("Sign in")
        || stderr.contains("Private video")
        || stderr.contains("members-only")
        || stderr.contains("This video is only available")
        || stderr.contains("login required")
}

/// Determine if an error is a DPAPI cookie decryption error (Windows Chrome/Edge)
pub fn is_dpapi_error(stderr: &str) -> bool {
    stderr.contains("Failed to decrypt with DPAPI")
        || stderr.contains("DPAPI")
        || (stderr.contains("decrypt") && stderr.contains("cookie"))
}

/// Determine if an error is related to ffmpeg/merge issues
pub fn is_ffmpeg_error(stderr: &str) -> bool {
    (stderr.contains("ffmpeg") || stderr.contains("Merger") || stderr.contains("merge"))
        && (stderr.contains("not found")
            || stderr.contains("does not exist")
            || stderr.contains("NoneType")
            || stderr.contains("'lower'")
            || stderr.contains("FFmpeg"))
}
