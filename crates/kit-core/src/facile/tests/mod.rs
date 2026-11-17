use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::net::base_name;
use super::{Backend, Env, Github, Head, HttpError, Net};

mod build_fallback;
mod cache;
mod end_to_end;
mod installer;
mod release;
mod self_update;
mod skill;
mod store;
mod update;
mod update_more;
mod version;

pub(super) const HOST: &str = "http://fake";

type Handler = Box<dyn Fn(&str, &str, &[(&str, String)]) -> (u16, Vec<u8>) + Send + Sync>;

pub(super) struct FakeNet {
    handler: Handler,
    calls: Arc<Mutex<Vec<String>>>,
}

impl Net for FakeNet {
    fn get(&self, url: &str, headers: &[(&str, String)]) -> Result<Vec<u8>, HttpError> {
        let path = url.trim_start_matches(HOST).to_string();
        self.calls.lock().unwrap().push(format!("GET {path}"));
        match (self.handler)("GET", &path, headers) {
            (200, body) => Ok(body),
            (code, _) => Err(HttpError::Status {
                name: base_name(url),
                code,
            }),
        }
    }

    fn head(&self, url: &str) -> Result<Head, HttpError> {
        let path = url.trim_start_matches(HOST).to_string();
        self.calls.lock().unwrap().push(format!("HEAD {path}"));
        let (status, body) = (self.handler)("HEAD", &path, &[]);
        Ok(Head {
            status,
            url: format!("{HOST}{}", String::from_utf8_lossy(&body)),
        })
    }
}

pub(super) fn github<F>(handler: F) -> (Github, Arc<Mutex<Vec<String>>>)
where
    F: Fn(&str, &str, &[(&str, String)]) -> (u16, Vec<u8>) + Send + Sync + 'static,
{
    let calls = Arc::new(Mutex::new(Vec::new()));
    let net = FakeNet {
        handler: Box::new(handler),
        calls: calls.clone(),
    };
    let mut gh = Github::new(Box::new(net), Box::new(String::new));
    gh.web = HOST.into();
    gh.api = HOST.into();
    gh.retry_delay = Duration::ZERO;
    (gh, calls)
}

pub(super) fn backend<F>(home: &Path, handler: F) -> (Backend, Arc<Mutex<Vec<String>>>)
where
    F: Fn(&str, &str, &[(&str, String)]) -> (u16, Vec<u8>) + Send + Sync + 'static,
{
    let (github, calls) = github(handler);
    let env = Env {
        home: home.to_path_buf(),
        data_home: None,
        path: Default::default(),
        cache_dir: home.join("cache"),
    };
    (Backend { github, env }, calls)
}

pub(super) fn offline(_: &str, _: &str, _: &[(&str, String)]) -> (u16, Vec<u8>) {
    (404, Vec::new())
}

pub(super) fn tar_gz(members: &[(&str, &str)]) -> Vec<u8> {
    let gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    let mut tw = tar::Builder::new(gz);
    for (name, body) in members {
        let mut header = tar::Header::new_gnu();
        header.set_size(body.len() as u64);
        header.set_mode(0o755);
        header.set_entry_type(tar::EntryType::Regular);
        tw.append_data(&mut header, name, body.as_bytes()).unwrap();
    }
    let mut gz = tw.into_inner().unwrap();
    gz.flush().unwrap();
    gz.finish().unwrap()
}

pub(super) fn stub_binary(dir: &Path, bin: &str, line: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    fs::create_dir_all(dir).unwrap();
    let path = dir.join(bin);
    fs::write(&path, format!("#!/bin/sh\necho '{line}'\n")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    path
}
