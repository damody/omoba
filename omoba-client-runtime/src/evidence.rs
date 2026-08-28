use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use prost::Message;
use serde::Serialize;
use sha2::{Digest, Sha256};

use omoba_core::{game_proto::RendererIpcEnvelope, runtime::FilteredRenderSnapshot};

use crate::{config::ClientRuntimeConfig, replica_host::ReplicaApplyReport, ClientRuntimeError};

#[derive(Debug)]
pub struct EvidenceRecorder {
    root: PathBuf,
    team_id: u32,
}

#[derive(Serialize)]
struct RuntimeManifest<'a> {
    schema_version: u32,
    process_role: &'static str,
    pid: u32,
    player_id: u32,
    team_id: u32,
    server_addr: String,
    presentation_addr: String,
    content_hash: &'a str,
    global_seed_sha256: String,
    executable_path: String,
    executable_sha256: String,
}

#[derive(Serialize)]
struct CheckpointLine {
    team_id: u32,
    replica_tick: u64,
    team_sequence: u64,
    authority_revision: u64,
    pre_repair_hash: String,
    post_repair_hash: String,
}

impl EvidenceRecorder {
    pub fn create(
        config: &ClientRuntimeConfig,
        global_seed: u64,
    ) -> Result<Option<Self>, ClientRuntimeError> {
        if !config.test_mode {
            return Ok(None);
        }
        let Some(base) = config.evidence_dir.as_ref() else {
            return Err(ClientRuntimeError::Config(
                "test mode requires evidence directory".into(),
            ));
        };
        let root = base.join(format!("team-{}-runtime", config.team_id));
        fs::create_dir_all(&root).map_err(io_error)?;
        let executable = std::env::current_exe().map_err(io_error)?;
        let executable_bytes = fs::read(&executable).map_err(io_error)?;
        let manifest = RuntimeManifest {
            schema_version: 1,
            process_role: "external-team-replica-runtime",
            pid: std::process::id(),
            player_id: config.player_id,
            team_id: config.team_id,
            server_addr: config.server_addr.to_string(),
            presentation_addr: config.presentation_bind.to_string(),
            content_hash: &config.content_hash,
            global_seed_sha256: hex_hash(&global_seed.to_be_bytes()),
            executable_path: executable.display().to_string(),
            executable_sha256: hex_hash(&executable_bytes),
        };
        write_json(root.join("manifest.json"), &manifest)?;
        Ok(Some(Self {
            root,
            team_id: config.team_id,
        }))
    }

    pub fn record_wire_frame(&self, bytes: &[u8]) -> Result<(), ClientRuntimeError> {
        append_framed(self.root.join("team-frame.capture"), bytes)
    }

    pub fn record_checkpoint(&self, report: &ReplicaApplyReport) -> Result<(), ClientRuntimeError> {
        let line = CheckpointLine {
            team_id: self.team_id,
            replica_tick: report.replica_tick,
            team_sequence: report.team_sequence,
            authority_revision: report.authority_revision,
            pre_repair_hash: hex::encode(report.pre_repair_hash),
            post_repair_hash: hex::encode(report.post_repair_hash),
        };
        append_json_line(self.root.join("filtered-timeline.jsonl"), &line)
    }

    pub fn record_filtered_world(
        &self,
        snapshot: &FilteredRenderSnapshot,
    ) -> Result<(), ClientRuntimeError> {
        #[derive(Serialize)]
        struct SafeWorld<'a> {
            team_id: u32,
            replica_tick: u64,
            render_ids: Vec<u64>,
            component_schema_ids: Vec<Vec<u32>>,
            #[serde(skip)]
            _source: &'a FilteredRenderSnapshot,
        }
        let value = SafeWorld {
            team_id: snapshot.team_id,
            replica_tick: snapshot.replica_tick,
            render_ids: snapshot.entities.iter().map(|e| e.replica_id).collect(),
            component_schema_ids: snapshot
                .entities
                .iter()
                .map(|e| e.components.keys().copied().collect())
                .collect(),
            _source: snapshot,
        };
        write_json(self.root.join("filtered-world.latest.json"), &value)
    }

    pub fn record_presentation(
        &self,
        envelope: &RendererIpcEnvelope,
    ) -> Result<(), ClientRuntimeError> {
        append_framed(
            self.root.join("presentation.capture"),
            &envelope.encode_to_vec(),
        )
    }
    pub fn record_marker(&self, name: &str, tick: u64) -> Result<(), ClientRuntimeError> {
        fs::write(self.root.join(format!("{name}.tick")), tick.to_string()).map_err(io_error)
    }
}

fn write_json(path: PathBuf, value: &impl Serialize) -> Result<(), ClientRuntimeError> {
    let bytes =
        serde_json::to_vec_pretty(value).map_err(|e| ClientRuntimeError::Ipc(e.to_string()))?;
    fs::write(path, bytes).map_err(io_error)
}

fn append_json_line(path: PathBuf, value: &impl Serialize) -> Result<(), ClientRuntimeError> {
    let mut file = append_file(&path)?;
    serde_json::to_writer(&mut file, value).map_err(|e| ClientRuntimeError::Ipc(e.to_string()))?;
    file.write_all(b"\n").map_err(io_error)
}

fn append_framed(path: PathBuf, bytes: &[u8]) -> Result<(), ClientRuntimeError> {
    let mut file = append_file(&path)?;
    file.write_all(&(bytes.len() as u32).to_be_bytes())
        .map_err(io_error)?;
    file.write_all(bytes).map_err(io_error)
}

fn append_file(path: &Path) -> Result<std::fs::File, ClientRuntimeError> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(io_error)
}

fn hex_hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
fn io_error(error: std::io::Error) -> ClientRuntimeError {
    ClientRuntimeError::Ipc(error.to_string())
}
