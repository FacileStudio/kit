use std::process::Command;
use std::sync::OnceLock;
use std::time::Duration;

use serde::Deserialize;

use super::net::{Net, UreqNet};
use super::FacileError;

const DOWNLOAD_ATTEMPTS: u32 = 3;

type TokenSource = Box<dyn Fn() -> String + Send + Sync>;

/// GitHub releases: the public download host first, the API as the fallback.
pub struct Github {
    /// The web host, `https://github.com` outside tests.
    pub web: String,
    /// The API host, `https://api.github.com` outside tests.
    pub api: String,
    /// The pause before the first retry; each later retry waits one more step.
    pub retry_delay: Duration,
    /// The network.
    pub net: Box<dyn Net>,
    token_source: TokenSource,
    token: OnceLock<String>,
}

#[derive(Deserialize)]
struct Asset {
    id: i64,
    name: String,
}

#[derive(Deserialize)]
struct ReleaseAssets {
    #[serde(default)]
    assets: Vec<Asset>,
}

impl Default for Github {
    fn default() -> Self {
        Github::new(Box::new(UreqNet::default()), Box::new(token_from_env))
    }
}

impl Github {
    /// A client on the real hosts with the given network and token source.
    pub fn new(net: Box<dyn Net>, token_source: TokenSource) -> Github {
        Github {
            web: "https://github.com".to_string(),
            api: "https://api.github.com".to_string(),
            retry_delay: Duration::from_secs(1),
            net,
            token_source,
            token: OnceLock::new(),
        }
    }

    /// Headers for an API call. The token goes to the API host only, never to a download host.
    pub fn api_headers(&self, accept: &str) -> Vec<(&'static str, String)> {
        let mut headers = vec![("Accept", accept.to_string())];
        let token = self.token.get_or_init(|| (self.token_source)());
        if !token.is_empty() {
            headers.push(("Authorization", format!("Bearer {token}")));
        }
        headers
    }

    /// Downloads one release file. A 5xx is retried twice, then fetched through the API.
    pub fn release_asset(&self, repo: &str, tag: &str, name: &str) -> Result<Vec<u8>, FacileError> {
        let url = format!("{}/{repo}/releases/download/{tag}/{name}", self.web);
        let mut last = None;
        for attempt in 0..DOWNLOAD_ATTEMPTS {
            match self.net.get(&url, &[]) {
                Ok(blob) => return Ok(blob),
                Err(e) if e.code().is_some_and(|c| c >= 500) => last = Some(e),
                Err(e) => return Err(e.into()),
            }
            std::thread::sleep(self.retry_delay * (attempt + 1));
        }
        let last = last.map(|e| e.to_string()).unwrap_or_default();
        self.api_asset(repo, tag, name)
            .map_err(|api| FacileError::Download(format!("{last} (API fallback: {api})")))
    }

    fn api_asset(&self, repo: &str, tag: &str, name: &str) -> Result<Vec<u8>, FacileError> {
        let meta_url = format!("{}/repos/{repo}/releases/tags/{tag}", self.api);
        let meta = self
            .net
            .get(&meta_url, &self.api_headers("application/vnd.github+json"))?;
        let release: ReleaseAssets = serde_json::from_slice(&meta).map_err(|e| {
            FacileError::Download(format!("cannot read the release of {repo}: {e}"))
        })?;
        let asset = release
            .assets
            .iter()
            .find(|a| a.name == name)
            .ok_or_else(|| {
                FacileError::Download(format!("{name} is not an asset of {repo} {tag}"))
            })?;
        let url = format!("{}/repos/{repo}/releases/assets/{}", self.api, asset.id);
        Ok(self
            .net
            .get(&url, &self.api_headers("application/octet-stream"))?)
    }
}

/// The token from `GH_TOKEN`, then `GITHUB_TOKEN`, then `gh auth token`. Empty when none is at hand.
pub fn token_from_env() -> String {
    for name in ["GH_TOKEN", "GITHUB_TOKEN"] {
        if let Ok(token) = std::env::var(name) {
            if !token.trim().is_empty() {
                return token.trim().to_string();
            }
        }
    }
    match Command::new("gh").args(["auth", "token"]).output() {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        _ => String::new(),
    }
}
