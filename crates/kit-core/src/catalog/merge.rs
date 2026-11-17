use super::discover::{discover_from_lists, resolve_single};
use super::load::Loader;
use super::{Manifest, SourceConfig};

/// Lays each layer over the base: a tool shadows the one with the same name, compared case-insensitively,
/// and new tools are appended in order. The base is not changed.
pub fn merge(base: &Manifest, layers: &[Manifest]) -> Manifest {
    let mut out = Manifest {
        version: base.version,
        tools: base.tools.clone(),
        mcp: base.mcp.clone(),
        load_errors: Vec::new(),
    };
    for tool in layers.iter().flat_map(|l| &l.tools) {
        match out
            .tools
            .iter()
            .position(|t| t.name.eq_ignore_ascii_case(&tool.name))
        {
            Some(i) => out.tools[i] = tool.clone(),
            None => out.tools.push(tool.clone()),
        }
    }
    out
}

/// Merges the user's sources over the base. A broken sources file is reported in `load_errors`, never fatal.
pub(crate) fn merge_sources(loader: &Loader, base: Manifest) -> Manifest {
    let cfg = match loader.load_sources() {
        Ok(cfg) => cfg,
        Err(e) => {
            let mut out = base;
            out.load_errors.push(e.to_string());
            return out;
        }
    };
    let (layers, errors) = collect_layers(loader, &cfg, base.version);
    let mut out = if layers.is_empty() {
        base
    } else {
        merge(&base, &layers)
    };
    out.load_errors.extend(errors);
    out
}

fn collect_layers(
    loader: &Loader,
    cfg: &SourceConfig,
    version: i64,
) -> (Vec<Manifest>, Vec<String>) {
    let mut errors = Vec::new();
    let mut layers = Vec::new();
    let layer = |tools| Manifest {
        version,
        tools,
        ..Manifest::default()
    };
    if !cfg.single.is_empty() {
        let mut tools = Vec::new();
        for s in &cfg.single {
            match resolve_single(loader.fetch.as_ref(), s) {
                Ok(t) => tools.push(t),
                Err(e) => errors.push(e.to_string()),
            }
        }
        layers.push(layer(tools));
    }
    if !cfg.lists.is_empty() {
        let (tools, errs) = discover_from_lists(loader.fetch.as_ref(), &loader.cache_dir, cfg);
        errors.extend(errs);
        if !tools.is_empty() {
            layers.push(layer(tools));
        }
    }
    (layers, errors)
}
