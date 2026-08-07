---
title: Release (M5)
type: concept
created: 2026-08-01
updated: 2026-08-01
sources: [prd-1.0]
tags: [release]
status: planned
---

# Release (M5)

Kill criterion: **clean-machine install on all three OSes, verified from the README, by someone who is not you.**

## Artifacts

| Artifact | Purpose |
|----------|---------|
| `kit` native binary | macOS arm64/x64, Linux glibc arm64/x64, Windows x64 |
| npm `@mzwin/kit` | launcher resolving platform package (trenchwire pattern) |
| `cargo install kit-cli` | Rust users |
| `curl \| sh` installer | checksum-verified |
| GitHub Release | binaries + SBOM optional |
| Demo GIF/video | 60s gate catch |

## Versioning

- `1.0.0-alpha.*` — current  
- `1.0.0-rc.1` — feature complete, install path works  
- `1.0.0` — DoD checklist complete  

Hard boundary: 0.1.x was Node; 1.0 is binary. Same command name `kit`.

## README structure (ship day)

1. One-liner + install  
2. 30-second quickstart (`kit doctor`, `kit run`, `kit`)  
3. Key map  
4. kit.toml gate example  
5. Receipts location  
6. Legacy 0.1 note (if still published)  

## Launch checklist

- [ ] Third-party install test (friend / CI clean VM)  
- [ ] All PRD DoD boxes checked in [[concepts/product/definition-of-done]]  
- [ ] Security pass: no secrets in logs/receipts  
- [ ] CHANGELOG 1.0.0 written in STE  
- [ ] Demo recorded and linked  
- [ ] Issues labeled `1.0.1` for known non-blockers (attach polish, mascot)  

## Rollback

npm dist-tags: keep `latest` on last good; `next` for rc.  
Binary releases immutable; fix-forward with 1.0.1.
