# Phase 1: remove duplication in KYB onboarding

Source: `kyb-walkthrough.md` (Altitude flow observed for a Kyrgyz sole proprietor, S1-S20). Observations, not insider knowledge: what a user sees, not how Altitude is built.

## 1. What is asked more than once

| Data | Asked at | Times | Why it repeats (guess) | Canonical place |
|---|---|---|---|---|
| Registration / tax number | S3 (registry lookup), S5 ("Tax ID"), S7 ("Registration number") | 3 | Each step comes from a different provider module (lookup, company form, questionnaire) | Company profile, entered once |
| Company name | S3, S5 (prefilled, but re-shaped), S13/S20 (checked against documents) | 3 | Same as above; the shortened value is what got rejected | Company profile, with the document's spelling |
| Country | S2 (privacy regime), S3 (incorporation), S5 (country), S8 (operating address country) | 4 | Different purposes, same answer for most sole proprietors | One country field, confirm elsewhere |
| Address | S5 (registered), S8 (operating), S14 (proof of address doc), S19 (Sumsub step 5) | 4 | Registered vs operating vs document vs personal | One address, "same as registered" toggle, then proven once |
| Activity description | S6 (restricted activities), S8 (NAICS + description), S10 (purpose and intended use), S12/S13 (proof of activity) | 4 | Risk, KYB and card/AML questionnaires merged | One "what do you do" block + one expected-activity block |
| Client funds | S6 ("Hold Client Funds"), S9 (free text), S10 (Yes/No) | 3 | Same | Ask once; derive the rest |
| Turnover / volume | S9 (annual), S10 (monthly) | 2 | Different units | One expected volume, derive the other |
| DAO / entity type | S7 (entity type list includes DAO), S8 ("Is the entity a DAO?") | 2 | Same | Entity type only |
| Owner identity | S16 (name, email), S19 (Sumsub: name, email, personal info) | 2 | Company vs personal check | Create the person once, pass to the provider |
| Questionnaire | S8-S10, then Sumsub step 4 | 2 | Company vs personal questionnaire | Personal questionnaire only for what is not already known |
| Bank statement | S12 (source of funds), S14 (proof of address) | 2 | Two requirements, two upload steps | One upload tagged for both |

Rough count: about 40 required inputs on 18 screens; about 15 of them are re-entries.

## 2. One data model, entered once

```ts
// Entered or confirmed by the user, then reused by every screen and every provider call.
type Company = {
  legalNameAsRegistered: string        // exact spelling from the registration document
  legalNameNormalised?: string         // only if a provider requires another form
  entityType: 'sole_proprietorship' | 'llc' | 'corporation' | 'partnership' | 'other'
  country: string                      // ISO-3166 alpha-2
  registrationNumber: string           // one value; taxId is derived or confirmed
  taxIdSameAsRegistration: boolean
  registeredAddress: Address
  operatingAddress: { sameAsRegistered: true } | Address
  email: string
  phone: string
  website?: string
}

type Activity = {
  naicsCode: string                    // may be suggested from the description
  description: string                  // >= 200 chars is a provider rule, show it up front
  restrictedActivities: string[]       // [] means "none of the above"
  holdsClientFunds: boolean            // single source for all three questions
  expected: {
    annualTurnoverUsd: Range
    monthlyVolumeUsd: Range            // derived from annual unless changed
    monthlyTransactions: Range
    cardSpendUsd: Range | 'none'
    counterpartyCountries: string[]
    targetMarkets: Array<'b2c' | 'b2b' | 'b2g' | 'other'>
  }
  sourceOfFunds: string
}

type Person = {
  role: 'ubo' | 'signatory'
  firstName: string
  lastName: string
  email: string
  ownershipPct: number
  // identity data is collected by the KYC provider, not by us
}

type Document = {
  id: string
  kind: 'registration' | 'bank_statement' | 'utility_bill' | 'contract' | 'invoice' | 'other'
  extracted: Record<string, { value: string; confidence: number; page: number }>
  satisfies: Requirement[]             // one upload can cover several requirements
  issuedAt?: string
}

type Requirement =
  | 'formation' | 'ownership' | 'proof_of_activity'
  | 'source_of_funds' | 'proof_of_address'
```

Rules:
- A field has one owner screen. Later screens show it read-only with an edit link.
- Every value carries its source: typed by user, extracted from document, derived.
- A document satisfies several requirements; the UI ticks them off together.

## 3. Proposed order of steps

| # | Step | What the user does | What the system does |
|---|---|---|---|
| 1 | Who are you | Country, entity type, name, one registration number | Registry lookup if supported; otherwise go straight to manual entry |
| 2 | Documents (one step) | Upload registration document, one bank statement, optionally a contract | Classify, extract name/address/period, compare with step 1, show diffs at once |
| 3 | Review | Confirm prefilled company, address, owner | Show only what is still missing |
| 4 | Activity | One description, expected volumes, client-funds question (once) | Suggest NAICS from the description; derive monthly from annual |
| 5 | Owner | One click "I am the owner" for sole proprietors | Pre-create the UBO, skip the signatory role |
| 6 | Identity | Passport and liveness | Pass name and email already known |
| 7 | Attest | Read-only summary, one checkbox | Submit |

Result: about 18 screens become 7, and the company-name mismatch that cost a full rejection would have appeared on step 2.

## 4. How to measure it

- Screens and required fields per application, before and after.
- Re-entry rate: share of fields entered more than once (target: 0).
- Time to complete and drop-off per step.
- First-pass acceptance (no resubmission) and the share of rejections caused by a mismatch that was detectable at upload.
- Median time from submit to decision.

## 5. Open questions for the team

1. Which duplicates come from the provider (Sumsub / the bank partner) and which are Altitude's own? Only the second group can be changed quickly.
2. Is the registry lookup coverage fixed, or can a country be flagged "manual" up front?
3. Is the Sumsub ID opt-in on by default on purpose?
4. How long is the verification token in the URL valid?
5. Who reviews documents: provider, partner bank, or Altitude's team?
