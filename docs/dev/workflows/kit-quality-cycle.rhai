let meta = #{
    name: "kit-quality-cycle",
    description: "Long-running Kit 1.0 quality ladder: audit → adversarial verify → plan → optional implement → cargo verify → report",
    when_to_use: "Drive high-quality Kit toward 1.0: multi-agent audit, skeptic verify, ordered plan, implement top slices, re-test",
    phases: [
        #{ title: "Truth", detail: "honest status vs PRD/CURRENT/CI/PR" },
        #{ title: "Audit", detail: "parallel quality shards (surface, engine, gate, CLI, tests, security, docs)" },
        #{ title: "Skeptic", detail: "adversarial verify top findings" },
        #{ title: "Plan", detail: "CEO-style ordered backlog with kill criteria" },
        #{ title: "Implement", detail: "optional Power slices (gated)" },
        #{ title: "Prove", detail: "cargo test/clippy evidence" },
        #{ title: "Report", detail: "scratch report + next cycle brief" },
    ],
};

// --- schemas ---
let truth_schema = #{
    "type": "object",
    "required": ["branch_reality", "prd_gaps", "ci_risks", "merge_readiness", "top_blockers"],
    "properties": #{
        "branch_reality": #{ "type": "string" },
        "prd_gaps": #{ "type": "array", "maxItems": 12, "items": #{ "type": "string" } },
        "ci_risks": #{ "type": "array", "maxItems": 8, "items": #{ "type": "string" } },
        "merge_readiness": #{ "type": "string" },
        "top_blockers": #{ "type": "array", "maxItems": 8, "items": #{ "type": "string" } },
    },
};

let audit_schema = #{
    "type": "object",
    "required": ["shard", "score_0_to_10", "findings", "must_fix", "nice_to_have"],
    "properties": #{
        "shard": #{ "type": "string" },
        "score_0_to_10": #{ "type": "integer" },
        "findings": #{
            "type": "array", "maxItems": 10,
            "items": #{
                "type": "object",
                "required": ["title", "severity", "detail", "path"],
                "properties": #{
                    "title": #{ "type": "string" },
                    "severity": #{ "type": "string" },
                    "detail": #{ "type": "string" },
                    "path": #{ "type": "string" },
                },
            },
        },
        "must_fix": #{ "type": "array", "maxItems": 6, "items": #{ "type": "string" } },
        "nice_to_have": #{ "type": "array", "maxItems": 6, "items": #{ "type": "string" } },
    },
};

let skeptic_schema = #{
    "type": "object",
    "required": ["claim", "real", "evidence", "severity_if_real"],
    "properties": #{
        "claim": #{ "type": "string" },
        "real": #{ "type": "boolean" },
        "evidence": #{ "type": "string" },
        "severity_if_real": #{ "type": "string" },
        "suggested_fix": #{ "type": "string" },
    },
};

let plan_schema = #{
    "type": "object",
    "required": ["summary", "ordered_slices", "kill_criteria", "non_goals"],
    "properties": #{
        "summary": #{ "type": "string" },
        "ordered_slices": #{
            "type": "array", "maxItems": 12,
            "items": #{
                "type": "object",
                "required": ["id", "title", "owner", "why", "acceptance", "verify", "files"],
                "properties": #{
                    "id": #{ "type": "string" },
                    "title": #{ "type": "string" },
                    "owner": #{ "type": "string" },
                    "why": #{ "type": "string" },
                    "acceptance": #{ "type": "string" },
                    "verify": #{ "type": "string" },
                    "files": #{ "type": "string" },
                },
            },
        },
        "kill_criteria": #{ "type": "array", "maxItems": 8, "items": #{ "type": "string" } },
        "non_goals": #{ "type": "array", "maxItems": 8, "items": #{ "type": "string" } },
    },
};

let implement_schema = #{
    "type": "object",
    "required": ["slice_id", "done", "summary", "tests_run", "files_touched"],
    "properties": #{
        "slice_id": #{ "type": "string" },
        "done": #{ "type": "boolean" },
        "summary": #{ "type": "string" },
        "tests_run": #{ "type": "string" },
        "files_touched": #{ "type": "array", "maxItems": 20, "items": #{ "type": "string" } },
        "follow_up": #{ "type": "string" },
    },
};

