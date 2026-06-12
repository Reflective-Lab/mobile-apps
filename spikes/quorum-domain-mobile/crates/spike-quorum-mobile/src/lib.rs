//! Mobile-shaped facade over the canonical `quorum-domain` crate.
//!
//! Spike goal: prove that real product domain logic from
//! `marquee-apps/quorum-sense` can be consumed directly by a mobile Rust
//! core. Every function here delegates to `quorum-domain`; nothing is
//! re-implemented. The DTOs are flat, serde-friendly shapes of the kind a
//! UniFFI `.udl` could bind to Swift/Kotlin later.

use quorum_domain::citation::{Citation, CitationParseError};
use quorum_domain::{Confidence, ProbeBudget};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ── Citation facade ──────────────────────────────────────────────────────────

/// Discriminant for [`CitationDto`]. Field-less so it maps 1:1 onto a
/// UniFFI flat enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CitationKind {
    InquiryStatus,
    UnresolvedQuestion,
    AgentResearch,
}

/// Flat DTO for a parsed `quorum://` citation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CitationDto {
    pub kind: CitationKind,
    /// Set for `InquiryStatus` and `UnresolvedQuestion`; `None` for
    /// `AgentResearch`.
    pub id: Option<String>,
    /// Canonical URI, re-rendered by the domain crate.
    pub uri: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FacadeError {
    #[error("invalid citation: {0}")]
    InvalidCitation(String),
    #[error("citation kind `{0:?}` requires an id")]
    MissingId(CitationKind),
}

impl From<CitationParseError> for FacadeError {
    fn from(e: CitationParseError) -> Self {
        Self::InvalidCitation(e.to_string())
    }
}

fn to_dto(citation: &Citation) -> CitationDto {
    let uri = citation.format();
    match citation {
        Citation::InquiryStatus { inquiry_id } => CitationDto {
            kind: CitationKind::InquiryStatus,
            id: Some(inquiry_id.clone()),
            uri,
        },
        Citation::UnresolvedQuestion { id } => CitationDto {
            kind: CitationKind::UnresolvedQuestion,
            id: Some(id.clone()),
            uri,
        },
        Citation::AgentResearch => CitationDto {
            kind: CitationKind::AgentResearch,
            id: None,
            uri,
        },
    }
}

/// Parse a `quorum://` URI using the canonical domain parser.
pub fn parse_citation(uri: &str) -> Result<CitationDto, FacadeError> {
    Ok(to_dto(&Citation::parse(uri)?))
}

/// Render the canonical URI for a citation kind + id, using the domain
/// crate's formatter so mobile can never typo the scheme.
pub fn format_citation(kind: CitationKind, id: Option<&str>) -> Result<CitationDto, FacadeError> {
    let citation = match (kind, id) {
        (CitationKind::InquiryStatus, Some(id)) => Citation::InquiryStatus {
            inquiry_id: id.to_owned(),
        },
        (CitationKind::UnresolvedQuestion, Some(id)) => {
            Citation::UnresolvedQuestion { id: id.to_owned() }
        }
        (CitationKind::AgentResearch, _) => Citation::AgentResearch,
        (kind, None) => return Err(FacadeError::MissingId(kind)),
    };
    Ok(to_dto(&citation))
}

// ── Confidence / probe budget facade ────────────────────────────────────────

/// Mobile capture sliders produce raw floats; the domain crate owns the
/// clamping rule ([0.0, 1.0]). Returns the normalized value.
#[must_use]
pub fn normalize_confidence(raw: f64) -> f64 {
    Confidence::new(raw).value()
}

/// Flat DTO for a probe budget a mobile client wants to propose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProbeBudgetDto {
    pub participant_slots: u32,
    pub time_seconds: u32,
    pub uncertainty_tokens: u32,
}

/// Feasibility check delegated to the canonical `ProbeBudget` invariant.
#[must_use]
pub fn is_probe_budget_feasible(budget: ProbeBudgetDto) -> bool {
    ProbeBudget::new(
        budget.participant_slots,
        budget.time_seconds,
        budget.uncertainty_tokens,
    )
    .is_feasible()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_inquiry_status_uri_from_canonical_parser() {
        let dto = parse_citation("quorum://inquiries/inq-42/status").unwrap();
        assert_eq!(dto.kind, CitationKind::InquiryStatus);
        assert_eq!(dto.id.as_deref(), Some("inq-42"));
        assert_eq!(dto.uri, "quorum://inquiries/inq-42/status");
    }

    #[test]
    fn parses_unresolved_question_and_agent_research() {
        let q = parse_citation("quorum://unresolved-questions/uq-7").unwrap();
        assert_eq!(q.kind, CitationKind::UnresolvedQuestion);
        assert_eq!(q.id.as_deref(), Some("uq-7"));

        let r = parse_citation("quorum://agent/research").unwrap();
        assert_eq!(r.kind, CitationKind::AgentResearch);
        assert_eq!(r.id, None);
    }

    #[test]
    fn rejects_wrong_scheme_and_malformed_paths_via_domain_errors() {
        // Error strings come from quorum-domain's thiserror messages,
        // proving the real domain validation runs behind the facade.
        let wrong = parse_citation("https://example.com").unwrap_err();
        assert_eq!(
            wrong,
            FacadeError::InvalidCitation("URI does not start with `quorum://`".into())
        );

        assert!(parse_citation("quorum://inquiries//status").is_err());
        assert!(parse_citation("quorum://inquiries/abc").is_err());
        assert!(parse_citation("quorum://unresolved-questions/a/b").is_err());
        assert!(parse_citation("quorum://nope").is_err());
    }

    #[test]
    fn format_round_trips_through_parse() {
        let formatted = format_citation(CitationKind::InquiryStatus, Some("inq-9")).unwrap();
        let reparsed = parse_citation(&formatted.uri).unwrap();
        assert_eq!(formatted, reparsed);

        assert_eq!(
            format_citation(CitationKind::UnresolvedQuestion, None).unwrap_err(),
            FacadeError::MissingId(CitationKind::UnresolvedQuestion)
        );
    }

    #[test]
    fn citation_dto_is_serde_friendly() {
        let dto = parse_citation("quorum://agent/research").unwrap();
        let json = serde_json::to_string(&dto).unwrap();
        assert_eq!(
            json,
            r#"{"kind":"agent_research","id":null,"uri":"quorum://agent/research"}"#
        );
        let back: CitationDto = serde_json::from_str(&json).unwrap();
        assert_eq!(back, dto);
    }

    #[test]
    fn confidence_clamps_via_domain_rule() {
        assert_eq!(normalize_confidence(1.5), 1.0);
        assert_eq!(normalize_confidence(-0.2), 0.0);
        assert_eq!(normalize_confidence(0.6), Confidence::medium().value());
    }

    #[test]
    fn probe_budget_feasibility_matches_domain_invariant() {
        let ok = ProbeBudgetDto {
            participant_slots: 3,
            time_seconds: 600,
            uncertainty_tokens: 300,
        };
        assert!(is_probe_budget_feasible(ok));

        let zero_slots = ProbeBudgetDto {
            participant_slots: 0,
            ..ok
        };
        assert!(!is_probe_budget_feasible(zero_slots));
    }
}
