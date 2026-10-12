use std::cmp::Ordering;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use uuid::Uuid;

use crate::{
    compare::{self, NumberKind, RequirementResult},
    domain::{DocKind, Extracted, Field, Requirement, Severity, Typed},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    SoleProprietor,
    Llc,
    Corporation,
    Partnership,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Business {
    pub country: String,
    pub entity_type: EntityType,
    #[serde(default)]
    pub legal_name: String,
    #[serde(default)]
    pub registration_number: String,
    #[serde(default)]
    pub address: String,
}

impl Business {
    pub fn normalized(self) -> Self {
        Self {
            country: self.country.trim().to_uppercase(),
            entity_type: self.entity_type,
            legal_name: self.legal_name.trim().to_string(),
            registration_number: self.registration_number.trim().to_string(),
            address: self.address.trim().to_string(),
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.country.len() != 2 || !self.country.chars().all(|c| c.is_ascii_uppercase()) {
            return Err("country must be a two-letter code");
        }
        if !self.legal_name.is_empty() && self.legal_name.chars().count() < 2 {
            return Err("legal name is too short");
        }
        if !self.registration_number.is_empty() && self.registration_number.chars().count() < 5 {
            return Err("registration number is too short");
        }
        if !self.address.is_empty() && self.address.chars().count() < 5 {
            return Err("address is too short");
        }
        Ok(())
    }

    pub fn details_complete(&self) -> bool {
        self.legal_name.chars().count() >= 2
            && self.registration_number.chars().count() >= 5
            && self.address.chars().count() >= 5
    }

    pub fn typed(&self) -> Typed {
        Typed {
            legal_name: Some(self.legal_name.clone()),
            registration_number: Some(self.registration_number.clone()),
            address: Some(self.address.clone()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Document,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Sources {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legal_name: Option<Source>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration_number: Option<Source>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<Source>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Draft,
    Submitted,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ApplicationRow {
    pub id: Uuid,
    pub status: String,
    pub business: Option<Json<Business>>,
    pub sources: Json<Sources>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ApplicationRow {
    pub fn status(&self) -> Status {
        if self.status == "submitted" { Status::Submitted } else { Status::Draft }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct DocumentRow {
    pub id: Uuid,
    pub application_id: Uuid,
    pub filename: String,
    pub size_bytes: i32,
    pub kind: String,
    pub extracted: Json<Extracted>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentView {
    pub id: Uuid,
    pub name: String,
    pub size: i32,
    pub extracted: Extracted,
    pub results: Vec<RequirementResult>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChecklistItem {
    pub requirement: Requirement,
    pub document_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Suggestion {
    pub value: String,
    pub document_id: Uuid,
    pub confidence: f32,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Suggestions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legal_name: Option<Suggestion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration_number: Option<Suggestion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<Suggestion>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationView {
    pub id: Uuid,
    pub status: Status,
    pub business: Option<Business>,
    pub sources: Sources,
    pub documents: Vec<DocumentView>,
    pub checklist: Vec<ChecklistItem>,
    pub suggestions: Suggestions,
    pub requirements_met: bool,
    pub business_complete: bool,
    pub complete: bool,
    pub submitted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub fn satisfies(result: &RequirementResult) -> bool {
    result.comparison.kind_accepted && !result.comparison.issues.iter().any(|i| i.severity == Severity::Blocking)
}

fn checklist(documents: &[DocumentView]) -> Vec<ChecklistItem> {
    Requirement::ALL
        .iter()
        .map(|&requirement| ChecklistItem {
            requirement,
            document_id: documents
                .iter()
                .find(|d| d.results.iter().any(|r| r.requirement == requirement && satisfies(r)))
                .map(|d| d.id),
        })
        .collect()
}

const MIN_SUGGESTION_CONFIDENCE: f32 = 0.7;

fn authority(kind: DocKind) -> Option<u8> {
    match kind {
        DocKind::Registration => Some(0),
        DocKind::BankStatement => Some(1),
        DocKind::UtilityBill => Some(2),
        _ => None,
    }
}

fn suggest(documents: &[DocumentView], field: impl Fn(&Extracted) -> &Field) -> Option<Suggestion> {
    documents
        .iter()
        .filter_map(|document| {
            let rank = authority(document.extracted.kind)?;
            let field = field(&document.extracted);
            let value = field.value.as_deref()?.trim();
            (!value.is_empty() && field.confidence >= MIN_SUGGESTION_CONFIDENCE)
                .then(|| (rank, field.confidence, document.id, value.to_string()))
        })
        .min_by(|a, b| a.0.cmp(&b.0).then(b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal)))
        .map(|(_, confidence, document_id, value)| Suggestion { value, document_id, confidence })
}

fn suggestions(documents: &[DocumentView]) -> Suggestions {
    Suggestions {
        legal_name: suggest(documents, |e| &e.legal_name),
        registration_number: suggest(documents, |e| &e.tax_number).or_else(|| suggest(documents, |e| &e.registration_number)),
        address: suggest(documents, |e| &e.address),
    }
}

pub fn view(application: ApplicationRow, documents: Vec<DocumentRow>, today: NaiveDate) -> ApplicationView {
    let status = application.status();
    let business = application.business.map(|b| b.0);
    let kind = business.as_ref().map_or(NumberKind::Unknown, |b| {
        let extracted: Vec<&Extracted> = documents.iter().map(|d| &d.extracted.0).collect();
        compare::number_kind(Some(b.registration_number.as_str()), &extracted)
    });
    let documents: Vec<DocumentView> = documents
        .into_iter()
        .map(|d| {
            let extracted = d.extracted.0;
            let results = match &business {
                Some(b) => compare::evaluate_with(&Requirement::ALL, &b.typed(), &extracted, today, kind),
                None => Vec::new(),
            };
            DocumentView { id: d.id, name: d.filename, size: d.size_bytes, extracted, results, created_at: d.created_at }
        })
        .collect();
    let checklist = checklist(&documents);
    let suggestions = suggestions(&documents);
    let requirements_met = checklist.iter().all(|item| item.document_id.is_some());
    let business_complete = business.as_ref().is_some_and(Business::details_complete);

    ApplicationView {
        id: application.id,
        status,
        business,
        sources: application.sources.0,
        documents,
        checklist,
        suggestions,
        requirements_met,
        business_complete,
        complete: requirements_met && business_complete,
        submitted_at: application.submitted_at,
        created_at: application.created_at,
        updated_at: application.updated_at,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DocKind, Field};

    fn business() -> Business {
        Business {
            country: "KG".into(),
            entity_type: EntityType::SoleProprietor,
            legal_name: "Tokoev Aliyar Bekovich".into(),
            registration_number: "20000000000001".into(),
            address: "Bishkek, Testovaya street 10, apt. 5".into(),
        }
    }

    fn field(value: Option<&str>) -> Field {
        Field { value: value.map(String::from), confidence: 0.9, page: Some(1) }
    }

    fn extracted(kind: DocKind, period_end: Option<&str>, issued: Option<&str>) -> Extracted {
        Extracted {
            kind,
            legal_name: field(Some("TOKOEV ALIYAR BEKOVICH PE")),
            registration_number: field(Some("20000000000001")),
            tax_number: field(None),
            address: field(Some("KYRGYZ REPUBLIC, BISHKEK, UL. TESTOVAYA 10, APT 5")),
            period_start: field(None),
            period_end: field(period_end),
            issue_date: field(issued),
            notes: None,
        }
    }

    fn row(application: Uuid, name: &str, extracted: Extracted) -> DocumentRow {
        DocumentRow {
            id: Uuid::new_v4(),
            application_id: application,
            filename: name.into(),
            size_bytes: 100,
            kind: "other".into(),
            extracted: Json(extracted),
            created_at: Utc::now(),
        }
    }

    fn application(business: Option<Business>) -> ApplicationRow {
        ApplicationRow {
            id: Uuid::new_v4(),
            status: "draft".into(),
            business: business.map(Json),
            sources: Json(Sources::default()),
            submitted_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn normalizes_and_validates_business() {
        let messy = Business { country: " kg ".into(), legal_name: "  Acme  ".into(), ..business() };
        let clean = messy.normalized();
        assert_eq!(clean.country, "KG");
        assert_eq!(clean.legal_name, "Acme");
        assert!(clean.validate().is_ok());
        assert!(Business { country: "KGZ".into(), ..business() }.validate().is_err());
        assert!(Business { legal_name: "A".into(), ..business() }.validate().is_err());
        assert!(Business { registration_number: "123".into(), ..business() }.validate().is_err());
        assert!(Business { address: "x".into(), ..business() }.validate().is_err());
    }

    #[test]
    fn sources_serialize_only_what_is_set() {
        let sources = Sources { legal_name: Some(Source::Document), ..Sources::default() };
        assert_eq!(serde_json::to_string(&sources).unwrap(), r#"{"legalName":"document"}"#);
        assert_eq!(serde_json::from_str::<Sources>("{}").unwrap(), Sources::default());
    }

    #[test]
    fn one_recent_statement_covers_two_requirements() {
        let today = day(2026, 10, 12);
        let app = application(Some(business()));
        let docs = vec![row(app.id, "statement.pdf", extracted(DocKind::BankStatement, Some("2026-09-30"), None))];
        let v = view(app, docs, today);
        let covered: Vec<Requirement> =
            v.checklist.iter().filter(|c| c.document_id.is_some()).map(|c| c.requirement).collect();
        assert_eq!(covered, vec![Requirement::SourceOfFunds, Requirement::ProofOfAddress]);
        assert!(!v.complete);
    }

    #[test]
    fn complete_needs_every_requirement() {
        let today = day(2026, 10, 12);
        let app = application(Some(business()));
        let docs = vec![
            row(app.id, "cert.pdf", extracted(DocKind::Registration, None, Some("2023-03-14"))),
            row(app.id, "statement.pdf", extracted(DocKind::BankStatement, Some("2026-09-30"), None)),
        ];
        assert!(view(app, docs, today).complete);
    }

    #[test]
    fn stale_statement_covers_nothing() {
        let today = day(2026, 10, 12);
        let app = application(Some(business()));
        let docs = vec![row(app.id, "old.pdf", extracted(DocKind::BankStatement, Some("2025-08-31"), None))];
        let v = view(app, docs, today);
        assert!(v.checklist.iter().all(|c| c.document_id.is_none()));
    }

    #[test]
    fn a_different_name_blocks_the_document() {
        let today = day(2026, 10, 12);
        let app = application(Some(Business { legal_name: "Aliyar Tokoev".into(), ..business() }));
        let docs = vec![row(app.id, "cert.pdf", extracted(DocKind::Registration, None, Some("2023-03-14")))];
        let v = view(app, docs, today);
        let formation = v.checklist.iter().find(|c| c.requirement == Requirement::Formation).unwrap();
        assert!(formation.document_id.is_none());
    }

    #[test]
    fn a_draft_business_may_leave_the_details_empty() {
        let draft = Business { legal_name: String::new(), registration_number: String::new(), address: String::new(), ..business() };
        assert!(draft.validate().is_ok());
        assert!(!draft.details_complete());
        assert!(business().details_complete());
    }

    #[test]
    fn details_are_missing_until_the_business_is_filled_in() {
        let today = day(2026, 10, 12);
        let draft = Business { legal_name: String::new(), registration_number: String::new(), address: String::new(), ..business() };
        let app = application(Some(draft));
        let docs = vec![
            row(app.id, "cert.pdf", extracted(DocKind::Registration, None, Some("2023-03-14"))),
            row(app.id, "statement.pdf", extracted(DocKind::BankStatement, Some("2026-09-30"), None)),
        ];
        let v = view(app, docs, today);
        assert!(v.requirements_met);
        assert!(!v.business_complete);
        assert!(!v.complete);
    }

    #[test]
    fn suggestions_prefer_the_registration_certificate() {
        let today = day(2026, 10, 12);
        let app = application(Some(business()));
        let mut statement = extracted(DocKind::BankStatement, Some("2026-09-30"), None);
        statement.legal_name = field(Some("SOMEONE ELSE PE"));
        let cert = extracted(DocKind::Registration, None, Some("2023-03-14"));
        let docs = vec![row(app.id, "statement.pdf", statement), row(app.id, "cert.pdf", cert)];
        let cert_id = docs[1].id;
        let v = view(app, docs, today);
        let name = v.suggestions.legal_name.unwrap();
        assert_eq!(name.value, "TOKOEV ALIYAR BEKOVICH PE");
        assert_eq!(name.document_id, cert_id);
    }

    #[test]
    fn suggestions_ignore_unsure_values_and_unrelated_documents() {
        let today = day(2026, 10, 12);
        let app = application(Some(business()));
        let mut unsure = extracted(DocKind::Registration, None, Some("2023-03-14"));
        unsure.address = Field { value: Some("SOMEWHERE".into()), confidence: 0.4, page: Some(1) };
        let invoice = extracted(DocKind::Invoice, None, Some("2026-09-01"));
        let docs = vec![row(app.id, "cert.pdf", unsure), row(app.id, "invoice.pdf", invoice)];
        let v = view(app, docs, today);
        assert!(v.suggestions.address.is_none());
        assert_eq!(v.suggestions.registration_number.unwrap().value, "20000000000001");
    }

    #[test]
    fn without_business_there_are_no_results() {
        let app = application(None);
        let docs = vec![row(app.id, "cert.pdf", extracted(DocKind::Registration, None, Some("2023-03-14")))];
        let v = view(app, docs, day(2026, 10, 12));
        assert!(v.documents[0].results.is_empty());
        assert!(!v.complete);
    }
}
