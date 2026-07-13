use crate::leader_slots::LeaderSlotsSnapshotArtifact;
use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::Path,
};

pub(crate) fn read_ndjson<T: DeserializeOwned>(path: &Path) -> Result<Vec<T>> {
    let mut rows = Vec::new();
    visit_ndjson(path, |row| {
        rows.push(row);
        Ok(())
    })?;
    Ok(rows)
}

pub(crate) fn visit_ndjson<T: DeserializeOwned>(
    path: &Path,
    mut visitor: impl FnMut(T) -> Result<()>,
) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    for (line_no, line) in BufReader::new(file).lines().enumerate() {
        let line = line.with_context(|| format!("read {}:{}", path.display(), line_no + 1))?;
        if line.trim().is_empty() {
            continue;
        }
        let row = serde_json::from_str(&line)
            .with_context(|| format!("parse {}:{}", path.display(), line_no + 1))?;
        visitor(row)?;
    }
    Ok(())
}

pub(crate) fn read_leader_slot_snapshots(
    artifact_dir: &Path,
) -> Result<Vec<LeaderSlotsSnapshotArtifact>> {
    let mut paths = fs::read_dir(artifact_dir)
        .with_context(|| format!("read {}", artifact_dir.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| is_leader_snapshot(path))
        .collect::<Vec<_>>();
    paths.sort();

    paths
        .into_iter()
        .map(|path| {
            let bytes = fs::read(&path).with_context(|| format!("read {}", path.display()))?;
            serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))
        })
        .collect()
}

fn is_leader_snapshot(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| {
            name == "leader-slots-snapshot.json"
                || (name.starts_with("leader-slots-snapshot-") && name.ends_with(".json"))
        })
        .unwrap_or(false)
}
