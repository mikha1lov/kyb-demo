# kyb-proto

Prototype of the "early document cross-check" idea for a KYB onboarding form.
Context and reasoning live in `docs/` (walkthrough log, phase 1, three ideas).

What it shows: a user uploads a registration document or a bank statement, and within seconds sees
what the document says next to what they typed, plus whether the document is recent enough for the
requirement. The mismatch that cost a full rejection after 18 screens in the real flow is caught at upload.

## Run with Docker

```bash
cp .env.example .env        # set POSTGRES_PASSWORD and OPENAI_API_KEY (the file is git-ignored)
docker compose up -d --build
```

Open http://localhost:8080. Compose runs three services: `web` (nginx serving the built SPA and proxying `/api`), `backend` (Rust) and `db` (Postgres 17, data in the `pgdata` volume). Everything is published on `127.0.0.1` only; the database is on port 5433 (`DB_PORT`) and the web app on 8080 (`WEB_PORT`).

```bash
docker compose logs -f backend
docker compose down         # keep the data
docker compose down -v      # drop the database volume too
```

## Layout

```
backend/    Rust + Axum + sqlx. Applications API, document analysis behind a trait, pure comparison logic, migrations in backend/migrations
fixtures/   Synthetic documents (fictional people, addresses, numbers) and the script that makes them
frontend/   TanStack Start, TanStack Form, Zod, PandaCSS, Ark UI. Onboarding in four steps (company, documents, details, review) where the details are prefilled from the documents; state lives on the server
```

## Run the backend

```bash
cd backend
cp .env.example .env        # then put OPENAI_API_KEY in .env (the file is git-ignored)
cargo test                  # unit tests plus API tests that need Postgres
cargo run                   # http://127.0.0.1:8787
```

The API tests use a stub extractor and a real Postgres: start it with `docker compose up -d db` and export `DATABASE_URL` (see below) before `cargo test`. Each test gets its own throwaway database.

Settings (environment or `backend/.env`): `DATABASE_URL`, `OPENAI_API_KEY`, `OPENAI_MODEL` (default `gpt-5.4-mini`), `BIND_ADDR`. For local development without Docker for the backend, start only the database with `docker compose up -d db` and use `postgres://kyb:<password>@127.0.0.1:5433/kyb`.

## Run the frontend

```bash
cd frontend
npm install
npm run dev                 # http://localhost:3000, or the next free port; /api is proxied to the backend
npm run typecheck
npm run build
```

The proxy target is `http://127.0.0.1:8787`; set `API_URL` to change it. The server holds the application; the browser only remembers the id of the current one, and an application can be resumed from its URL (`/onboarding/<id>/...`).

## Try it

```bash
python3 fixtures/generate.py   # needs Google Chrome; writes fixtures/out/*.pdf

curl -s http://127.0.0.1:8787/api/documents/analyze \
  -F "file=@fixtures/out/registration_certificate.pdf;type=application/pdf" \
  -F "requirement=formation" \
  -F 'typed={"legalName":"Aliyar Tokoev PE","registrationNumber":"20000000000001"}'
```

Test documents in `fixtures/out/`, with the company they describe:

| File | Company | What it is | Expected result |
|---|---|---|---|
| `registration_certificate.pdf` | Kyrgyz sole proprietor | registration certificate | covers the registration requirement |
| `bank_statement_recent.pdf` | Kyrgyz sole proprietor | statement, last 3 months | covers source of funds and proof of address |
| `bank_statement_stale.pdf` | Kyrgyz sole proprietor | statement, over a year old | flagged as too old for both requirements |
| `us_certificate_of_formation.pdf` | Texas LLC | certificate of formation | covers the registration requirement; carries a state file number, not the EIN |
| `us_ein_letter.pdf` | Texas LLC | EIN assignment letter | recognised as a tax letter, which also covers the registration requirement; carries the EIN |
| `us_bank_statement_recent.pdf` | Texas LLC | statement, last 3 months, name written "Acme Test Holdings, LLC" | covers source of funds and proof of address; the name still matches |
| `us_bank_statement_stale.pdf` | Texas LLC | statement, over a year old | flagged as too old |
| `us_utility_bill.pdf` | Texas LLC | electricity bill | covers proof of address only |

A document can carry two identifiers: a registry number (company or file number) and a tax number (EIN, TIN). The number typed in the details is compared only with the identifier of the same type, so a certificate with a file number and a letter with an EIN do not contradict each other; with several documents the suggestion prefers the tax number.

The recent statement and the bill carry dates relative to the day the script ran; run `python3 fixtures/generate.py` again when they get old.

Requirements: `formation`, `source_of_funds` (max 180 days old), `proof_of_address` (max 90 days). Repeat `-F requirement=...` to test one upload against several; the response has one result per requirement.

`POST /api/documents/compare` takes JSON `{requirements, typed, extracted}` and returns the same per-requirement results without calling the model.

## Applications API

| Method and path | What it does |
|---|---|
| `POST /api/applications` | create a draft from the country and the entity type; name, number and address may be left out |
| `GET /api/applications/{id}` | the application with its documents, comparison results, requirement checklist and `suggestions` (name, number, address taken from the documents) |
| `PUT /api/applications/{id}/business` | update the business details and the value sources |
| `POST /api/applications/{id}/documents` | upload a document (multipart `file`); extracted fields are stored, the file is not |
| `DELETE /api/applications/{id}/documents/{document}` | remove a document |
| `POST /api/applications/{id}/submit` | submit when every document requirement is met and the details are filled in; the application is then locked |

Suggestions come from registration certificates first, then bank statements, then utility bills, and only for values read with confidence of 0.7 or more. Errors are JSON `{error, message}`. Submitted applications answer `409 already_submitted` to any change. There is no authentication yet: the application id is an unguessable UUID and acts as the access key.

## Data and privacy

- Use only the synthetic documents. Real passports, bank statements and tax numbers must not be sent to a third-party API.
- The OpenAI call sets `store: false`. Logs contain sizes and outcomes, never document contents or extracted values.
- The extractor is a trait (`DocumentExtractor`); a self-hosted model can replace OpenAI without touching the comparison, the API or the UI.

## Design notes

- Comparison is deliberately separate from extraction: names are normalised (case, word order, legal-form words, Cyrillic to Latin, one-letter transliteration variants), and a missing patronymic is reported as `partial`, not as a mismatch.
- Every extracted value carries a confidence and a page, so the UI can show where a value came from and ask a human about doubtful ones.
- Applications, documents (extracted fields only) and the submitted state live in Postgres. See `docs/plan.md` for what is next.
