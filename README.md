# kyb-proto

Prototype of a KYB onboarding flow that reads documents at upload time. You drop in a registration certificate or a bank statement, it pulls out the name, number and address, checks them against what you typed and prefills the details form. A name that doesn't match the certificate shows up right on the upload screen instead of after a rejection a few steps later.

Rust (Axum, sqlx) and Postgres on the back, TanStack Start with PandaCSS on the front, OpenAI for reading the documents.

## Quick start

You need Docker and an OpenAI key.

```bash
cp .env.example .env    # set POSTGRES_PASSWORD and OPENAI_API_KEY
docker compose up -d --build
```

The app is on http://localhost:8080. Everything binds to 127.0.0.1: web on 8080 (`WEB_PORT`), Postgres on 5433 (`DB_PORT`).

```bash
docker compose logs -f backend
docker compose down       # keeps the data
docker compose down -v    # drops the database too
```

## Working on it without Docker

Start only the database, then run the backend and the frontend on the host.

```bash
docker compose up -d db

cd backend
cp .env.example .env      # OPENAI_API_KEY, DATABASE_URL=postgres://kyb:<password>@127.0.0.1:5433/kyb
cargo run                 # 127.0.0.1:8787

cd frontend
npm install
npm run dev               # http://localhost:3000, Vite takes the next port if it's busy
```

Vite proxies `/api` to `127.0.0.1:8787`, set `API_URL` to change that.

Tests:

```bash
cd backend && cargo test   # needs DATABASE_URL pointing at the compose database
cd frontend && npm run typecheck
```

The API tests run against real Postgres with a stub instead of OpenAI, and every test gets its own temporary database. Everything else in `cargo test` is plain unit tests.

## How it works

The flow is company (country and entity type), documents, details, review. You upload whatever files you have. Each one is classified and checked against three requirements: registration document, source of funds, proof of address. One bank statement can cover the last two. When the requirements are met, the details step opens already filled in from the documents, with a "From document" tag on every value that still matches. After submit the application is locked.

The application lives in Postgres. The browser only remembers the id of the current one, so `/onboarding/<id>/...` picks it up in any browser.

What the checks do:

- Names are compared loosely: case, word order, PE/LLC style suffixes, Cyrillic to Latin and small transliteration differences don't count. A missing patronymic is reported as a partial match.
- A document can carry a registry number (company or file number) and a tax number (EIN, TIN). The number you typed is compared with the identifier of the same type only, so a certificate with a file number and a letter with an EIN don't contradict each other.
- Statements must be at most 180 days old for source of funds and 90 days for proof of address.
- Prefill takes values from the registration certificate first, then the tax letter, the bank statement and the utility bill, and ignores anything read with confidence below 0.7. For the number it prefers the tax number.

Reading the documents goes through the `DocumentExtractor` trait. The only implementation calls OpenAI (`gpt-5.4-mini`, change it with `OPENAI_MODEL`) with `store: false`, so a local model can replace it later without touching the rest.

## API

```
GET    /api/health
POST   /api/applications                                   country and entity type, the rest is optional
GET    /api/applications/{id}                              documents, results, checklist, suggestions
PUT    /api/applications/{id}/business                     details and where each value came from
POST   /api/applications/{id}/documents                    multipart "file"
DELETE /api/applications/{id}/documents/{document}
POST   /api/applications/{id}/submit
```

Errors are `{error, message}`. A submitted application answers 409 `already_submitted` to any change, and submit returns 422 `incomplete` until everything is met. Only the extracted fields are stored, never the uploaded file.

There is no auth. The application id is a random UUID and works as the key, which is fine locally and not much else.

`POST /api/documents/analyze` and `/api/documents/compare` are stateless versions of the same thing, handy for poking with curl:

```bash
curl -s http://127.0.0.1:8787/api/documents/analyze \
  -F "file=@fixtures/out/registration_certificate.pdf;type=application/pdf" \
  -F "requirement=formation" \
  -F 'typed={"legalName":"Aliyar Tokoev PE","registrationNumber":"20000000000001"}'
```

Repeat `-F requirement=...` to check one file against several requirements.

## Test documents

`python3 fixtures/generate.py` writes test documents to `fixtures/out/` (needs Chrome). Names, numbers and addresses are made up and every file says so.

- `registration_certificate.pdf`, `bank_statement_recent.pdf`, `bank_statement_stale.pdf`: a Kyrgyz sole proprietor.
- `us_certificate_of_formation.pdf`, `us_ein_letter.pdf`, `us_bank_statement_recent.pdf`, `us_bank_statement_stale.pdf`, `us_utility_bill.pdf`: a Texas LLC. The recent statement writes the name as "Acme Test Holdings, LLC" on purpose.

What to expect: the recent statement covers source of funds and proof of address, the stale one gets flagged as too old for both, the utility bill covers proof of address only, the EIN letter counts as a registration document. The certificate carries a state file number and the letter and statement carry the EIN, which is why numbers are compared by type.

The dates are relative to the day you run the script, so regenerate them when they get old.
