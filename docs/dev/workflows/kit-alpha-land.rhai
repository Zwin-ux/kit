let meta = #{
    name: "kit-alpha-land",
    description: "CEO-framed alpha-land swarm: Factory isolation, N1 CI, Session B proof, remaining TUI craft",
    when_to_use: "Land Kit 1.0.0-alpha.1 honestly: parallel Factory/Power/Luna with design|product first, no 1.0.0 tag",
    phases: [
        #{ title: "CEO frame", detail: "lock alpha-land vs 1.0.0" },
        #{ title: "Luna", detail: "parallel diagnose Factory, CI, TUI, dogfood" },
        #{ title: "Skeptic", detail: "fail-closed verify" },
        #{ title: "Plan", detail: "ordered owners" },
        #{ title: "Report", detail: "scratch brief" },
    ],
};

let frame_schema = #{
    "type": "object",
    "required": ["alpha_only", "do_not_merge", "kill_criteria", "next_three"],
    "properties": #{
        "alpha_only": #{ "type": "boolean" },
        "do_not_merge": #{ "type": "boolean" },
        "kill_criteria": #{ "type": "array", "maxItems": 8, "items": #{ "type": "string" } },
        "next_three": #{ "type": "array", "maxItems": 3, "items": #{ "type": "string" } },
    },
};

let shard_schema = #{
    "type": "object",
    "required": ["shard", "ready", "findings", "next_actions"],
    "properties": #{
        "shard": #{ "type": "string" },
        "ready": #{ "type": "boolean" },
        "findings": #{
            "type": "array", "maxItems": 8,
            "items": #{
                "type": "object",
                "required": ["title", "severity", "detail"],
                "properties": #{
                    "title": #{ "type": "string" },
                    "severity": #{ "type": "string" },
                    "detail": #{ "type": "string" },
                    "path": #{ "type": "string" },
                },
            },
        },
        "next_actions": #{ "type": "array", "maxItems": 6, "items": #{ "type": "string" } },
    },
};

let skeptic_schema = #{
    "type": "object",
    "required": ["claim", "real", "evidence"],
    "properties": #{
        "claim": #{ "type": "string" },
        "real": #{ "type": "boolean" },
        "evidence": #{ "type": "string" },
    },
};

let plan_schema = #{
    "type": "object",
    "required": ["summary", "ordered_slices"],
    "properties": #{
        "summary": #{ "type": "string" },
        "ordered_slices": #{
            "type": "array", "maxItems": 10,
            "items": #{
                "type": "object",
                "required": ["id", "owner", "title", "acceptance"],
                "properties": #{
                    "id": #{ "type": "string" },
                    "owner": #{ "type": "string" },
                    "title": #{ "type": "string" },
                    "acceptance": #{ "type": "string" },
                },
            },
        },
    },
};

phase("CEO frame");
let ceo_p = "";
ceo_p += "You are Kit CEO (read-only). Product: Control Room, gate is the wedge. Alpha land only.\n";
ceo_p += "Read AGENTS.md, docs/dev/SPEC-next.md, tasks/todo.md, wiki/kit-1.0/outputs/queries/2026-09-11-kit-quality-cycle-surface.md.\n";
ceo_p += "Do not merge. Do not tag v1.0.0. Frozen contracts stay Claude-only.\n";
ceo_p += "Set alpha_only=true, do_not_merge=true. next_three are the highest-leverage unblocked slices.\n";
let ceo = agent(ceo_p, #{
    label: "ceo-frame",
    capability_mode: "read-only",
    output_schema: frame_schema,
});
log("CEO frame done");

phase("Luna");
let jobs = [
    #{
        prompt: "Factory Luna, read-only. Inspect crates/kit-agents/src/process.rs isolate_cargo_target_dir, crates/kit-cli/src/engine/runner.rs dry_run_gate_does_not_write_parent_cargo_target_dir, crates/kit-gate/src/lib.rs isolate_cargo_target_dir. Is P1 honestly done? ready=true only with test names you read. shard=p1-target-dir.",
        label: "luna:p1",
        capability_mode: "read-only",
        output_schema: shard_schema,
    },
    #{
        prompt: "Factory Luna, read-only. Read .github/workflows/ci.yml and keep-alive.yml and rust.yml. Does every Rust-only PR still pay Node CI + keep-alive? Propose exact path filters. shard=n1-ci.",
        label: "luna:n1",
        capability_mode: "read-only",
        output_schema: shard_schema,
    },
    #{
        prompt: "Power Luna, read-only. Read crates/kit-tui/src/theme.rs Theme::resolve and ui draw paths. Is ANSI16/NO_COLOR paint proof still missing? Do not invent Theme-on-App if a smaller proof exists. shard=theme.",
        label: "luna:theme",
        capability_mode: "read-only",
        output_schema: shard_schema,
    },
    #{
        prompt: "Dogfood Luna, read-only. Read docs/dev/DOGFOOD.md and docs/dev/dogfood-notes/. What is missing for Session B live kit run --agent ready? shard=session-b.",
        label: "luna:session-b",
        capability_mode: "read-only",
        output_schema: shard_schema,
    },
    #{
        prompt: "Product Luna, read-only. Read README.md first 80 lines and crates/kit-tui snapshots populated_control_room_snapshot. Does first paint still sell the gate? shard=product-paint.",
        label: "luna:product",
        capability_mode: "read-only",
        output_schema: shard_schema,
    },
];
let luna = parallel(jobs);

let findings = [];
let li = 0;
for r in luna {
    if r != () && r.success && r.output != () && r.output.findings != () {
        for f in r.output.findings {
            findings.push(#{
                shard: r.output.shard,
                title: f.title,
                severity: f.severity,
                detail: f.detail,
            });
        }
    }
    li += 1;
}

phase("Skeptic");
let MAX_S = 6;
let n = findings.len();
if n > MAX_S { n = MAX_S; }
let vjobs = [];
let vi = 0;
while vi < n {
    let f = findings[vi];
    let vp = "";
    vp += "Adversarially verify. Default real=false. Use read_file/grep.\n";
    vp += "Claim: " + json_encode(f.title) + "\nDetail: " + json_encode(f.detail) + "\n";
    vjobs.push(#{
        prompt: vp,
        label: "skeptic:" + vi.to_string(),
        capability_mode: "read-only",
        output_schema: skeptic_schema,
    });
    vi += 1;
}
let confirmed = [];
if vjobs.len() > 0 {
    let vs = parallel(vjobs);
    let vj = 0;
    for v in vs {
        if v != () && v.success && v.output != () && v.output.real == true && v.output.evidence != () {
            confirmed.push(findings[vj]);
        }
        vj += 1;
    }
}

phase("Plan");
let plan_p = "";
plan_p += "Power planner. Confirmed: " + json_encode(confirmed) + "\n";
plan_p += "Owners: Factory=kit-cli/CI, Power=kit-tui, You=GitHub About/merge. No v1.0.0. No mascot.\n";
plan_p += "Read tasks/todo.md so you do not redo shipped TUI slices.\n";
let plan = agent(plan_p, #{
    label: "alpha-plan",
    capability_mode: "read-only",
    output_schema: plan_schema,
});

phase("Report");
let summary = "alpha-land";
if plan != () && plan.success && plan.output != () { summary = plan.output.summary; }
let report = "# Kit alpha-land swarm\n\n" + summary + "\n\nConfirmed: " + confirmed.len().to_string() + "\n";
let path = write_scratch_file("kit-alpha-land.md", report);
complete(#{ path: path, summary: summary, confirmed: confirmed.len() });
