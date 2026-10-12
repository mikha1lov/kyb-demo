//! Compare what the user typed with what a document says.
//!
//! Pure functions, no I/O: easy to test and to reuse behind any extractor.

use chrono::NaiveDate;
use serde::Serialize;

use crate::domain::{
    Comparison, DocKind, Extracted, Field, FieldCheck, Freshness, Issue, Requirement, Severity, Typed,
};

#[derive(Debug, Clone, Serialize)]
pub struct RequirementResult {
    pub requirement: Requirement,
    pub comparison: Comparison,
}

pub fn evaluate(requirements: &[Requirement], typed: &Typed, extracted: &Extracted, today: NaiveDate) -> Vec<RequirementResult> {
    let kind = number_kind(typed.registration_number.as_deref(), &[extracted]);
    evaluate_with(requirements, typed, extracted, today, kind)
}

pub fn evaluate_with(
    requirements: &[Requirement],
    typed: &Typed,
    extracted: &Extracted,
    today: NaiveDate,
    kind: NumberKind,
) -> Vec<RequirementResult> {
    requirements
        .iter()
        .map(|&requirement| RequirementResult {
            requirement,
            comparison: compare_with(requirement, typed, extracted, today, kind),
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberKind {
    Tax,
    Registry,
    Unknown,
}

/// Legal-form words that do not identify a person or company.
const LEGAL_FORM: &[&str] = &["pe", "ip", "ie", "llc", "ltd", "inc", "ooo", "osoo", "zao", "oao"];

/// Address words that differ between writings of the same address.
const ADDRESS_NOISE: &[&str] = &[
    "ul", "street", "str", "st", "apt", "kv", "kvartira", "g", "city", "d", "dom", "house", "republic",
    "respublika", "kyrgyz", "kyrgyzskaya", "kyrgyzstan", "kg", "kr",
];

fn translit(c: char) -> &'static str {
    match c {
        'а' => "a", 'б' => "b", 'в' => "v", 'г' => "g", 'д' => "d", 'е' | 'э' => "e", 'ё' => "e",
        'ж' => "zh", 'з' => "z", 'и' => "i", 'й' => "y", 'к' => "k", 'л' => "l", 'м' => "m",
        'н' => "n", 'о' => "o", 'п' => "p", 'р' => "r", 'с' => "s", 'т' => "t", 'у' => "u",
        'ф' => "f", 'х' => "kh", 'ц' => "ts", 'ч' => "ch", 'ш' => "sh", 'щ' => "shch",
        'ъ' | 'ь' => "", 'ы' => "y", 'ю' => "yu", 'я' => "ya", 'ү' => "u", 'ө' => "o", 'ң' => "n",
        _ => "",
    }
}

/// Lower-case, transliterate Cyrillic to Latin, split into alphanumeric tokens, drop noise.
fn tokens(input: &str, noise: &[&str]) -> Vec<String> {
    let mut latin = String::with_capacity(input.len());
    for c in input.chars().flat_map(|c| c.to_lowercase()) {
        if c.is_ascii_alphanumeric() {
            latin.push(c);
        } else if c.is_alphabetic() {
            latin.push_str(translit(c));
        } else {
            latin.push(' ');
        }
    }
    let mut out: Vec<String> = Vec::new();
    for t in latin.split_whitespace() {
        if noise.contains(&t) {
            continue;
        }
        if !out.iter().any(|x| x == t) {
            out.push(t.to_string());
        }
    }
    out
}

fn levenshtein(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut cur = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            cur.push((prev[j] + cost).min(prev[j + 1] + 1).min(cur[j] + 1));
        }
        prev = cur;
    }
    prev[b.len()]
}

/// Equal, or one edit apart for longer words (transliteration variants).
fn same_token(a: &str, b: &str) -> bool {
    a == b || (a.len() >= 5 && b.len() >= 5 && levenshtein(a, b) <= 1)
}

