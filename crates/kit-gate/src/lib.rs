//! Definition-of-done gate and blast-radius firewall.
//!
//! M0 SCAFFOLD. The real engine is a port of Guardian's `guard.js` (546 lines)
//! and `done-gate.js` (154 lines), landing in M3. Guardian's fixture suite at
//! `grok-build-guardian/tests/firewall.test.js` is the acceptance oracle —
//! every fixture must pass before this crate is considered done.
//!
//! Until then `KitGate` is deliberately inert: it claims no proof it did not
//! produce, and it screens nothing. `kit-cli` must not present its results as
//! a passing gate.

use kit_core::{FirewallVerdict, Gate, GateConfig, GateOutcome};
use std::path::Path;

/// The gate engine. Inert until the Guardian port lands (M3).
#[derive(Debug, Clone, Default)]
pub struct KitGate {
    _private: (),
}

impl KitGate {
    pub fn new() -> Self {
        Self::default()
    }

    /// True once the Guardian port has landed. Callers must check this before
    /// reporting a gate result, so an unported gate can never look like proof.
    pub fn is_implemented(&self) -> bool {
        false
    }
}

#[async_trait::async_trait]
impl Gate for KitGate {
    async fn evaluate(&self, _worktree: &Path, _config: &GateConfig) -> GateOutcome {
        GateOutcome::vacuous()
    }

    fn screen(&self, _command: &str) -> FirewallVerdict {
        FirewallVerdict::Allow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaffold_admits_it_is_not_implemented() {
        assert!(!KitGate::new().is_implemented());
    }

    #[tokio::test]
    async fn vacuous_outcome_claims_no_checks() {
        let out = KitGate::new()
            .evaluate(Path::new("."), &GateConfig::default())
            .await;
        assert!(out.checks.is_empty());
        assert!(out.first_failure().is_none());
    }
}
