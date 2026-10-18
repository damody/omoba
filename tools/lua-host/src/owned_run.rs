//! Blocking native process primitive. Lua owns workflow policy and private-job supervision.
//! Keep the original Child handle until its real exit; never infer success from a vanished PID.
use serde_json::Value;

#[cfg(windows)]
fn validate(params: &Value) -> Result<std::path::PathBuf, String> {
    use std::path::{Component, Path};
    let object=params.as_object().ok_or("expected owned-run object")?;
    for key in object.keys() {
        if !["exe","args","cwd","env","stdout","stderr","ready_file"].contains(&key.as_str()) {
            return Err("unknown owned-run field".into());
        }
    }
    let mut paths=Vec::new();
    for key in ["exe","stdout","stderr","ready_file"] {
        let path=Path::new(super::required_str(params,key)?);
        if !path.is_absolute() || path.components().any(|part| matches!(part,Component::ParentDir)) {
            return Err(format!("owned-run {key} requires an absolute path without parent traversal"));
        }
        paths.push(path.to_path_buf());
    }
    let ready=paths[3].clone();
    // No output may replace another output or the executable, even through a parent junction.
    let mut canonical=Vec::new();
    for (index,path) in paths.iter().enumerate() {
        if index==0 {canonical.push(std::fs::canonicalize(path).map_err(|e|e.to_string())?.to_string_lossy().to_lowercase());continue;}
        if path.try_exists().map_err(|e|e.to_string())? {return Err("owned-run output already exists".into());}
        let parent=path.parent().ok_or("owned-run output requires a parent")?;
        std::fs::create_dir_all(parent).map_err(|e|e.to_string())?;
        let full=std::fs::canonicalize(parent).map_err(|e|e.to_string())?.join(path.file_name().ok_or("owned-run output requires a file name")?);
        canonical.push(full.to_string_lossy().to_lowercase());
    }
    for index in 0..canonical.len() {if canonical[..index].contains(&canonical[index]) {return Err("owned-run paths must be distinct".into());}}
    Ok(ready)
}

#[cfg(windows)]
struct RetireOnError(std::process::Child);
#[cfg(windows)]
impl Drop for RetireOnError {
    fn drop(&mut self) {
        if !matches!(self.0.try_wait(),Ok(Some(_))) {let _=self.0.kill();let _=self.0.wait();}
    }
}

#[cfg(windows)]
pub fn run(params: &Value) -> Result<Value,String> {
    use std::{io::Write,os::windows::io::AsRawHandle};
    use serde_json::json;
    let job=std::env::var("_OMOBA_PRIVATE_WORKFLOW_JOB").ok().filter(|value|!value.is_empty())
        .ok_or("owned run-to-files requires a supervised private workflow")?;
    let proof=super::workflow_supervisor::member(&json!({"job_name":job}))?;
    if proof.get("supervised").and_then(Value::as_bool)!=Some(true) {return Err("owned-run workflow membership rejected".into());}
    let ready_file=validate(params)?;
    let mut child=RetireOnError(super::spawn_command(params,true)?.spawn().map_err(|e|format!("owned-run spawn failed: {e}"))?);
    let exact=super::owned_identity_from_handle(child.0.as_raw_handle() as _,child.0.id())?;
    let identity=json!({"pid":exact["pid"],"executable":exact["path"],"creation_token":exact["creation_token"]});
    let ready=json!({"schema":"omoba-owned-run-ready/1","identity":identity});
    // The adapter retries incomplete JSON only while its original waiter is alive.
    // create_new prevents replacing any existing notice; write failure retires the held child.
    let mut file=std::fs::OpenOptions::new().write(true).create_new(true).open(ready_file).map_err(|e|e.to_string())?;
    file.write_all(&serde_json::to_vec(&ready).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    file.sync_all().map_err(|e|e.to_string())?;drop(file);
    let status=child.0.wait().map_err(|e|format!("owned-run wait failed: {e}"))?;
    // Windows NTSTATUS retains all 32 bits; a high-bit crash must never become zero.
    let exit_code=status.code().ok_or("owned-run result has no exit code")? as u32;
    Ok(json!({"schema":"omoba-owned-run-exit/1","identity":identity,"exit_code":exit_code,"retired":true}))
}

#[cfg(not(windows))]
pub fn run(_: &Value) -> Result<Value,String> {Err("owned run-to-files requires Windows; no shell fallback".into())}

#[cfg(all(test,windows))]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn rejects_unknown_fields_before_file_or_process_work() {
        assert!(validate(&json!({"pid":1})).unwrap_err().contains("unknown"));
    }
    #[test]
    fn rejects_relative_executable_and_parent_traversal() {
        assert!(validate(&json!({"exe":"lua.exe"})).unwrap_err().contains("absolute"));
        assert!(validate(&json!({"exe":"D:/a/../lua.exe"})).unwrap_err().contains("parent"));
    }
}