fn compare_sets(typed: &[String], doc: &[String], partial_from: f32) -> FieldCheck {
    let missing_in_document: Vec<String> =
        typed.iter().filter(|t| !doc.iter().any(|d| same_token(t, d))).cloned().collect();
    let missing_in_typed: Vec<String> =
        doc.iter().filter(|d| !typed.iter().any(|t| same_token(t, d))).cloned().collect();
    if missing_in_document.is_empty() && missing_in_typed.is_empty() {
        return FieldCheck::Match;
    }
    let overlap = (typed.len() - missing_in_document.len()) as f32;
    let ratio = overlap / typed.len().max(doc.len()).max(1) as f32;
    if overlap > 0.0 && ratio >= partial_from {
        FieldCheck::Partial { missing_in_typed, missing_in_document }
    } else {
        FieldCheck::Mismatch
    }
}

fn present(field: &Field) -> Option<&str> {
    field.value.as_deref().map(str::trim).filter(|v| !v.is_empty())
}

pub fn compare_name(typed: Option<&str>, doc: &Field) -> FieldCheck {
    let Some(typed) = typed.filter(|t| !t.trim().is_empty()) else { return FieldCheck::NotChecked };
    let Some(doc) = present(doc) else { return FieldCheck::NotFound };
    compare_sets(&tokens(typed, LEGAL_FORM), &tokens(doc, LEGAL_FORM), 0.5)
}

pub fn compare_address(typed: Option<&str>, doc: &Field) -> FieldCheck {
    let Some(typed) = typed.filter(|t| !t.trim().is_empty()) else { return FieldCheck::NotChecked };
    let Some(doc) = present(doc) else { return FieldCheck::NotFound };
    compare_sets(&tokens(typed, ADDRESS_NOISE), &tokens(doc, ADDRESS_NOISE), 0.5)
}

fn normalized(value: &str) -> String {
    value.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_uppercase).collect()
}

fn checks_number(kind: DocKind) -> bool {
    matches!(kind, DocKind::Registration | DocKind::TaxLetter | DocKind::BankStatement)
}

fn identifier(field: &Field) -> Option<String> {
    present(field).map(normalized).filter(|d| !d.is_empty())
}

pub fn number_kind(typed: Option<&str>, documents: &[&Extracted]) -> NumberKind {
    let Some(typed) = typed.map(normalized).filter(|t| !t.is_empty()) else { return NumberKind::Unknown };
    let documents: Vec<&&Extracted> = documents.iter().filter(|d| checks_number(d.kind)).collect();
    if documents.iter().any(|d| identifier(&d.tax_number).as_deref() == Some(typed.as_str())) {
        NumberKind::Tax
    } else if documents.iter().any(|d| identifier(&d.registration_number).as_deref() == Some(typed.as_str())) {
        NumberKind::Registry
    } else {
        NumberKind::Unknown
    }
}

pub fn compare_number(typed: Option<&str>, doc: &Extracted, kind: NumberKind) -> FieldCheck {
    let Some(typed) = typed.map(normalized).filter(|t| !t.is_empty()) else { return FieldCheck::NotChecked };
    let tax = identifier(&doc.tax_number);
    let registry = identifier(&doc.registration_number);
    let candidates: Vec<String> = match kind {
        NumberKind::Tax => tax.into_iter().collect(),
        NumberKind::Registry => registry.into_iter().collect(),
        NumberKind::Unknown => tax.into_iter().chain(registry).collect(),
    };
    if candidates.is_empty() {
        return if kind == NumberKind::Unknown { FieldCheck::NotFound } else { FieldCheck::NotChecked };
    }
    if candidates.contains(&typed) { FieldCheck::Match } else { FieldCheck::Mismatch }
}

fn parse_date(field: &Field) -> Option<NaiveDate> {
    present(field).and_then(|v| NaiveDate::parse_from_str(v, "%Y-%m-%d").ok())
}

pub fn freshness(req: Requirement, doc: &Extracted, today: NaiveDate) -> Freshness {
    let Some(max_days) = req.max_age_days() else { return Freshness::NotRequired };
    let Some(date) = parse_date(&doc.period_end).or_else(|| parse_date(&doc.issue_date)) else {
        return Freshness::Unknown;
    };
    let age_days = (today - date).num_days();
    if age_days <= max_days {
        Freshness::Fresh { age_days, max_days }
    } else {
        Freshness::Stale { age_days, max_days }
    }
}

