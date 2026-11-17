use std::io::Read;
use std::time::Duration;

const FETCH_LIMIT: u64 = 1 << 20;

/// Why a fetch failed.
#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    /// The server answered with a status other than 200.
    #[error("{url} returned status {code}")]
    Status {
        /// The URL requested.
        url: String,
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

impl FetchError {
    /// True when the server said the file does not exist.
    pub fn is_not_found(&self) -> bool {
        matches!(self, FetchError::Status { code: 404, .. })
    }
}

/// Fetches a URL's body. Tests swap in a closure so they never touch the network.
pub trait Fetch {
    /// Returns the body of a 200 response, capped at 1 MiB.
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError>;
}

impl<F> Fetch for F
where
    F: Fn(&str) -> Result<Vec<u8>, FetchError>,
{
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        self(url)
    }
}

/// The real network, over blocking HTTPS.
pub struct HttpFetch {
    agent: ureq::Agent,
}

impl HttpFetch {
    /// A fetcher whose every request gives up after `timeout`.
    pub fn new(timeout: Duration) -> Self {
        HttpFetch {
            agent: ureq::AgentBuilder::new().timeout(timeout).build(),
        }
    }
}

fn status(url: &str, code: u16) -> FetchError {
    FetchError::Status {
        url: url.to_string(),
        code,
    }
}

fn transport(url: &str, message: String) -> FetchError {
    FetchError::Transport {
        url: url.to_string(),
        message,
    }
}

impl Fetch for HttpFetch {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        let accept = "text/plain, text/html, application/json";
        let response = match self.agent.get(url).set("Accept", accept).call() {
            Ok(r) if r.status() == 200 => r,
            Ok(r) => return Err(status(url, r.status())),
            Err(ureq::Error::Status(code, _)) => return Err(status(url, code)),
            Err(e) => return Err(transport(url, e.to_string())),
        };
        let mut body = Vec::new();
        response
            .into_reader()
            .take(FETCH_LIMIT)
            .read_to_end(&mut body)
            .map_err(|e| transport(url, e.to_string()))?;
        Ok(body)
    }
}
