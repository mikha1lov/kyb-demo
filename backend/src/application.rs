use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use uuid::Uuid;

use crate::{
    compare::{self, RequirementResult},
    domain::{Extracted, Requirement, Severity, Typed},
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
    pub legal_name: String,
    pub registration_number: String,
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
        if self.legal_name.chars().count() < 2 {
            return Err("legal name is too short");
        }
        if self.registration_number.chars().count() < 5 {
            return Err("registration number is too short");
        }
        if self.address.chars().count() < 5 {
            return Err("address is too short");
        }
        Ok(())
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
pub struct ApplicationView {
    pub id: Uuid,
    pub status: Status,
    pub business: Option<Business>,
    pub sources: Sources,
    pub documents: Vec<DocumentView>,
    pub checklist: Vec<ChecklistItem>,
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

pub fn view(application: ApplicationRow, documents: Vec<DocumentRow>, today: NaiveDate) -> ApplicationView {
    let status = application.status();
    let business = application.business.map(|b| b.0);
    let documents: Vec<DocumentView> = documents
        .into_iter()
        .map(|d| {
            let extracted = d.extracted.0;
            let results = match &business {
                Some(b) => compare::evaluate(&Requirement::ALL, &b.typed(), &extracted, today),
                None => Vec::new(),
            };
            DocumentView { id: d.id, name: d.filename, size: d.size_bytes, extracted, results, created_at: d.created_at }
        })
        .collect();
    let checklist = checklist(&documents);
    let complete = checklist.iter().all(|item| item.document_id.is_some());

    ApplicationView {
        id: application.id,
        status,
        business,
        sources: application.sources.0,
        documents,
        checklist,
        complete,
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
    fn without_business_there_are_no_results() {
        let app = application(None);
        let docs = vec![row(app.id, "cert.pdf", extracted(DocKind::Registration, None, Some("2023-03-14")))];
        let v = view(app, docs, day(2026, 10, 12));
        assert!(v.documents[0].results.is_empty());
        assert!(!v.complete);
    }
}
