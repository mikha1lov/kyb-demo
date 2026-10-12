# Three ideas to optimise the KYB form, step by step

Based on `kyb-walkthrough.md` (S1-S20) and `phase1-dedup-and-data-model.md`. These are hypotheses from outside: what a user sees, not how Altitude is built.

---

## 1. Early cross-check between the uploaded document and what was typed

**Problem (real case).** The company name was typed on step 2 and rejected only after about 18 screens, as a generic "document not accepted".

**What the user sees**
1. On "Who are you" they enter country, entity type, name and registration number.
2. The next step asks for the registration document (and a bank statement).
3. Within a few seconds a card appears: "We read in the document: name `...`, address `...`, period `...`" next to "You entered: `...`". The difference (for example the patronymic) is highlighted.
4. If the values differ, two choices: "Use the name from the document" or "Keep mine and upload another document".
5. If the document cannot be read, the reason is explained (glare, cropped, older than 90 days) with a re-upload button.

**What the system does**
- Classifies the document (registration, bank statement, utility bill), extracts fields with a confidence per field and the page number.
- Compares with the typed values: name, address, number, date range (freshness).
- Stores the value together with its source ("from document, page 1, confidence 0.93").
- Keeps both spellings: as in the document and a normalised one, if a provider needs a different form.

**Metrics**
- Share of applications without a rejection caused by a mismatch.
- Number of mismatches caught at upload.
- Share of users who accept the value from the document.
- Time to the first error.

**Risks**
- False positives (transliteration, word order).
- Forged documents (a text cross-check does not prove authenticity).
- Legal sensitivity of the company name.
- Mitigation: confidence thresholds and a manual path for doubtful cases.

---

## 2. One input per field, one data model

**Problem.** Registration number x3, address x4, activity description x4, client-funds question x3.

**What the user sees**
1. Country, entity type, name and number are entered once, on the first step.
2. Later screens show them read-only with an "edit" link. No repeated fields.
3. Address: a toggle "Operating address is the same as registered" (on by default for sole proprietors).
4. One block "What you do" with the description, from which a NAICS code is suggested; one block "Expected activity" (annual turnover, from which the monthly volume is derived).
5. The client-funds question is asked once. If "none of the above" is chosen in the restricted-activities list, the other formulations are filled in automatically.

**What the system does**
- One company record; every field has an owner screen and a source (typed, from document, derived).
- Builds the request each provider needs from this record, instead of asking the user again.
- Derivation rules: DAO from entity type, signatory from "sole proprietorship", monthly volume from annual.

**Metrics**
- Required inputs per application.
- Share of re-entered fields (target 0).
- Completion time and drop-off per step.

**Risks**
- A provider may require its own format: convert on our side, do not ask again.
- Derived values must be shown (not silently filled) and remain editable.

---

## 3. Field-level errors and a review screen

**Problem.** A generic "Some steps had errors" banner with no field; a "1-10" placeholder that looked selected; an attestation "everything is correct" without a summary.

**What the user sees**
1. **Errors.** A red message next to the field with the reason ("minimum 200 characters, now 94"). The top banner links to the field and focuses it.
2. **Hints in advance.** Limits (200 characters, 90 days, file formats and size) are visible before typing.
3. **Placeholders.** Grey "Select..." instead of a value-like "1-10"; an empty required field looks empty.
4. **Review.** Before the attestation a screen "Check what you entered", grouped (company, address, activity, documents, parties), each block with "edit" and a source marker. Differences from documents are highlighted.
5. **Attestation** is available only on this screen, below the review.

**What the system does**
- Validates on blur, saves a draft on every step, passes the current values to the review.

**Metrics**
- Share of submission errors.
- Time to fix an error.
- Share of applications where "edit" was used on the review.
- Support requests of the type "I do not understand what is wrong".

**Risks**
- A review without highlighted differences becomes a formality.
- Over-eager validation annoys users: validate after leaving a field, not on every keystroke.

---

## How they depend on each other

- Idea 3 is the fastest and the most independent: UI only, no integrations.
- Idea 2 needs the single data model and agreement with the providers.
- Idea 1 depends on idea 2 (something to compare against) and on field extraction from documents.
