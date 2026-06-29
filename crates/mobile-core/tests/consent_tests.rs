use reflective_mobile_core::consent::ConsentDecision;

#[test]
fn consent_decision_wire_labels_are_stable() {
    assert_eq!(ConsentDecision::Accepted.as_str(), "accepted");
    assert_eq!(
        ConsentDecision::EditedAndAccepted.as_str(),
        "edited_and_accepted"
    );
    assert_eq!(ConsentDecision::SavedPrivate.as_str(), "saved_private");
}

#[test]
fn consent_decision_parse_rejects_booleans_and_draft_state_strings() {
    assert!(ConsentDecision::parse("true").is_none());
    assert!(ConsentDecision::parse("pending").is_none());
    assert!(ConsentDecision::parse("consented").is_none());
}

#[test]
fn consent_decision_queue_semantics_match_product_contract() {
    assert!(ConsentDecision::Accepted.permits_queue());
    assert!(ConsentDecision::EditedAndAccepted.permits_queue());
    assert!(!ConsentDecision::SavedPrivate.permits_queue());
}
