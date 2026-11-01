# Security policy

## Reporting a vulnerability

Please report security problems privately through GitHub:
**[Report a vulnerability](https://github.com/Zwin-ux/kit/security/advisories/new)**.
Do not open a public issue. If that form is not available, open an issue titled "Security contact
request" with no details, and the owner will reply with a private channel.

Include what you found, how to reproduce it, the Kit version (`kit --version`) and your OS.

What to expect:

- An acknowledgement within 7 days.
- A fix or a plan within 30 days for confirmed issues.
- Public disclosure after a fix ships, or after 90 days, whichever is first, agreed with you.
- Credit in the advisory and the CHANGELOG, unless you ask not to be named.

## Supported versions

Security fixes go into the latest release line (currently 2.0.x). Older lines are not patched.

## Scope

In scope: the `kit` binary and its install scripts, the npm launcher, the starter kits in
`crates/kit-cli/kits`, and anything Kit writes to your machine (skills, rules, MCP config, hooks, receipts).

Especially interesting: a way to make Kit write outside what its plan showed; a repo or kit that makes Kit
run code without the user's yes; a gate or scope check that passes a change it should fail; anything that
leaks a key or token into a receipt, log or agent environment.

Out of scope: vulnerabilities in Claude Code, Codex, Grok or other third-party tools Kit launches (report
those to their vendors), and third-party skills or MCP servers a kit pins (report upstream; tell us too
so we can unpin).
