use serde::{Deserialize, Serialize};

/// What the user is trying to prove with an upload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Requirement {
    Formation,
    SourceOfFunds,
    ProofOfAddress,
}

impl Requirement {
    pub const ALL: [Requirement; 3] = [Self::Formation, Self::SourceOfFunds, Self::ProofOfAddress];

    pub fn max_age_days(self) -> Option<i64> {
        match self {
            Self::Formation => None,
            Self::SourceOfFunds => Some(180),
            Self::ProofOfAddress => Some(90),
        }
    }

    pub fn accepted_kinds(self) -> &'static [DocKind] {
        match self {
            Self::Formation => &[DocKind::Registration, DocKind::TaxLetter],
            Self::SourceOfFunds => &[DocKind::BankStatement],
            Self::ProofOfAddress => &[DocKind::BankStatement, DocKind::UtilityBill],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocKind {
    Registration,
    TaxLetter,
    BankStatement,
    UtilityBill,
    Contract,
    Invoice,
    Other,
}

/// One extracted value with where it came from and how sure the model is.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Field {
    pub value: Option<String>,
    pub confidence: f32,
    pub page: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Extracted {
    pub kind: DocKind,
    pub legal_name: Field,
    pub registration_number: Field,
    #[serde(default)]
    pub tax_number: Field,
    pub address: Field,
    pub period_start: Field,
    pub period_end: Field,
    pub issue_date: Field,
    pub notes: Option<String>,
}

/// What the user typed earlier in the form.
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Typed {
    pub legal_name: Option<String>,
    pub registration_number: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case", rename_all_fields = "camelCase")]
pub enum FieldCheck {
    /// Same after normalisation.
    Match,
    /// Overlap, but one side has words the other lacks (for example a patronymic).
    Partial {
        missing_in_typed: Vec<String>,
        missing_in_document: Vec<String>,
    },
    Mismatch,
    /// The document does not contain this field.
    NotFound,
    /// Nothing typed to compare with.
    NotChecked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case", rename_all_fields = "camelCase")]
pub enum Freshness {
    Fresh { age_days: i64, max_days: i64 },
    Stale { age_days: i64, max_days: i64 },
    /// The document shows no usable date.
    Unknown,
    NotRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Blocking,
    Warning,
}

#[derive(Debug, Clone, Serialize)]
pub struct Issue {
    pub code: &'static str,
    pub field: &'static str,
    pub severity: Severity,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Comparison {
    pub kind_accepted: bool,
    pub legal_name: FieldCheck,
    pub registration_number: FieldCheck,
    pub address: FieldCheck,
    pub freshness: Freshness,
    pub issues: Vec<Issue>,
}
