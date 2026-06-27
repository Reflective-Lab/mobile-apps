import Foundation

// Host-side affordances for director enums generated in `CoreBridge/QuorumFFI.swift`.

extension ContextLevel: CaseIterable, Identifiable {
    public static var allCases: [ContextLevel] {
        [.task, .localContext, .session, .formation, .organization, .everything]
    }

    public var id: Self { self }

    public var label: String {
        switch self {
        case .task: "Task"
        case .localContext: "Local Context"
        case .session: "Session"
        case .formation: "Formation"
        case .organization: "Organization"
        case .everything: "Everything"
        }
    }
}

extension GateVerdict {
    public var wireLabel: String {
        switch self {
        case .approve: "approve"
        case .reject: "reject"
        }
    }
}

extension ReviewStance {
    public var label: String {
        switch self {
        case .agree: "Agree"
        case .disagree: "Disagree"
        case .needMoreContext: "Need context"
        }
    }
}

extension BlockingState {
    public var label: String {
        switch self {
        case .notBlocking: "Not blocking"
        case .blocksFormation: "Cannot continue — formation blocked"
        case .blocksSession: "Cannot continue — session blocked"
        }
    }
}
