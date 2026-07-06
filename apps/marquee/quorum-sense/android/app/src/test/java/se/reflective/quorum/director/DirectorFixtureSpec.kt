package se.reflective.quorum.director

import io.kotest.core.spec.style.FunSpec
import io.kotest.matchers.shouldBe

class DirectorFixtureSpec : FunSpec({
    test("director fixture uses upstream sequence version") {
        val snapshot = DirectorFixture.quorumDecisionCheckpoint

        snapshot.version shouldBe 1844L
        snapshot.frame.now?.objective shouldBe "Evaluate Vendor X's security claims"
    }

    test("gate verdicts are contract-backed in secondary actions") {
        val verdicts = DirectorFixture.quorumDecisionCheckpoint.frame.secondary.mapNotNull { action ->
            (action.intent as? DirectorIntent.RespondGate)?.verdict
        }

        verdicts shouldBe listOf(GateVerdict.REJECT)
    }
})
