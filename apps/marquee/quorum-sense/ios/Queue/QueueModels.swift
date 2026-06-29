import Foundation

/// Summary of a durable queue record for UI and reload (M4.6).
public struct PersistedQueueRecordSummary: Equatable, Sendable, Identifiable {
    public let recordId: String
    public let queueState: String
    public let updatedAt: String

    public var id: String { recordId }

    public init(recordId: String, queueState: String, updatedAt: String) {
        self.recordId = recordId
        self.queueState = queueState
        self.updatedAt = updatedAt
    }

    /// Parse display fields from opaque JSON without learning the full schema.
    static func fromJSON(_ json: String) -> PersistedQueueRecordSummary? {
        guard let data = json.data(using: .utf8),
              let object = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let recordId = object["record_id"] as? String,
              let queueState = object["queue_state"] as? String,
              let updatedAt = object["updated_at"] as? String
        else {
            return nil
        }
        return PersistedQueueRecordSummary(
            recordId: recordId,
            queueState: queueState,
            updatedAt: updatedAt
        )
    }
}

extension QueueState {
    public var wireLabel: String {
        switch self {
        case .draftLocal: "draft_local"
        case .pendingConsent: "pending_consent"
        case .queued: "queued"
        case .submitting: "submitting"
        case .admitted: "admitted"
        case .rejected: "rejected"
        case .needsReview: "needs_review"
        case .abandoned: "abandoned"
        }
    }

    public var label: String {
        switch self {
        case .draftLocal: "Draft (local)"
        case .pendingConsent: "Pending consent"
        case .queued: "Queued"
        case .submitting: "Submitting"
        case .admitted: "Admitted"
        case .rejected: "Rejected"
        case .needsReview: "Needs review"
        case .abandoned: "Abandoned"
        }
    }
}

extension ConsentDecision {
    public var label: String {
        switch self {
        case .accepted: "Accepted"
        case .editedAndAccepted: "Edited and accepted"
        case .rejected: "Rejected"
        case .savedPrivate: "Saved private"
        case .expired: "Expired"
        }
    }
}

enum QueueTimestamp {
    static func nowISO8601() -> String {
        ISO8601DateFormatter().string(from: Date())
    }
}