let prove_schema = #{
    "type": "object",
    "required": ["tests_ok", "clippy_ok", "notes", "commands"],
    "properties": #{
        "tests_ok": #{ "type": "boolean" },
        "clippy_ok": #{ "type": "boolean" },
        "notes": #{ "type": "string" },
        "commands": #{ "type": "array", "maxItems": 10, "items": #{ "type": "string" } },
    },
};

// --- args ---
// mode: audit | plan | implement | full  (default full)
// focus: all | surface | engine | ship  (default all)
// max_implement: number of slices to implement (default 2)
// auto_implement: true to skip await_user before implement (default false)
let mode = "full";
let focus = "all";
let max_implement = 2;
let auto_implement = false;
if args != () {
    if args.mode != () { mode = args.mode; }
    if args.focus != () { focus = args.focus; }
    if args.max_implement != () { max_implement = args.max_implement; }
    if args.auto_implement == true { auto_implement = true; }
}

log("kit-quality-cycle mode=" + mode + " focus=" + focus + " max_implement=" + max_implement.to_string());

// ============================================================================
// Phase 1 — Truth
// ============================================================================
phase("Truth");
let truth_prompt = "";
truth_prompt += "You are the Kit quality scout (read-only). Establish HONEST status for a high-quality 1.0.\n";
truth_prompt += "Read with tools (not memory): docs/dev/CURRENT.md, docs/dev/PRD-1.0.md, docs/dev/SPEC-surface-1.0.md, ";
truth_prompt += "docs/dev/DESIGN-tui.md, docs/dev/tasks/CEO-STAMP-P1.md, tasks/todo.md, tasks/plan.md, ";
truth_prompt += "docs/json-contract.md, Agents.md / AGENTS.md.\n";
truth_prompt += "If available use git status/log and note open PR #11 branch f2-f3-control-room-detail.\n";
truth_prompt += "Focus lens: " + focus + ".\n";
truth_prompt += "Report: what is actually shipped vs claimed; PRD gaps; CI/merge risks; top blockers.\n";
truth_prompt += "Never invent green status. Empty evidence = risk.\n";

let truth = agent(truth_prompt, #{
    label: "truth-scout",
    capability_mode: "read-only",
    output_schema: truth_schema,
});

let truth_blockers = [];
let branch_reality = "(truth agent failed)";
let merge_readiness = "unknown";
if truth != () && truth.success && truth.output != () {
    branch_reality = truth.output.branch_reality;
    merge_readiness = truth.output.merge_readiness;
    if truth.output.top_blockers != () {
        for b in truth.output.top_blockers { truth_blockers.push(b); }
    }
}
log("Truth: merge_readiness=" + merge_readiness);

// ============================================================================
// Phase 2 — Parallel audits
// ============================================================================
phase("Audit");
let audits = [];

// Always include core shards; filter by focus.
let want_surface = focus == "all" || focus == "surface";
let want_engine = focus == "all" || focus == "engine";
let want_ship = focus == "all" || focus == "ship";

if want_surface {
    audits.push(#{
        id: "surface-tui",
        prompt: "Audit kit-tui craft for Kit 1.0 quality. READ crates/kit-tui/src (theme.rs, ui/*, app.rs), docs/dev/DESIGN-tui.md, docs/dev/SPEC-surface-1.0.md, and snapshots. Score 0-10 vs concept art density, semantic colors, empty states, help overlay, monochrome NO_COLOR, footer grammar. List must_fix with paths. Use read_file/grep only.",
    });
    audits.push(#{
        id: "surface-ux",
        prompt: "UX audit of Kit Control Room interaction language. Read SPEC-surface-1.0.md and app.rs key handlers for Control Room, Detail, Dispatch, Board, Attached. Find inconsistencies (q/Esc/?, dead keys advertised, attach stub honesty). Schema findings with severity.",
    });
}

