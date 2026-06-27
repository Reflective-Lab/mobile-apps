package se.reflective.quorum.director

import kotlin.test.Test
import kotlin.test.assertEquals

class DirectorFixtureSpec {
    @Test
    fun directorFixtureUsesUpstreamSequenceVersion() {
        val snapshot = DirectorFixture.quorumDecisionCheckpoint

        assertEquals(1844L, snapshot.version)
        assertEquals("Evaluate Vendor X's security claims", snapshot.frame.now?.objective)
    }

    @Test
    fun gateVerdictsAreContractBackedInSecondaryActions() {
        val verdicts = DirectorFixture.quorumDecisionCheckpoint.frame.secondary.mapNotNull { action ->
            (action.intent as? DirectorIntent.RespondGate)?.verdict
        }

        assertEquals(listOf(GateVerdict.REJECT), verdicts)
    }
}
