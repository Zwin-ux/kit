# Kit flagship example: Workshop Desk

Date: 2026-10-02. Status: proposed product and demonstration specification; implementation has not started. Working audience assumption: builders and small business owners evaluating whether Kit helps them ship a useful website. Working title only; no domain or market-position claim.

## The recommendation

Build a small, real website for free workshops: publish an event, reserve a seat, see the reservation later, and cancel it. The host sees the corresponding attendee roster and accurate capacity. The memorable proof is two attendees competing for the final seat: exactly one succeeds, the other sees Sold out, and cancellation lets the other attendee reserve.

The choice follows independent product, acceptance and presentation reviews. A freelancer intake portal risks stopping at a form and admin list. A feedback/approval portal introduces uploads and permissions before delivering its main value. Workshop booking has a recognizable public website, two useful private workflows and an easily understood correctness test.

This is a proposed user-need hypothesis, not validated market demand. Existing event products demonstrate that registration, capacity and guest management are recognizable workflows: [Luma guest management](https://help.luma.com/p/managing-your-guest-list) and [Luma capacity behavior](https://help.luma.com/p/waitlist). We are designing a narrower complete example, not implementing either service or connecting to it.

## Two products, two outcomes

**Workshop attendee:** “I want to know whether this workshop is for me, reserve a place, and find or cancel my reservation later.”

**Workshop organizer:** “I want to share a clear page and know who is actually coming without overselling the room.”

**Kit user:** “I want to turn my idea into a working site with specialist help, understand the changes, and review the exact assembled result before accepting it.”

Kit's example must demonstrate useful specialist output, successful integration and observable behavior. The website is the delivered product; Kit's native task/results/review experience is how we build and inspect it. Public copy can say: “Build a useful website with four specialists. Inspect their changes. Review the exact result.” Do not claim automatic dispatch, merging, test gating, approval or security certification.

## The bounded first product

Build in a separate disposable Git repository, preserving Kit and existing business websites. Use fictional event content and test accounts. Local production-mode browser acceptance comes first. A public deployment is a separate artifact with its own URL, persistence, HTTPS and live checks; localhost evidence never counts as deployment.

1. Public workshop index with three seeded examples and direct workshop URLs.
2. Workshop detail: title, short promise, audience, agenda, venue text, date/time with explicit time zone, capacity and server-derived remaining seats.
3. Real email/password attendee registration, sign-in and sign-out. Preserve the selected workshop through sign-in using a same-origin return path. The attendee explicitly presses Reserve after returning; signing in starts no booking.
4. One authenticated attendee reserves one seat; the server returns a persisted confirmation reference. My reservations shows the same record after sign-out/sign-in.
5. The attendee cancels their own reservation; the seat becomes available again. They can reserve again if capacity and registration state allow it.
6. A provisioned organizer creates a workshop, sees its roster, and opens/closes registration. Dates must be in the future at creation; bookings close at the start time. Closing registration preserves existing reservations. Attendees can cancel before the start even when registration is closed.
7. Only the owning organizer sees names/emails or manages that workshop. Public pages expose no roster or attendee identities.

Accounts created through public signup are attendees. Organizer status is provisioned server-side for the private demo; no role picker, client-selected organizer flag or seed password ships to a public deployment. Email verification, password recovery and notifications are outside this private example: show in-product confirmations, make no claim that an email was sent, and label this as a private demo until account recovery is provided for a public service.

Do not add payments, waitlists, multiple tickets per booking, recurring events, calendar sync, maps, uploads, social login, AI chat, CRM or analytics dashboards. Add those only after observing an unmet need in the completed core loop.

## Website experience

The public page leads with the actual event and a clear Reserve a seat action. Use readable typography, a restrained accent and a compact event list. Display date, time zone, location and availability before the action. Do not invent testimonials, attendee counts or sponsors. Content and photos are fictional, original or licensed, with attribution when needed.

Required pages: `/`, `/workshops/[slug]`, `/sign-in`, `/sign-up`, `/my-reservations`, `/organizer`, `/organizer/workshops/new`, `/organizer/workshops/[id]`. Direct navigation and refresh must work. A nonexistent workshop returns a useful 404.

Required states: loading, empty reservations, field errors, expired session, request failure with retry, confirmed, cancelled, sold out, registration closed and event already started. A stale availability display never guarantees a seat; the server decides and the UI explains a sold-out response. Fetch fresh availability on navigation, successful mutations and explicit refresh. No live-updating/WebSocket claim.

Double-clicking Reserve must not duplicate a booking. A timeout can leave an uncertain outcome: show “We could not confirm yet. Check My reservations before trying again.” A successful cancellation names the workshop and updates its registration/availability state. Say the seat is available to reserve only while registration is open and before the start; otherwise confirm cancellation without inviting a new booking. A repeated cancellation remains harmless.

Use semantic forms, visible labels/focus, keyboard-operable actions and announced status/error messages. Validate actual 390px and 1440px views, 200% zoom and a keyboard-only booking/cancellation journey. Motion remains optional and respects reduced motion.

## Proposed implementation boundary

One Next.js/React/TypeScript application on the Node runtime; same-origin route handlers; SQLite on a persistent local file; Better Auth email/password sessions with its SQLite adapter. Use `better-sqlite3` rather than introducing an ORM. Choose and lock supported versions at build preparation, checking official docs and local compatibility; do not install unpinned `latest` independently in specialist worktrees.

This uses supported building blocks: [Next.js self-hosting](https://nextjs.org/docs/app/guides/self-hosting), [Better Auth installation and SQLite/Next integration](https://better-auth.com/docs/installation), and [Better Auth database/custom-field rules](https://better-auth.com/docs/concepts/database). Organizer role must not be writable through signup/update input. A maintained authentication library does not replace application ownership checks.

Use one application instance with a persistent SQLite volume. Do not promise ephemeral/serverless or multi-instance deployment with this storage design. Generate an ignored development authentication secret locally; never commit, print or record it. No external service account or paid API is needed for the private example.

Use the auth library's complete generated core schema (user, session, account and verification), including its credential handling; do not substitute handmade password/session tables. Add only:

- `workshops`: id, unique slug, owner user id, title, summary, audience, agenda, venue, starts-at UTC, display time zone, positive integer capacity, registration-open flag, creation time.
- `reservations`: id, workshop id, attendee user id, active/cancelled status, creation/update times. Unique workshop/user pair. Rebooking changes that same row back to active; historical audit history is outside v1.

Seat availability is capacity minus active reservations. Reservation admission and reactivation use a database transaction that locks before reading capacity. The active count must never exceed capacity. Cancellation and its count change are atomic. Validate this with actual HTTP requests and a separate two-connection database concurrency check, not a mocked repository. The contention check uses independent worker processes/connections to the same dedicated test database; do not block one synchronous event loop waiting for its own lock holder.

## API contract to freeze before parallel implementation

Auth remains under `/api/auth/*`. Every custom protected route obtains identity from the validated server session, never a supplied user id. Custom mutations check the expected request origin and allowed fields. Sensitive responses are private/non-cacheable; public responses contain no personal records.

| Route | Behavior |
| --- | --- |
| `GET /api/workshops` | Public future workshop summaries and availability; registration state explicit |
| `GET /api/workshops/:slug` | Public detail; unknown slug 404; no roster |
| `POST /api/workshops/:id/reservations` | Signed-in attendee only; create/reactivate 201, already-active 200 with same reference even when closed/started; new/reactivated requests get full 409 `SOLD_OUT` or closed/started 409 `REGISTRATION_CLOSED` |
| `GET /api/me/reservations` | Only the session attendee's active and cancelled reservations |
| `POST /api/reservations/:id/cancel` | Own reservation only; already-cancelled returns 200 even after start; another user's or unknown reservation 404; cancelling an active reservation after start returns 409 `EVENT_STARTED` |
| `GET /api/organizer/workshops` | Organizer's own workshops only |
| `POST /api/organizer/workshops` | Organizer creates a validated future workshop owned by the session user |
| `GET /api/organizer/workshops/:id/reservations` | Owning organizer's roster, including status |
| `PATCH /api/organizer/workshops/:id/registration` | Owning organizer changes only open/closed state |

Identity/role/ownership checks run before idempotent no-op handling, so an existing status never grants another user access. Protected anonymous requests return 401. An attendee attempting organizer routes gets 403; an organizer accessing someone else's private workshop gets 404. Invalid input returns 400 with useful field errors. Domain errors use `{error:{code,message}}`. Success returns the documented resource. Freeze exact JSON shapes in `docs/contract.md` before Frontend/Backend begin; this table is the required behavior, not permission to invent incompatible shapes independently. OpenAPI generation is unnecessary for this small contract.

## Acceptance: the site actually works

Use O1/O2 organizer accounts and A1/A2 attendees, independently authenticated. Seed through the actual authentication implementation. Browser contexts share no cookies. Developer seeds use only fictional `.invalid` addresses and must fail outside the approved local demo configuration. Explicit local-demo mode is separate from Node production build mode: it permits seeding a fresh, designated demo database, then running the production build locally against it. It must refuse public/unapproved deployments and non-demo database paths.

| ID | Test | Required proof |
| --- | --- | --- |
| W1 | O1 creates a future, one-seat workshop; new anonymous context opens its exact URL | UI, HTTP response and persisted workshop agree |
| W2 | New attendee signs up, returns to selected workshop and explicitly reserves | Real session and persisted confirmation; original workshop retained |
| W3 | Booking appears in My reservations and O1's roster | Matching reference, attendee and active state |
| W4 | Reload, new browser login and actual server stop/start | Same database and reservation survive; no browser-storage substitute |
| W5 | A1/A2 submit independent concurrent requests for a fresh final seat | Exactly one 201, one 409 `SOLD_OUT`, one active row; no 500/oversell |
| W6 | W5 winner retries; separately send two concurrent first-booking requests for the same attendee to a fresh two-seat workshop | Retry returns same reference; concurrent creation yields one 201/one 200 with the same reference, one active row and one remaining seat |
| W7 | Winner cancels; repeated cancellation; loser reserves | One seat released only once; persisted cancelled/active states agree |
| W8 | Use a separate A1-owned reservation fixture: anonymous mutation, A2 cancelling A1, attendee organizer request, O2 private roster access | Agreed 401/404/403/404; no record mutation or personal-data leakage |
| W9 | Submitted identity/role fields and off-origin mutations | Cannot become organizer, impersonate another user or mutate data cross-origin |
| W10 | O1 closes/reopens registration; event reaches start time; retry active reservation/already-cancelled record | New/reactivated booking rejected while closed/started; active cancellation after start rejected; idempotent existing-status requests stay 200 with no mutation; records retained |
| W11 | Real invalid input, failed request, logout and direct URLs | Useful errors; no false success; logged-out protected actions denied; no broken redirects/404s |
| W12 | Mobile/desktop, 200% zoom, keyboard-only journey | All primary actions reachable; visible focus/status; no horizontal form clipping |

For W10, the owner seeds a dedicated event starting 30 seconds in the future, makes requests before/after that time with a bounded 60-second check, and preserves the responses. Do not alter the machine clock or bypass production time checks. A test-only clock can support unit tests but cannot replace this real-server boundary proof.

Run production build/start and browser journeys against the real server/database. Unit tests may support development but cannot close W1–W12. Record the actual requests and database invariants for concurrency/authorization. Automated integration/browser checks must fail on a fake success response, missing persistence, unauthorized mutation or oversell. No test-count target substitutes for these outcomes.

## Build through the actual Kit roles

The showcase uses the exact retained Kit alpha.4 archive; record its checksum and installed Claude version. Our planning subagents do not count as native Kit specialist execution evidence.

1. Owner creates the separate demo repo, minimal shared shell and locked dependencies. Product is explicitly submitted through native Kit: refine this spec into `docs/product.md`, `docs/contract.md` and `docs/acceptance.md` in its isolated writer worktree. Report exact paths and outstanding decisions. Owner inspects/adopts the contract and records the new shared base revision.
2. Frontend and Backend receive that same frozen revision/contract and work in distinct native worktrees. Ignored dependencies/configuration are not assumed to propagate through Git: the owner provides approved per-worktree setup, local ignored secrets and a separate SQLite path/port for each writer. No writer uses another writer's database or the final acceptance database. The owner provisions a separate persistent database for the integrated candidate. Native concurrent/background execution is requested explicitly through Claude where supported; overlapping native timings prove parallelism. If the host runs them sequentially, record that truthfully; Kit adds no scheduler. Frontend owns public/auth/attendee/organizer presentation; Backend owns session configuration, database, custom API routes and integration checks. Owner owns dependency/lockfile changes, root config and shared types. A contract mismatch is reported before changing it.
3. Each writer reports its original task, worktree path, changed files, actual checks and limitations. Kit/owner records real native attempt/result IDs from runtime evidence; writers must not invent unavailable IDs. Kit captures immutable answers as Needs review. Failed tools/checks are disclosed; no invented passing checks.
4. Owner inspects and integrates selected patches into a provisional review candidate. The owner runs W1–W12 against the assembled app and freezes its exact source revision plus a source file-hash manifest before Security review. Include uncommitted source and config needed to identify the candidate; exclude secrets, runtime database files, node_modules and generated build output. Verify those source hashes again after review. This is an explicit integration action, not automatic Kit merging.
5. Keep the source result's live Kit session open through preparation of Kit's generated fixed-result Security request. A separate process/resume reset cannot recreate that session-local key; do not substitute reconstructed text and call it a demonstrated Kit action. Add an owner-authored candidate manifest mapping the captured Frontend/Backend/Product results to the exact integrated files. Security reads that candidate, contract and tests with Read/Glob/Grep only. It reviews ownership/session boundaries, personal data, atomic capacity, duplicate/cancel behavior and unchecked assumptions. It cannot execute scanners or runtime tests; the owner supplies independently executed evidence.
6. Findings go back as a scoped writer task; changed behavior receives fresh checks and review. Preserve the earlier fixed result. Product performs a final acceptance-gap check against the assembled app. Owner decides acceptance; every native completion can still honestly display Needs review.

Prompt instructions common to all writer tasks: use the native isolated worktree; report and stop if isolation is unavailable; preserve the frozen contract; do not deploy, merge, alter another worktree, expand scope or delegate. Give necessary tool authorization explicitly. Frontend must connect the real endpoints; mocks cannot pass acceptance. Backend must leave runnable checks. Security gets named immutable artifacts and acknowledges unavailable execution evidence.

## Ready-to-use task briefs

The owner adds the exact repo/base revision, allowed paths and commands before submitting each brief. Native Kit preparation appends the chosen specialist mention; inspect the visible prompt and press Enter explicitly. Never replace a captured result's JSON answer when adding integration context.

**Product:** “Refine the Workshop Desk proposal into product, API/data contract and W1–W12 acceptance documents. Keep the scope fixed. Resolve response shapes, idempotence and error precedence. Work in your isolated native worktree, report exact paths and limits, and stop if isolation is unavailable. Make no application changes.”

**Backend:** “Implement the frozen contract's real sessions, private organizer ownership, durable SQLite persistence and atomic single-seat booking/cancellation. Leave runnable auth, duplicate, concurrent-capacity and restart checks. Use only assigned backend/API/test paths; no UI, lockfile, scope changes, deployment or merge. Report actual executed checks and unavailable checks.”

**Frontend:** “Build the public workshop website and attendee/organizer journeys from the frozen contract. Use actual APIs for acceptance. Cover sign-in return, saved reservations, sold-out/closed/error states and keyboard/mobile behavior. Use only assigned presentation/client paths; preserve backend/shared contracts and lockfile. Report exact changed paths, actual checks and limits.”

**Security:** “Review the fixed captured result and owner-supplied integrated candidate manifest against the original contract and W1–W12. Read the exact named candidate files. Assess server-derived identity/roles, private-record access, session/origin boundaries, transactional capacity and idempotent cancellation. Give actionable findings with paths/lines, distinguish independently supplied runtime evidence from your source inspection, and state unavailable checks. Read/Glob/Grep only; no edits, execution, delegation, merge or approval.”

## What to show

Make a 2–3 minute product/story video, backed by a longer uncut acceptance recording and machine-readable evidence. Condense real elapsed build time with honest edits; never label edited footage a single continuous build. Do not manufacture a vulnerability, review finding, captured result or successful run for the video.

| Time | On screen | Message/proof |
| --- | --- | --- |
| 0:00–0:20 | Finished workshop page with one seat left | “A host needs accurate attendance. A guest needs a reservation they can trust.” |
| 0:20–0:55 | Actual native Kit task preparation, manual Enter, four roles and distinct writer worktrees | “I choose the tasks; the specialists build against one contract.” |
| 0:55–1:35 | Two real attendees compete; one succeeds, one sold out; cancel/rebook; matching organizer roster | “The whole booking loop works.” |
| 1:35–2:15 | Saved result id, generated exact-result Security draft, named files and independently run checks | “Completion has inspectable evidence and a review tied to the exact change.” |
| 2:15–2:40 | Working mobile view, restart-persistence proof and final artifact/source link | “This is the product built, with the evidence needed to trust this version.” |

Keep an evidence folder with spec/contract, source revision, Kit version/hash, native attempt/result mapping, worktree paths/diffs, actual check logs, request/database results, browser images/recording and Security findings/limits. Redact passwords, cookies, session tokens and secrets. Use synthetic identities; no business/client records. Public sharing requires review of the concrete recording/artifact.

## Definition of demonstrated success

The product gate passes when W1–W12 pass on the exact assembled artifact and a fresh setup can run the documented flow. The engineering gate passes when all four real native roles have verifiable outputs, writers were isolated, the owner integrated them, and Security reviewed those exact bytes with independent owner checks.

The native Kit experience gate also requires an actual allowed terminal showing visible draft preparation, manual submission, focus/Escape and discoverable results. Previous Terminal computer-use access was denied. Headless traces, companion observers and app-browser images do not close that gate. Keep it explicitly open until legitimate visible evidence exists.

Call the outcome “working product and verified specialist build/review flow” only for the gates actually passed. “Fully demonstrated native Kit” requires all three gates. Local success is a private demo; public website, supported-platform and release-readiness claims each require their own live evidence.

## First decision and next action

This document is the concrete proposed direction for owner review; it does not claim an implemented product or accepted native demonstration. Settle this product direction and frozen core scope before implementation. Then run the native Product specification task, establish the common contract/base revision, and dispatch Frontend/Backend explicitly. Do not build extra platform features to prepare this demonstration.