if want_engine {
    audits.push(#{
        id: "engine-p3",
        prompt: "Audit P3 concurrency proof readiness. Read crates/kit-cli/src/engine (registry, runner, main supervisor), MAX_CONCURRENT_RUNS, and tests. Is there a harness that dispatches 12 and proves <=8 running? Gap analysis for CEO kill criteria. Paths + must_fix.",
    });
    audits.push(#{
        id: "engine-kill",
        prompt: "Audit kill/retry/timeout honesty. Read kit-agents process.rs tree kill, kit-cli cancel/registry, kit-tui KillSelected/RetrySelected. List remaining zombie/Windows risks and missing tests. Evidence from code only.",
    });
    audits.push(#{
        id: "gate-p2",
        prompt: "Audit vacuous gate + inference. Read kit-cli engine/infer.rs, runner gate phase, kit-tui format_gate_label UNCONFIGURED, CEO-STAMP-P1. Is demo deliberate FAIL still open? Inference conservative? must_fix list.",
    });
}

if want_ship || want_engine {
    audits.push(#{
        id: "cli-json",
        prompt: "Audit CLI JSON + doctor. Read crates/kit-cli/src/main.rs json_envelope, docs/json-contract.md. Confirm schemaVersion 1, warnings array, camelCase. Gaps: receipt show/list, security flags. Findings only from code.",
    });
}

audits.push(#{
    id: "tests-ci",
    prompt: "Audit test/CI quality bar. Read .github/workflows for Rust CI, kit-tui insta snapshots, kit-cli/kit-agents tests. List flaky risks, missing proof tests, Windows-only assumptions. Score 0-10 CI confidence.",
});

audits.push(#{
    id: "security",
    prompt: "Security/hardening pass for Kit 1.0 control room. Read firewall in kit-gate, bounds, KIT_FULL_AUTO, worktree paths, no credential custody claims in PRD. List real risks (path escape, command injection in gate commands, full-auto). Severity ranked.",
});

if want_ship {
    audits.push(#{
        id: "ship-p5",
        prompt: "Distribution readiness audit. Search for installers, npm packages, README install path, M5 notes in PRD. What blocks clean-machine install kill criterion? Concrete gaps only.",
    });
}

let ajobs = [];
for a in audits {
    let p = "";
    p += "You are a Kit Luna quality auditor (read-only). Shard id: " + a.id + ".\n";
    p += a.prompt + "\n";
    p += "Hard rules: use tools (read_file, grep, list_dir). Do not invent file contents. ";
    p += "score_0_to_10 is your confidence this area is 1.0-ready. must_fix are blocking. ";
    p += "Output only schema fields. Empty findings only if you inspected code and found none.\n";
    ajobs.push(#{
        prompt: p,
        label: "audit:" + a.id,
        capability_mode: "read-only",
        output_schema: audit_schema,
    });
}

let audit_results = parallel(ajobs);

// Collect findings
let all_findings = [];
let shard_scores = [];
let i = 0;
for r in audit_results {
    let sid = audits[i].id;
    if r != () && r.success && r.output != () {
        let sc = r.output.score_0_to_10;
        shard_scores.push(sid + "=" + sc.to_string());
        if r.output.findings != () {
            for f in r.output.findings {
                let sev = f.severity;
                let title = f.title;
                let path = f.path;
                let detail = f.detail;
                all_findings.push(#{
                    shard: sid,
                    severity: sev,
                    title: title,
                    path: path,
                    detail: detail,
                });
            }
        }
        if r.output.must_fix != () {
            for m in r.output.must_fix {
                all_findings.push(#{
                    shard: sid,
                    severity: "high",
                    title: m,
                    path: "",
                    detail: "must_fix from " + sid,
                });
            }
        }
    } else {
        log("audit failed: " + sid);
        all_findings.push(#{
            shard: sid,
            severity: "high",
            title: "audit shard failed or empty",
            path: "",
            detail: "fail-closed: treat as unverified quality risk",
        });
    }
    i += 1;
}
log("Audits produced " + all_findings.len().to_string() + " raw findings; scores: " + shard_scores.len().to_string() + " shards");

