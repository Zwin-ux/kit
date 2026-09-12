let meta = #{
    name: "kit-craft-cycle",
    description: "Design + product + Claude-grade eng in parallel, then optional Power TUI/motion slice",
    when_to_use: "Before Kit TUI, motion, or Control Room craft. Always think design | product first.",
    phases: [
        #{ title: "Lenses", detail: "product, design, eng in parallel" },
        #{ title: "Skeptic", detail: "fail-closed verify of top findings" },
        #{ title: "Plan", detail: "ordered craft slices with kill criteria" },
        #{ title: "Implement", detail: "optional Power slice (gated)" },
        #{ title: "Report", detail: "scratch brief for the next cycle" },
    ],
};

let lens_schema = #{
    "type": "object",
    "required": ["lens", "score_0_to_10", "findings", "must_fix", "non_goals"],
    "properties": #{
        "lens": #{ "type": "string" },
        "score_0_to_10": #{ "type": "integer" },
        "findings": #{
            "type": "array", "maxItems": 8,
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
        "non_goals": #{ "type": "array", "maxItems": 6, "items": #{ "type": "string" } },
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
            "type": "array", "maxItems": 8,
            "items": #{
                "type": "object",
                "required": ["id", "title", "owner", "why", "acceptance", "verify"],
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
        "kill_criteria": #{ "type": "array", "maxItems": 6, "items": #{ "type": "string" } },
        "non_goals": #{ "type": "array", "maxItems": 6, "items": #{ "type": "string" } },
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

fn trimmed(s) {
    if type_of(s) == "string" { s.trim(); s } else { "" }
}

let focus = "motion";
let auto_implement = false;
let max_implement = 1;
if args != () {
    if args.focus != () { focus = args.focus; }
    if args.auto_implement == true { auto_implement = true; }
    if args.max_implement != () { max_implement = args.max_implement; }
}

log("kit-craft-cycle focus=" + focus + " auto_implement=" + auto_implement.to_string());

phase("Lenses");

let product_p = "";
product_p += "You are Kit product (read-only). Job: dispatch many agents, watch them, refuse unproven work.\n";
product_p += "Focus lens: " + focus + ".\n";
product_p += "Read with tools: docs/dev/PRD-1.0.md, docs/dev/SPEC-surface-1.0.md, docs/dev/SPEC-next.md, ";
product_p += "docs/dev/CURRENT.md, AGENTS.md, docs/dev/DESIGN-tui.md, tasks/todo.md.\n";
product_p += "Ask: does this surface make the gate the wedge? Would a recruiter understand the 30-second demo?\n";
product_p += "Cut marketplace, mascot-as-product, extra personas, SaaS homepage patterns.\n";
product_p += "Use read_file/grep. Empty findings only after inspecting those files. lens=product.\n";

let design_p = "";
design_p += "You are Kit design (read-only). TUI craft vs docs/dev/DESIGN-tui.md and crates/kit-tui/src/ui.\n";
design_p += "Focus lens: " + focus + ".\n";
design_p += "Pressure-test 80x14 and 60x12. Motion: RUNNING/GATING spinner only, one clock AppEvent::AnimationTick, ";
design_p += "KIT_MOTION=off resting frame, no blink, no mascot GIF, no Nerd Font requirement, never color-alone.\n";
design_p += "Clutter audit: border nesting, duplicate signals, chrome vs data. Footer grammar must include keys.\n";
design_p += "Read control_room.rs, dispatch.rs, theme.rs, app.rs format_state_label, snapshots. lens=design.\n";

let eng_p = "";
eng_p += "You are Kit eng using Claude-grade practice (read-only): Karpathy surgical/simple/tests-as-success; ";
eng_p += "addyosmani TDD + five-axis review; frozen contracts (event.rs, kit-core, kit-gate) are Claude-only.\n";
eng_p += "Focus lens: " + focus + ".\n";
eng_p += "Read crates/kit-tui/src/app.rs (AnimationTick dirty policy), event.rs (READ ONLY), ui/control_room.rs.\n";
eng_p += "Flag: extra timers, dirty-on-tick in idle rooms, motion-off regressions, untested spinner frames, ";
eng_p += "byte-slicing unicode, contract edits by Grok. lens=eng.\n";

let lens_jobs = [
    #{ prompt: product_p, label: "lens:product", capability_mode: "read-only", output_schema: lens_schema },
    #{ prompt: design_p, label: "lens:design", capability_mode: "read-only", output_schema: lens_schema },
    #{ prompt: eng_p, label: "lens:eng", capability_mode: "read-only", output_schema: lens_schema },
];
let lens_results = parallel(lens_jobs);

