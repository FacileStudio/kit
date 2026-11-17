use std::io::Read;
use std::time::Duration;

const DOWNLOAD_LIMIT: u64 = 256 << 20;

/// Why a request failed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum HttpError {
    /// The server answered with a status other than 200.
    #[error("{name} returned status {code}")]
    Status {
        /// The last path segment of the URL, for messages.
        name: String,
        /// The HTTP status code.
        code: u16,
    },
    /// The request never got an answer, or the body could not be read.
    #[error("fetching {url}: {message}")]
    Transport {
        /// The URL requested.
        url: String,
        /// What went wrong.
        message: String,
    },
}

impl HttpError {
    /// The status code of a server answer, if there was one.
    pub fn code(&self) -> Option<u16> {
        match self {
            HttpError::Status { code, .. } => Some(*code),
            HttpError::Transport { .. } => None,
        }
    }
}

/// Where a HEAD request ended after redirects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Head {
    /// The final status code.
    pub status: u16,
    /// The final URL.
    pub url: String,
}

/// The network. Tests swap in a fake so they never leave the machine.
pub trait Net: Send + Sync {
    /// The body of a 200 answer, capped at 256 MiB.
    fn get(&self, url: &str, headers: &[(&str, String)]) -> Result<Vec<u8>, HttpError>;
    /// Follows redirects and reports where they ended, whatever the status.
    fn head(&self, url: &str) -> Result<Head, HttpError>;
}

/// The real network over blocking HTTPS.
pub struct UreqNet {
    download: ureq::Agent,
    probe: ureq::Agent,
}

impl Default for UreqNet {
    fn default() -> Self {
        UreqNet {
            download: ureq::AgentBuilder::new()
                .timeout(Duration::from_secs(60))
                .build(),
            probe: ureq::AgentBuilder::new()
                .timeout(Duration::from_secs(15))
                .build(),
        }
    }
}

/// The last segment of a URL, the way messages name a file.
pub(crate) fn base_name(url: &str) -> String {
    url.rsplit('/').next().unwrap_or(url).to_string()
}

fn transport(url: &str, message: String) -> HttpError {
    HttpError::Transport {
        url: url.to_string(),
        message,
    }
}

impl Net for UreqNet {
    fn get(&self, url: &str, headers: &[(&str, String)]) -> Result<Vec<u8>, HttpError> {
        let mut request = self.download.get(url);
        for (key, value) in headers {
            request = request.set(key, value);
        }
        let response = match request.call() {
            Ok(r) if r.status() == 200 => r,
            Ok(r) => {
                return Err(HttpError::Status {
                    name: base_name(url),
                    code: r.status(),
                })
            }
            Err(ureq::Error::Status(code, _)) => {
                return Err(HttpError::Status {
                    name: base_name(url),
                    code,
                })
            }
            Err(e) => return Err(transport(url, e.to_string())),
        };
        let mut body = Vec::new();
        response
            .into_reader()
            .take(DOWNLOAD_LIMIT)
            .read_to_end(&mut body)
            .map_err(|e| transport(url, e.to_string()))?;
        Ok(body)
    }

    fn head(&self, url: &str) -> Result<Head, HttpError> {
        match self.probe.head(url).call() {
            Ok(r) | Err(ureq::Error::Status(_, r)) => Ok(Head {
                status: r.status(),
                url: r.get_url().to_string(),
            }),
            Err(e) => Err(transport(url, e.to_string())),
        }
    }
}
