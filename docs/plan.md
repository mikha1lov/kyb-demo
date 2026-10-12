# Plan

Last updated: 2026-10-12.

## Status

| Area | State |
|---|---|
| Walkthrough of the real onboarding flow (S1-S20) | Done, see `kyb-walkthrough.md` |
| Duplication analysis, single data model, 7-step order | Done, see `phase1-dedup-and-data-model.md` |
| Three ideas written out step by step | Done, see `three-ideas-step-by-step.md` |
| Backend: analyze and compare endpoints, extractor behind a trait | Done, 14 unit tests |
| Frontend: business, documents, review, done | Done, verified in a browser on synthetic documents |
| Containers: Dockerfiles, Compose, Postgres, nginx for the SPA | Done |
| Persistence of applications | Schema and connection only; no API yet |
| Case workflow, auth, local models | Not started |

## Architecture today

```
browser -> web (nginx, static SPA) -> /api -> backend (Rust, Axum) -> db (Postgres)
                                                    |
                                                    +-> document extractor (OpenAI now, a local model later)
```

- `web`: TanStack Start in SPA mode, built to static files, served by nginx; `/api` is proxied to the backend.
- `backend`: Axum. `POST /api/documents/analyze` extracts fields and compares them with what the user typed; `POST /api/documents/compare` recomputes the comparison without calling the model; `GET /api/health` checks the database.
- `db`: Postgres 17. Tables `applications` and `documents` exist; documents store extracted fields only, never the file.
- The extractor is a trait, so a self-hosted model can replace OpenAI without touching the API or the UI.

## Milestones

### M1. Persist applications
- `POST /api/applications` creates a draft; `PATCH` updates business data and sources; `POST .../documents` stores extracted fields; `POST .../submit` locks the application; `GET` returns the current state.
- The frontend reads and writes the server state; localStorage stays as a cache so a refresh is instant.
- Done when a draft started in one browser can be resumed in another and a submitted application cannot be edited.

### M2. Prefill from documents
- After an upload, offer to fill empty fields (name, registration number, address) from the document, with the source shown on each value.
- Registration number is asked once; operating address defaults to "same as registered".
- Done when a sole proprietor completes the flow typing the number and the address at most once.

### M3. Errors and review
- Field-level errors, limits shown before typing (200 characters, 90 days), review screen with differences highlighted.
- Partly done: field-level errors on the business step, review with "From document" markers.

### M4. Case workflow
- Port the state machine from `kyb-demo` (statuses, idempotent events, audit log) to Rust, run it on Temporal.
- Reviewer view: approve, reject, request information.
- Done when every status change is an event in an audit log and duplicate events are no-ops.

### M5. Local models
- Implement `DocumentExtractor` with self-hosted OCR and a vision model; benchmark on the synthetic set (field accuracy, character error rate, latency, cost per document).
- Decision table: self-host versus a private API with a data processing agreement.
- Done when a field-accuracy target is met on the synthetic set without sending data to a third party.

### M6. Hardening
- Authentication, rate limits, upload scanning, retention policy, encryption of personal data at rest, structured logs and metrics.
- CI: `cargo test`, `tsc`, frontend build, Docker build.

## Before the call

- One-page summary: the four biggest problems of the flow, the company-name case as the main example, the three ideas, the 7-step flow.
- A two-minute demo: type the name without the patronymic, upload the certificate, see the highlighted difference, apply the document value, upload a stale and a recent statement.
- Questions for the team (see `phase1-dedup-and-data-model.md`, section 5).

## Metrics

- Screens and required fields per application, before and after.
- Re-entry rate: share of fields entered more than once (target 0).
- First-pass acceptance: applications that need no resubmission.
- Share of rejections caused by a mismatch that was detectable at upload.
- Time from first screen to submit; median time from submit to decision.

## Risks and open points

- Personal data: only synthetic documents go to a third-party API; real documents stay out until M5 or a signed agreement.
- Extraction errors: every value carries a confidence and a page; low confidence goes to a human, never silently into the form.
- A text comparison does not prove a document is genuine; authenticity checks stay with the verification provider.
- Which duplicates belong to the provider and which to the product is unknown from the outside; it decides how much of M2 is possible.
- Compose publishes only on `127.0.0.1`; nothing here is ready to be exposed to the internet before M6.