let all_findings = [];
let i = 0;
let lens_names = ["product", "design", "eng"];
for r in lens_results {
    let name = lens_names[i];
    if r != () && r.success && r.output != () {
        if r.output.findings != () {
            for f in r.output.findings {
                all_findings.push(#{
                    lens: name,
                    title: f.title,
                    severity: f.severity,
                    detail: f.detail,
                    path: f.path,
                });
            }
        }
        if r.output.must_fix != () {
            for m in r.output.must_fix {
                all_findings.push(#{
                    lens: name,
                    title: m,
                    severity: "high",
                    detail: "must_fix from " + name,
                    path: "",
                });
            }
        }
        log(name + " score=" + r.output.score_0_to_10.to_string());
    } else {
        log(name + " lens failed or empty");
    }
    i += 1;
}

phase("Skeptic");
let MAX_SKEPTICS = 6;
let vjobs = [];
let n = all_findings.len();
if n > MAX_SKEPTICS { n = MAX_SKEPTICS; log("truncated skeptics from " + all_findings.len().to_string()); }
let vi = 0;
while vi < n {
    let f = all_findings[vi];
    let vp = "";
    vp += "Adversarially verify this Kit craft claim by reading the shipped code. Default real=false.\n";
    vp += "Claim: " + json_encode(f.title) + "\n";
    vp += "Detail: " + json_encode(f.detail) + "\n";
    vp += "Path hint: " + json_encode(f.path) + "\n";
    vp += "Use read_file/grep. Set real=true only with concrete evidence you inspected.\n";
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
    let verdicts = parallel(vjobs);
    let vj = 0;
    for v in verdicts {
        if v != () && v.success && v.output != () && v.output.real == true {
            let ev = trimmed(v.output.evidence);
            if ev != "" {
                confirmed.push(all_findings[vj]);
            }
        }
        vj += 1;
    }
}
log(confirmed.len().to_string() + " confirmed craft findings");

phase("Plan");
let plan_p = "";
plan_p += "You are Kit Power planner. Turn confirmed findings into ordered slices.\n";
plan_p += "Focus: " + focus + ".\n";
plan_p += "Confirmed JSON: " + json_encode(confirmed) + "\n";
plan_p += "Rules: Grok owns kit-tui only. Do not edit event.rs, kit-core, kit-gate. No mascot. ";
plan_p += "Motion is RUNNING/GATING spinner on the existing clock. Tests + insta snapshots are the verify step.\n";
plan_p += "Owners: Power (tui), Factory (engine/CI), You (GitHub About, merge). Max 8 slices.\n";
plan_p += "Read tasks/todo.md and docs/dev/SPEC-next.md so you do not redo shipped work.\n";

let plan = agent(plan_p, #{
    label: "craft-plan",
    capability_mode: "read-only",
    output_schema: plan_schema,
});

let slices = [];
let plan_summary = "(plan agent failed)";
if plan != () && plan.success && plan.output != () {
    plan_summary = plan.output.summary;
    if plan.output.ordered_slices != () {
        for s in plan.output.ordered_slices { slices.push(s); }
    }
}
log("plan: " + plan_summary);

if auto_implement && slices.len() > 0 {
    phase("Implement");
    let take = slices.len();
    if take > max_implement { take = max_implement; }
    let si = 0;
    while si < take {
        let sl = slices[si];
        let ip = "";
        ip += "You are Kit Power. Implement ONE slice. Karpathy: surgical, no extra features.\n";
        ip += "Do NOT edit crates/kit-tui/src/event.rs, crates/kit-core, crates/kit-gate, crates/kit-cli.\n";
        ip += "Slice id: " + sl.id + " title: " + sl.title + "\n";
        ip += "Why: " + sl.why + "\nAcceptance: " + sl.acceptance + "\nVerify: " + sl.verify + "\n";
        ip += "Cwd is the Kit repo. Use tests first when changing behavior. cargo test -p kit-tui after edits.\n";
        ip += "If the slice is already shipped, set done=true and say so. Do not invent mascot or extra timers.\n";
        let impl_r = agent(ip, #{
            label: "craft-impl:" + sl.id,
            capability_mode: "read-write",
            output_schema: implement_schema,
        });
        if impl_r != () && impl_r.success && impl_r.output != () {
            log("implemented " + sl.id + " done=" + impl_r.output.done.to_string());
        }
        si += 1;
    }
}

phase("Report");
let report = "";
report += "# Kit craft cycle\n\n";
report += "Focus: " + focus + "\n\n";
report += "## Plan\n" + plan_summary + "\n\n";
report += "## Confirmed findings (" + confirmed.len().to_string() + ")\n";
for c in confirmed {
    report += "- [" + c.lens + "] " + c.title + " — " + c.detail + "\n";
}
report += "\n## Slices\n";
for s in slices {
    report += "- " + s.id + " " + s.title + " (" + s.owner + ")\n";
}
let path = write_scratch_file("kit-craft-report.md", report);
complete(#{ path: path, summary: plan_summary, confirmed: confirmed.len() });
