# Governance

Kit is an MIT-licensed open-source project made by Westside Web Studio. This file says how decisions are
made, who merges, and what happens if the maintainer is unavailable.

## Roles

- **Owner:** Mazen Zwin ([@Zwin-ux](https://github.com/Zwin-ux)), Westside Web Studio. Sets the product
  direction (`docs/PRD.md`), cuts releases, and has the final say when maintainers disagree.
- **Maintainers:** people with merge rights. Today that is the owner. A maintainer reviews and merges PRs
  that meet [`docs/QUALITY.md`](docs/QUALITY.md) and are green on all three operating systems.
- **Contributors:** anyone who opens an issue or a PR.

## Becoming a maintainer

A contributor with a record of good merged PRs over at least three months can be invited by the owner.
Maintainers agree to the review bar in `docs/QUALITY.md` and to never merge their own PR that changes a
public contract (`--json`, `KIT.toml`, `kit.toml`, `kit.lock`, receipts) without a second maintainer's review.

## Decisions

- Small changes: a PR and a maintainer's review.
- Product direction, new public contracts, or anything in `docs/PRD.md` §12 (non-goals): an issue or a PR
  to the PRD first, decided by the owner, recorded in the PRD's decision table.
- Releases: only the owner pushes release tags.

## Licence and contributions

MIT, inbound = outbound. No CLA. Contributions carry a DCO `Signed-off-by` line (see `CONTRIBUTING.md`).
The licence will not change for code already released.

## Continuity

If the owner is unreachable for 90 days (no activity on GitHub and no reply to an issue labelled
`governance`), the maintainers may appoint a new owner by majority. If there are no other maintainers,
anyone may fork under the MIT licence, and this file asks that the fork be named in the README of this repo
once the owner returns or the repo is archived.