// Cap findings for skeptic panel (budget)
let MAX_SKEPTICS = 12;
let skeptic_pool = [];
for f in all_findings {
    if skeptic_pool.len() >= MAX_SKEPTICS { break; }
    // Prefer high/critical
    let sev = f.severity;
    if sev == "critical" || sev == "high" || sev == "HIGH" || sev == "CRITICAL" || skeptic_pool.len() < 6 {
        skeptic_pool.push(f);
    }
}
// If still thin, fill with remaining
if skeptic_pool.len() < 6 {
    for f in all_findings {
        if skeptic_pool.len() >= MAX_SKEPTICS { break; }
        let already = false;
        for s in skeptic_pool {
            if s.title == f.title && s.shard == f.shard { already = true; }
        }
        if !already { skeptic_pool.push(f); }
    }
}

// ============================================================================
// Phase 3 — Adversarial verify
// ============================================================================
phase("Skeptic");
let vjobs = [];
for f in skeptic_pool {
    let claim = f.title + " [" + f.shard + "] " + f.detail;
    if f.path != "" { claim += " @" + f.path; }
    let vp = "";
    vp += "You are an adversarial skeptic for Kit quality claims. FAIL CLOSED.\n";
    vp += "Claim: " + claim + "\n";
    vp += "Independently inspect the codebase with read_file/grep. Set real=true ONLY with concrete evidence you read. ";
    vp += "If you cannot verify, real=false. evidence must quote paths/symbols you saw. suggested_fix if real.\n";
    vjobs.push(#{
        prompt: vp,
        label: "skeptic",
        capability_mode: "read-only",
        output_schema: skeptic_schema,
    });
}

let confirmed = [];
if vjobs.len() > 0 {
    let verdicts = parallel(vjobs);
    let vi = 0;
    for v in verdicts {
        if v != () && v.success && v.output != () && v.output.real == true {
            if v.output.evidence != () && v.output.evidence != "" {
                let base = skeptic_pool[vi];
                confirmed.push(#{
                    title: base.title,
                    shard: base.shard,
                    path: base.path,
                    severity: v.output.severity_if_real,
                    evidence: v.output.evidence,
                    fix: v.output.suggested_fix,
                });
            }
        }
        vi += 1;
    }
}
log("Skeptics confirmed " + confirmed.len().to_string() + "/" + skeptic_pool.len().to_string() + " claims");

if mode == "audit" {
    let report = "";
    report += "# Kit quality cycle — AUDIT ONLY\n\n";
    report += "## Truth\n" + branch_reality + "\n\nmerge_readiness: " + merge_readiness + "\n\n";
    report += "## Confirmed findings (" + confirmed.len().to_string() + ")\n";
    for c in confirmed {
        report += "- **" + c.title + "** (" + c.shard + ") — " + c.evidence + "\n";
        if c.fix != () && c.fix != "" { report += "  - fix: " + c.fix + "\n"; }
    }
    report += "\n## Top blockers (truth)\n";
    for b in truth_blockers { report += "- " + b + "\n"; }
    let path = write_scratch_file("kit-quality-audit.md", report);
    complete(#{ mode: "audit", path: path, confirmed: confirmed.len(), merge_readiness: merge_readiness });
}

// ============================================================================
// Phase 4 — Plan
// ============================================================================
phase("Plan");
let plan_prompt = "";
plan_prompt += "You are Kit CEO-style planner (read-only). Produce an ordered implementation backlog for high-quality 1.0.\n";
plan_prompt += "Truth merge_readiness: " + merge_readiness + "\n";
plan_prompt += "Branch reality: " + branch_reality + "\n";
plan_prompt += "Confirmed verified findings (JSON-ish list follows). Use ONLY these + CURRENT.md priorities.\n";
plan_prompt += "Rules: Power owns kit-tui; Codex-style engine work can be listed owner=Factory; never edit kit-core contracts without owner=CEO.\n";
plan_prompt += "Non-goals: marketplace, PTY attach (1.0.1), mascot before P3, Power merging own PR.\n";
plan_prompt += "Each slice: small, verifiable, files limited, acceptance + verify command.\n";
plan_prompt += "Priority order: land PR/CI green, P3 concurrency proof, remaining gate/demo FAIL, surface polish gaps, ship path.\n";
plan_prompt += "Confirmed count: " + confirmed.len().to_string() + "\n";
for c in confirmed {
    plan_prompt += "FINDING: " + c.title + " | shard=" + c.shard + " | path=" + c.path + " | fix=" + c.fix + " | evidence=" + c.evidence + "\n";
}
for b in truth_blockers {
    plan_prompt += "BLOCKER: " + b + "\n";
}

let plan = agent(plan_prompt, #{
    label: "quality-plan",
    capability_mode: "read-only",
    output_schema: plan_schema,
});