pub fn compare(req: Requirement, typed: &Typed, doc: &Extracted, today: NaiveDate) -> Comparison {
    let kind = number_kind(typed.registration_number.as_deref(), &[doc]);
    compare_with(req, typed, doc, today, kind)
}

pub fn compare_with(req: Requirement, typed: &Typed, doc: &Extracted, today: NaiveDate, number_kind: NumberKind) -> Comparison {
    let kind_accepted = req.accepted_kinds().contains(&doc.kind);
    let legal_name = compare_name(typed.legal_name.as_deref(), &doc.legal_name);
    let registration_number = if checks_number(doc.kind) {
        compare_number(typed.registration_number.as_deref(), doc, number_kind)
    } else {
        FieldCheck::NotChecked
    };
    let address = compare_address(typed.address.as_deref(), &doc.address);
    let freshness = freshness(req, doc, today);

    let mut issues = Vec::new();
    if !kind_accepted {
        issues.push(Issue {
            code: "wrong_document_kind",
            field: "kind",
            severity: Severity::Blocking,
            message: format!("This looks like a {:?}, which does not satisfy this requirement.", doc.kind),
        });
    }
    match &legal_name {
        FieldCheck::Mismatch => issues.push(Issue {
            code: "name_mismatch",
            field: "legalName",
            severity: Severity::Blocking,
            message: "The name in the document is different from the one you entered.".into(),
        }),
        FieldCheck::Partial { missing_in_typed, missing_in_document } => {
            let mut parts = Vec::new();
            if !missing_in_typed.is_empty() {
                parts.push(format!("the document also has \"{}\"", missing_in_typed.join(" ")));
            }
            if !missing_in_document.is_empty() {
                parts.push(format!("you also typed \"{}\"", missing_in_document.join(" ")));
            }
            issues.push(Issue {
                code: "name_partial",
                field: "legalName",
                severity: Severity::Blocking,
                message: format!("The names differ: {}.", parts.join("; ")),
            });
        }
        _ => {}
    }
    if registration_number == FieldCheck::Mismatch {
        issues.push(Issue {
            code: "number_mismatch",
            field: "registrationNumber",
            severity: Severity::Blocking,
            message: "The registration number in the document is different from the one you entered.".into(),
        });
    }
    if matches!(address, FieldCheck::Mismatch | FieldCheck::Partial { .. }) {
        issues.push(Issue {
            code: "address_differs",
            field: "address",
            severity: Severity::Warning,
            message: "The address in the document differs from the one you entered.".into(),
        });
    }
    match &freshness {
        Freshness::Stale { age_days, max_days } => issues.push(Issue {
            code: "document_stale",
            field: "date",
            severity: Severity::Blocking,
            message: format!("The document is {age_days} days old; this requirement allows {max_days} days."),
        }),
        Freshness::Unknown => issues.push(Issue {
            code: "date_unreadable",
            field: "date",
            severity: Severity::Warning,
            message: "We could not read a date on this document.".into(),
        }),
        _ => {}
    }

    Comparison { kind_accepted, legal_name, registration_number, address, freshness, issues }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DocKind;

    fn f(v: &str) -> Field {
        Field { value: Some(v.to_string()), confidence: 0.9, page: Some(1) }
    }
    fn none() -> Field {
        Field { value: None, confidence: 0.0, page: None }
    }
    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }
    fn doc(kind: DocKind) -> Extracted {
        Extracted {
            kind,
            legal_name: f("TOKOEV ALIYAR BEKOVICH PE"),
            registration_number: f("20000000000001"),
            tax_number: none(),
            address: f("KYRGYZ REPUBLIC, BISHKEK, UL. TESTOVAYA 10, APT 5"),
            period_start: none(),
            period_end: f("2026-09-30"),
            issue_date: none(),
            notes: None,
        }
    }

    #[test]
    fn missing_patronymic_is_a_partial_match_not_a_mismatch() {
        let r = compare_name(Some("Aliyar Tokoev PE"), &f("TOKOEV ALIYAR BEKOVICH PE"));
        assert_eq!(
            r,
            FieldCheck::Partial { missing_in_typed: vec!["bekovich".into()], missing_in_document: vec![] }
        );
    }

    #[test]
    fn word_order_and_legal_form_do_not_matter() {
        assert_eq!(compare_name(Some("Aliyar Bekovich Tokoev"), &f("TOKOEV ALIYAR BEKOVICH PE")), FieldCheck::Match);
        assert_eq!(compare_name(Some("IP Tokoev Aliyar Bekovich"), &f("TOKOEV ALIYAR BEKOVICH PE")), FieldCheck::Match);
    }

    #[test]
    fn cyrillic_document_matches_latin_input() {
        assert_eq!(compare_name(Some("Tokoev Aliyar Bekovich"), &f("ИП ТОКОЕВ АЛИЯР БЕКОВИЧ")), FieldCheck::Match);
    }

    #[test]
    fn transliteration_variant_is_tolerated() {
        assert_eq!(compare_name(Some("Tokoyev Aliyar Bekovich"), &f("TOKOEV ALIYAR BEKOVICH PE")), FieldCheck::Match);
    }

    #[test]
    fn different_person_is_a_mismatch() {
        assert_eq!(compare_name(Some("John Smith"), &f("TOKOEV ALIYAR BEKOVICH PE")), FieldCheck::Mismatch);
    }

    #[test]
    fn nothing_typed_or_nothing_found() {
        assert_eq!(compare_name(None, &f("X Y")), FieldCheck::NotChecked);
        assert_eq!(compare_name(Some("X Y"), &none()), FieldCheck::NotFound);
    }

    #[test]
    fn number_ignores_spaces_and_dashes() {
        let e = doc(DocKind::Registration);
        assert_eq!(compare_number(Some("2000 0000-0000 01"), &e, NumberKind::Unknown), FieldCheck::Match);
        assert_eq!(compare_number(Some("20000000000002"), &e, NumberKind::Unknown), FieldCheck::Mismatch);
    }

    fn us_certificate() -> Extracted {
        Extracted { registration_number: f("0000000001"), tax_number: none(), ..doc(DocKind::Registration) }
    }

    fn us_ein_letter() -> Extracted {
        Extracted { registration_number: none(), tax_number: f("99-0000001"), ..doc(DocKind::Registration) }
    }

    #[test]
    fn the_kind_of_the_typed_number_follows_the_documents() {
        let docs = [&us_certificate(), &us_ein_letter()];
        assert_eq!(number_kind(Some("99-0000001"), &docs), NumberKind::Tax);
        assert_eq!(number_kind(Some("0000000001"), &docs), NumberKind::Registry);
        assert_eq!(number_kind(Some("12345678"), &docs), NumberKind::Unknown);
        assert_eq!(number_kind(None, &docs), NumberKind::Unknown);
    }

    #[test]
    fn an_identifier_of_another_type_is_not_a_conflict() {
        assert_eq!(compare_number(Some("99-0000001"), &us_certificate(), NumberKind::Tax), FieldCheck::NotChecked);
        assert_eq!(compare_number(Some("99-0000001"), &us_ein_letter(), NumberKind::Tax), FieldCheck::Match);
        assert_eq!(compare_number(Some("0000000001"), &us_ein_letter(), NumberKind::Registry), FieldCheck::NotChecked);
        assert_eq!(compare_number(Some("0000000001"), &us_certificate(), NumberKind::Registry), FieldCheck::Match);
    }

    #[test]
    fn letters_in_an_identifier_matter() {
        let vat = Extracted { tax_number: f("DE123456789"), ..us_ein_letter() };
        assert_eq!(compare_number(Some("DE 123-456-789"), &vat, NumberKind::Tax), FieldCheck::Match);
        assert_eq!(compare_number(Some("123456789"), &vat, NumberKind::Tax), FieldCheck::Mismatch);
        assert_eq!(compare_number(Some("FR123456789"), &vat, NumberKind::Tax), FieldCheck::Mismatch);
    }

    #[test]
    fn documents_that_are_not_checked_do_not_decide_the_kind() {
        let statement = Extracted { registration_number: none(), tax_number: f("99-0000001"), ..doc(DocKind::BankStatement) };
        let bill = Extracted { registration_number: f("12345678"), tax_number: none(), ..doc(DocKind::UtilityBill) };
        let docs = [&statement, &bill];
        assert_eq!(number_kind(Some("12345678"), &docs), NumberKind::Unknown);
        assert_eq!(compare_number(Some("12345678"), &statement, NumberKind::Unknown), FieldCheck::Mismatch);
    }

    #[test]
    fn a_different_number_of_the_same_type_is_still_a_conflict() {
        let other = Extracted { tax_number: f("11-1111111"), ..us_ein_letter() };
        assert_eq!(compare_number(Some("99-0000001"), &other, NumberKind::Tax), FieldCheck::Mismatch);
    }

    #[test]
    fn address_written_differently_still_matches() {
        assert_eq!(
            compare_address(Some("Bishkek, Testovaya street 10, apt. 5"), &f("KYRGYZ REPUBLIC, BISHKEK, UL. TESTOVAYA 10, APT 5")),
            FieldCheck::Match
        );
        assert!(matches!(
            compare_address(Some("Bishkek, Other street 99"), &f("KYRGYZ REPUBLIC, BISHKEK, UL. TESTOVAYA 10, APT 5")),
            FieldCheck::Partial { .. } | FieldCheck::Mismatch
        ));
    }

    #[test]
    fn freshness_uses_the_requirement_window() {
        let today = d(2026, 10, 12);
        let e = doc(DocKind::BankStatement);
        assert!(matches!(freshness(Requirement::ProofOfAddress, &e, today), Freshness::Fresh { .. }));
        let old = Extracted { period_end: f("2025-08-31"), ..doc(DocKind::BankStatement) };
        assert!(matches!(freshness(Requirement::ProofOfAddress, &old, today), Freshness::Stale { .. }));
        assert!(matches!(freshness(Requirement::SourceOfFunds, &old, today), Freshness::Stale { .. }));
        assert_eq!(freshness(Requirement::Formation, &old, today), Freshness::NotRequired);
    }

    #[test]
    fn the_case_from_the_walkthrough_is_caught_at_upload() {
        let typed = Typed {
            legal_name: Some("Aliyar Tokoev PE".into()),
            registration_number: Some("20000000000001".into()),
            address: None,
        };
        let c = compare(Requirement::Formation, &typed, &doc(DocKind::Registration), d(2026, 10, 12));
        assert!(c.kind_accepted);
        assert_eq!(c.issues.len(), 1);
        assert_eq!(c.issues[0].code, "name_partial");
        assert_eq!(c.issues[0].severity, Severity::Blocking);
    }

    #[test]
    fn a_utility_bill_account_number_is_not_compared_with_the_registration_number() {
        let typed = Typed {
            legal_name: Some("Tokoev Aliyar Bekovich".into()),
            registration_number: Some("20000000000001".into()),
            address: None,
        };
        let mut bill = doc(DocKind::UtilityBill);
        bill.registration_number = f("00-000-0001");
        bill.period_end = f("2026-09-30");
        let c = compare(Requirement::ProofOfAddress, &typed, &bill, d(2026, 10, 12));
        assert_eq!(c.registration_number, FieldCheck::NotChecked);
        assert!(c.issues.is_empty());
    }

    #[test]
    fn a_tax_letter_covers_the_registration_requirement() {
        let typed = Typed { legal_name: None, registration_number: Some("99-0000001".into()), address: None };
        let c = compare(Requirement::Formation, &typed, &us_ein_letter_as(DocKind::TaxLetter), d(2026, 10, 12));
        assert!(c.kind_accepted);
        assert_eq!(c.registration_number, FieldCheck::Match);
        assert!(c.issues.is_empty());
    }

    fn us_ein_letter_as(kind: DocKind) -> Extracted {
        Extracted { kind, ..us_ein_letter() }
    }

    #[test]
    fn wrong_kind_is_reported() {
        let c = compare(Requirement::Formation, &Typed::default(), &doc(DocKind::BankStatement), d(2026, 10, 12));
        assert!(!c.kind_accepted);
        assert!(c.issues.iter().any(|i| i.code == "wrong_document_kind"));
    }
}
