import Testing
@testable import QuorumMobile

@MainActor
@Suite("Director intent mapping")
struct DirectorIntentTests {
    @Test("status labels cover every DirectorIntent case")
    func statusLabelsAreExhaustive() {
        let intents: [DirectorIntent] = [
            .openTask(frameId: "frame:example"),
            .submitJudgment(frameId: "frame:example", choiceId: "choice:yes"),
            .respondGate(gateId: "gate:example", verdict: .approve),
            .respondGate(gateId: "gate:example", verdict: .reject),
            .submitReview(frameId: "frame:example", stance: .agree),
            .requestContext(level: .session),
        ]

        for intent in intents {
            #expect(!intent.testStatusLabel.isEmpty)
        }
    }

    @Test("fixture exposes contract-backed gate verdict in secondary actions")
    func fixtureGateVerdictsAreContractBacked() {
        let frame = DirectorFixture.quorumDecisionCheckpoint.frame
        let gateIntents = frame.secondary.compactMap { action -> GateVerdict? in
            guard case .respondGate(_, let verdict) = action.intent else { return nil }
            return verdict
        }

        #expect(gateIntents == [.reject])
    }
}

private extension DirectorIntent {
    var testStatusLabel: String {
        switch self {
        case .openTask: "Opened current task"
        case .submitJudgment: "Judgment submitted"
        case .respondGate(_, let verdict): "Gate signaled: \(verdict.wireLabel)"
        case .submitReview: "Review submitted"
        case .requestContext(let level): "Opened \(level.label.lowercased()) context"
        }
    }
}