let slices = [];
let plan_summary = "plan failed";
let kill_criteria = [];
if plan != () && plan.success && plan.output != () {
    plan_summary = plan.output.summary;
    if plan.output.ordered_slices != () {
        for s in plan.output.ordered_slices { slices.push(s); }
    }
    if plan.output.kill_criteria != () {
        for k in plan.output.kill_criteria { kill_criteria.push(k); }
    }
}
log("Plan slices: " + slices.len().to_string() + " — " + plan_summary);

if mode == "plan" {
    let report = "";
    report += "# Kit quality cycle — PLAN\n\n";
    report += plan_summary + "\n\n## Ordered slices\n";
    for s in slices {
        report += "### " + s.id + " — " + s.title + "\n";
        report += "- owner: " + s.owner + "\n- why: " + s.why + "\n";
        report += "- acceptance: " + s.acceptance + "\n- verify: " + s.verify + "\n";
        report += "- files: " + s.files + "\n\n";
    }
    report += "## Kill criteria\n";
    for k in kill_criteria { report += "- " + k + "\n"; }
    let path = write_scratch_file("kit-quality-plan.md", report);
    complete(#{ mode: "plan", path: path, slices: slices.len(), summary: plan_summary });
}

// ============================================================================
// Phase 5 — Implement (optional, gated)
// ============================================================================
phase("Implement");
let implemented = [];

if mode == "implement" || mode == "full" {
    if !auto_implement {
        await_user("verification", "Review the quality plan. Resume to implement up to max_implement Power-owned slices (kit-tui / kit-cli wiring only). Cancel if you only wanted audit/plan.");
    }

    let n = 0;
    for s in slices {
        if n >= max_implement { break; }
        // Skip CEO-only / Factory-only labels if present — Power implements Power work
        let owner = s.owner;
        let owner_l = owner;
        // Prefer Power/Grok slices; still allow Factory if mode implement
        let ip = "";
        ip += "You are Grok Power implementing ONE Kit quality slice. Scope discipline.\n";
        ip += "Slice id: " + s.id + "\nTitle: " + s.title + "\n";
        ip += "Why: " + s.why + "\nAcceptance: " + s.acceptance + "\n";
        ip += "Verify: " + s.verify + "\nFiles hint: " + s.files + "\n";
        ip += "Hard rules:\n";
        ip += "1. Do NOT edit kit-core contracts, kit-tui event.rs, or frozen gate trait signatures.\n";
        ip += "2. Touch only files needed for this slice (~5 max ideal).\n";
        ip += "3. Run cargo test / clippy for affected crates after changes.\n";
        ip += "4. If blocked by CEO contract need, set done=false and explain in follow_up.\n";
        ip += "5. Prefer boring correct code over clever.\n";
        ip += "6. Update tasks/todo.md checkboxes only if you completed the item.\n";
        ip += "Implement now with tools. Report schema honestly.\n";

        let r = agent(ip, #{
            label: "impl:" + s.id,
            capability_mode: "all",
            output_schema: implement_schema,
        });

        if r != () && r.success && r.output != () {
            implemented.push(r.output);
            log("Implement " + s.id + " done=" + r.output.done.to_string());
        } else {
            log("Implement failed: " + s.id);
            implemented.push(#{
                slice_id: s.id,
                done: false,
                summary: "agent failed",
                tests_run: "",
                files_touched: [],
                follow_up: "retry slice",
            });
        }
        n += 1;
    }
} else {
    log("Skipping implement (mode=" + mode + ")");
}

