package se.reflective.quorum.ui

import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import org.junit.Rule
import org.junit.Test
import se.reflective.quorum.corebridge.PreviewQuorumCoreBridge

/**
 * Compose UI test of the capture → consent → queue flow, against the preview
 * bridge so it needs no native core. Runs as an instrumented test (emulator/device).
 */
class SignalCaptureScreenTest {
    @get:Rule
    val composeRule = createComposeRule()

    @Test
    fun draftingThenConsentingSurfacesTheQueuedEvent() {
        composeRule.setContent {
            SignalCaptureScreen(bridge = PreviewQuorumCoreBridge())
        }

        composeRule.onNodeWithText("Create Draft").assertIsDisplayed().performClick()

        val consent = composeRule.onNodeWithText("Consent And Queue")
        consent.assertExists()
        consent.performClick()

        // The Queued Event section renders the human-facing sync label.
        composeRule.onNodeWithText("Queued for sync").assertExists()
    }
}
