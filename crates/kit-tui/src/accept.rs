//! Review-branch acceptance delegates to the installed Kit CLI, never to git here.
use kit_core::RunId;
use serde_json::Value;
use std::{path::Path, process::Stdio};
use tokio::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Accepted {
    pub branch: String,
    pub commit: String,
}

fn command(executable: &Path, id: &RunId) -> Command {
    let mut cmd = Command::new(executable);
    cmd.args(["land", "--json", "--", &id.0])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    cmd
}

fn parse(id: &RunId, success: bool, bytes: &[u8]) -> Result<Accepted, String> {
    let v: Value = serde_json::from_slice(bytes)
        .map_err(|_| "Kit land returned malformed JSON".to_string())?;
    if v["schemaVersion"] != 1
        || v["command"] != "land"
        || !v["warnings"]
            .as_array()
            .is_some_and(|warnings| warnings.iter().all(Value::is_string))
        || v.get("error").is_none()
        || !v["ok"].is_boolean()
    {
        return Err("Kit land returned an invalid envelope".into());
    }
    if v["ok"] == false {
        return Err(v["error"]
            .as_str()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("Kit land failed without a reason")
            .to_string());
    }
    let d = &v["data"];
    if !success
        || !v["error"].is_null()
        || d["id"] != id.0
        || d["mode"] != "branch"
        || d["forced"] != false
    {
        return Err("Kit land did not confirm an unforced review branch for this run".into());
    }
    let branch = d["branch"]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or("Kit land omitted the branch")?;
    if branch != format!("kit/{}", id.0.chars().take(12).collect::<String>()) {
        return Err("Kit land returned an unexpected review branch".into());
    }
    let commit = d["commit"]
        .as_str()
        .filter(|s| matches!(s.len(), 40 | 64) && s.bytes().all(|b| b.is_ascii_hexdigit()))
        .ok_or("Kit land omitted a valid commit")?;
    Ok(Accepted {
        branch: branch.into(),
        commit: commit.into(),
    })
}

pub(crate) async fn land(
    id: &RunId,
    runs_dir: &Path,
    reviewed_diff: &str,
) -> Result<Accepted, String> {
    let saved = tokio::fs::read_to_string(runs_dir.join(&id.0).join("diff.patch"))
        .await
        .map_err(|e| format!("Cannot recheck reviewed diff: {e}"))?;
    if saved != reviewed_diff {
        return Err("The saved diff changed or was truncated. Reopen it before accepting.".into());
    }
    let executable = std::env::current_exe().map_err(|e| format!("Cannot locate Kit: {e}"))?;
    let output = command(&executable, id)
        .output()
        .await
        .map_err(|e| format!("Cannot start Kit land: {e}"))?;
    parse(id, output.status.success(), &output.stdout)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn envelope() -> Value {
        serde_json::json!({"schemaVersion":1,"command":"land","ok":true,"error":null,"warnings":[],"data":{"id":"01TEST","mode":"branch","forced":false,"branch":"kit/01TEST","commit":"a".repeat(40)}})
    }
    #[test]
    fn accepts_only_exact_unforced_successful_branch_receipt() {
        let id = RunId("01TEST".into());
        let v = envelope();
        assert_eq!(
            parse(&id, true, &serde_json::to_vec(&v).unwrap())
                .unwrap()
                .branch,
            "kit/01TEST"
        );
        for (key, value) in [
            ("forced", Value::Bool(true)),
            ("mode", Value::String("apply".into())),
            ("id", Value::String("another".into())),
            ("commit", Value::String("bad".into())),
            ("branch", Value::Null),
            ("branch", Value::String("another-branch".into())),
        ] {
            let mut v = envelope();
            v["data"][key] = value;
            assert!(parse(&id, true, &serde_json::to_vec(&v).unwrap()).is_err());
        }
        assert!(parse(&id, false, &serde_json::to_vec(&v).unwrap()).is_err());
        assert!(parse(&id, true, b"not JSON").is_err());
        let mut wrong = envelope();
        wrong["command"] = Value::String("run".into());
        assert!(parse(&id, true, &serde_json::to_vec(&wrong).unwrap()).is_err());
    }
    #[tokio::test]
    async fn changed_saved_diff_is_rejected_before_any_cli_invocation() {
        let tmp = crate::past::tests::TempRuns::new("land-diff-recheck");
        let id = RunId("01TEST".into());
        let dir = tmp.0.join(&id.0);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("diff.patch"), "new patch").unwrap();
        assert_eq!(
            land(&id, &tmp.0, "reviewed patch").await,
            Err("The saved diff changed or was truncated. Reopen it before accepting.".into())
        );
    }

    #[test]
    fn failure_reason_is_preserved_and_cli_arguments_cannot_force_or_apply() {
        let id = RunId("01TEST".into());
        let v = serde_json::json!({"schemaVersion":1,"command":"land","ok":false,"error":"gate failed","warnings":[],"data":null});
        assert_eq!(
            parse(&id, false, &serde_json::to_vec(&v).unwrap()),
            Err("gate failed".into())
        );
        let c = command(Path::new("kit"), &id);
        assert_eq!(
            c.as_std().get_args().collect::<Vec<_>>(),
            ["land", "--json", "--", "01TEST"]
        );
    }
}
