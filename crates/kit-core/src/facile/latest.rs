use std::collections::HashMap;

use serde::Deserialize;

use super::github::Github;
use super::FacileError;

#[derive(Deserialize)]
struct LatestRelease {
    #[serde(default)]
    tag_name: String,
}

impl Github {
    /// The latest release tag of `owner/repo`, through the `/releases/latest` redirect.
    ///
    /// The redirect is tried twice, then the API answers. A repository kit cannot read looks like one
    /// with no release, so that error names both causes.
    pub fn latest_tag(&self, repo: &str) -> Result<String, FacileError> {
        let mut last = None;
        for _ in 0..2 {
            match self.redirect_tag(repo) {
                Ok(tag) => return Ok(tag),
                Err(e @ FacileError::NoRelease(_)) => return Err(e),
                Err(e) => last = Some(e),
            }
        }
        let last = last.map(|e| e.to_string()).unwrap_or_default();
        self.api_latest_tag(repo)
            .map_err(|api| FacileError::Download(format!("{last} (API fallback: {api})")))
    }

    /// Every repo mapped to its latest tag, resolved in parallel. A failed lookup maps to an empty tag.
    pub fn latest(&self, repos: &[String]) -> HashMap<String, String> {
        std::thread::scope(|scope| {
            let handles: Vec<_> = repos
                .iter()
                .map(|repo| scope.spawn(move || self.latest_tag(repo).unwrap_or_default()))
                .collect();
            repos
                .iter()
                .zip(handles)
                .map(|(repo, h)| (repo.clone(), h.join().unwrap_or_default()))
                .collect()
        })
    }

    fn redirect_tag(&self, repo: &str) -> Result<String, FacileError> {
        let url = format!("{}/{repo}/releases/latest", self.web);
        let head = self.net.head(&url)?;
        if head.status >= 500 {
            return Err(super::net::HttpError::Status {
                name: format!("{repo} releases/latest"),
                code: head.status,
            }
            .into());
        }
        let path = head.url.split(['?', '#']).next().unwrap_or_default();
        match path.split_once("/releases/tag/") {
            Some((_, tag)) if !tag.is_empty() => Ok(tag.to_string()),
            _ => Err(FacileError::NoRelease(format!(
                "cannot read a release tag for {repo} (status {}): check that the repository is public \
                 and has a release published",
                head.status
            ))),
        }
    }

    fn api_latest_tag(&self, repo: &str) -> Result<String, FacileError> {
        let url = format!("{}/repos/{repo}/releases/latest", self.api);
        let body = self
            .net
            .get(&url, &self.api_headers("application/vnd.github+json"))?;
        match serde_json::from_slice::<LatestRelease>(&body) {
            Ok(r) if !r.tag_name.is_empty() => Ok(r.tag_name),
            _ => Err(FacileError::Download(format!(
                "cannot read the latest release of {repo}"
            ))),
        }
    }
}
