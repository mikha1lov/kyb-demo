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
backend/    Rust + Axum + sqlx. Analyze and compare endpoints; extraction behind a trait; comparison is pure and tested; migrations in backend/migrations
fixtures/   Synthetic documents (fictional people, addresses, numbers) and the script that makes them
frontend/   TanStack Start, TanStack Form, Zod, PandaCSS, Ark UI. Three-step onboarding with the document cross-check
```

## Run the backend

```bash
cd backend
cp .env.example .env        # then put OPENAI_API_KEY in .env (the file is git-ignored)
cargo test                  # 14 tests, no network needed
cargo run                   # http://127.0.0.1:8787
```

Settings (environment or `backend/.env`): `DATABASE_URL`, `OPENAI_API_KEY`, `OPENAI_MODEL` (default `gpt-5.4-mini`), `BIND_ADDR`. For local development without Docker for the backend, start only the database with `docker compose up -d db` and use `postgres://kyb:<password>@127.0.0.1:5433/kyb`.

## Run the frontend

```bash
cd frontend
npm install
npm run dev                 # http://localhost:3000, or the next free port; /api is proxied to the backend
npm run typecheck
npm run build
```

The proxy target is `http://127.0.0.1:8787`; set `API_URL` to change it. The draft is kept in localStorage.

## Try it

```bash
python3 fixtures/generate.py   # needs Google Chrome; writes fixtures/out/*.pdf

curl -s http://127.0.0.1:8787/api/documents/analyze \
  -F "file=@fixtures/out/registration_certificate.pdf;type=application/pdf" \
  -F "requirement=formation" \
  -F 'typed={"legalName":"Aliyar Tokoev PE","registrationNumber":"20000000000001"}'
```

Requirements: `formation`, `source_of_funds` (max 180 days old), `proof_of_address` (max 90 days). Repeat `-F requirement=...` to test one upload against several; the response has one result per requirement.

`POST /api/documents/compare` takes JSON `{requirements, typed, extracted}` and returns the same per-requirement results without calling the model.

## Data and privacy

- Use only the synthetic documents. Real passports, bank statements and tax numbers must not be sent to a third-party API.
- The OpenAI call sets `store: false`. Logs contain sizes and outcomes, never document contents or extracted values.
- The extractor is a trait (`DocumentExtractor`); a self-hosted model can replace OpenAI without touching the comparison, the API or the UI.

## Design notes

- Comparison is deliberately separate from extraction: names are normalised (case, word order, legal-form words, Cyrillic to Latin, one-letter transliteration variants), and a missing patronymic is reported as `partial`, not as a mismatch.
- Every extracted value carries a confidence and a page, so the UI can show where a value came from and ask a human about doubtful ones.
- Postgres is connected and migrated on start, and `/api/health` reports the database. Applications are not persisted through the API yet; see `docs/plan.md`.