// ============================================================================
// Phase 6 — Prove
// ============================================================================
phase("Prove");
let prove_prompt = "";
prove_prompt += "You are Kit verification Power. Run quality gates and report evidence.\n";
prove_prompt += "Commands to run (shell):\n";
prove_prompt += "  cargo test -p kit-tui -p kit-cli -p kit-agents --all-targets\n";
prove_prompt += "  cargo clippy -p kit-tui -p kit-cli -p kit-agents --all-targets -- -D warnings\n";
prove_prompt += "  cargo fmt --all --check\n";
prove_prompt += "Optional: cargo run -p kit-cli -- doctor --json  (check schemaVersion)\n";
prove_prompt += "Set tests_ok/clippy_ok from actual exit codes. notes = short summary of failures if any.\n";
prove_prompt += "Do not claim green without running commands.\n";

let prove = agent(prove_prompt, #{
    label: "prove",
    capability_mode: "all",
    output_schema: prove_schema,
});

let tests_ok = false;
let clippy_ok = false;
let prove_notes = "prove agent failed";
if prove != () && prove.success && prove.output != () {
    tests_ok = prove.output.tests_ok;
    clippy_ok = prove.output.clippy_ok;
    prove_notes = prove.output.notes;
}
log("Prove tests_ok=" + tests_ok.to_string() + " clippy_ok=" + clippy_ok.to_string());

// ============================================================================
// Phase 7 — Report
// ============================================================================
phase("Report");
let report = "";
report += "# Kit quality cycle report\n\n";
report += "- mode: " + mode + "\n- focus: " + focus + "\n";
report += "- merge_readiness: " + merge_readiness + "\n";
report += "- confirmed findings: " + confirmed.len().to_string() + "\n";
report += "- plan slices: " + slices.len().to_string() + "\n";
report += "- implemented attempts: " + implemented.len().to_string() + "\n";
report += "- tests_ok: " + tests_ok.to_string() + " clippy_ok: " + clippy_ok.to_string() + "\n\n";

report += "## Truth\n" + branch_reality + "\n\n";

report += "## Confirmed findings\n";
for c in confirmed {
    report += "### " + c.title + "\n";
    report += "- shard: " + c.shard + "\n- path: " + c.path + "\n";
    report += "- evidence: " + c.evidence + "\n";
    if c.fix != () && c.fix != "" { report += "- fix: " + c.fix + "\n"; }
    report += "\n";
}

report += "## Plan summary\n" + plan_summary + "\n\n";
report += "### Ordered slices\n";
for s in slices {
    report += "1. **" + s.id + "** " + s.title + " (" + s.owner + ")\n";
    report += "   - " + s.acceptance + "\n";
    report += "   - verify: " + s.verify + "\n";
}

report += "\n## Implementation results\n";
for im in implemented {
    report += "- " + im.slice_id + ": done=" + im.done.to_string() + " — " + im.summary + "\n";
    if im.follow_up != () && im.follow_up != "" {
        report += "  - follow_up: " + im.follow_up + "\n";
    }
}

report += "\n## Prove\n" + prove_notes + "\n";

report += "\n## Kill criteria (from plan)\n";
for k in kill_criteria { report += "- " + k + "\n"; }

report += "\n## Next cycle brief (for CEO/Power)\n";
report += "1. If tests/clippy red — fix before more features.\n";
report += "2. Land PR #11 only when 3-OS CI green (Power does not merge).\n";
report += "3. P3 concurrency harness if not done.\n";
report += "4. Re-run: /workflow kit-quality-cycle with args {\"mode\":\"full\",\"focus\":\"all\"}\n";
report += "5. Audit-only cheap cycle: {\"mode\":\"audit\"}\n";

let path = write_scratch_file("kit-quality-report.md", report);

// Also try to write a durable wiki-style log path via an agent if implement ran — skip; scratch is enough.

complete(#{
    path: path,
    mode: mode,
    focus: focus,
    merge_readiness: merge_readiness,
    confirmed_findings: confirmed.len(),
    slices_planned: slices.len(),
    slices_attempted: implemented.len(),
    tests_ok: tests_ok,
    clippy_ok: clippy_ok,
    summary: plan_summary,
});
