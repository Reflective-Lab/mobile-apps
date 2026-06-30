package se.reflective.quorum.ui

import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
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

        composeRule.onNodeWithText("Captured signal").performTextInput(
            "The sales team says rollout is fine, but support is seeing confusion.",
        )
        composeRule.onNodeWithText("Create Draft").assertIsDisplayed().performClick()

        val consent = composeRule.onNodeWithText("Accept and queue")
        consent.assertExists()
        consent.performClick()

        composeRule.onNodeWithText("Queued for sync").assertExists()
    }
}
