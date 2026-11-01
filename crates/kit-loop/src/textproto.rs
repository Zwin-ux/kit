//! The text protocol for models that cannot call tools (PRD §6.3, the
//! mini-SWE-agent design): one fenced shell block per reply.

/// The command in the first fenced block tagged `shell` (or `sh`, `bash`,
/// `powershell`, `ps1`), if the reply has one.
pub fn command(reply: &str, shell: &str) -> Option<String> {
    let tags: &[&str] = if shell == "powershell" {
        &["powershell", "ps1", "pwsh"]
    } else {
        &["bash", "sh", "shell"]
    };
    let text = reply.replace("\r\n", "\n");
    let mut rest = text.as_str();
    while let Some(start) = rest.find("```") {
        let after = &rest[start + 3..];
        let (tag, body) = after.split_once('\n')?;
        let end = body.find("```")?;
        if tags.contains(&tag.trim().to_ascii_lowercase().as_str()) {
            let cmd = body[..end].trim();
            return (!cmd.is_empty()).then(|| cmd.to_owned());
        }
        rest = &body[end + 3..];
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_shell_block_only() {
        let reply = "Look:\n```rust\nfn x() {}\n```\nNow run:\r\n```bash\ncargo test\n```\n```bash\nrm -rf x\n```";
        assert_eq!(command(reply, "bash").as_deref(), Some("cargo test"));
        assert_eq!(
            command("```powershell\nGet-ChildItem\n```", "powershell").as_deref(),
            Some("Get-ChildItem")
        );
        assert_eq!(command("Done. No block.", "bash"), None);
        assert_eq!(command("```bash\n\n```", "bash"), None);
    }
}
