package se.reflective.quorum.app

import io.kotest.core.spec.style.FunSpec
import io.kotest.matchers.nulls.shouldBeNull
import io.kotest.matchers.shouldBe
import io.sentry.SentryEvent
import io.sentry.protocol.User

/**
 * The Android crash client must not exfiltrate device/user identity through the
 * Sentry pipe (QF-2026-06-24-05, ADR 0004). `scrubPii` is the `beforeSend` hook
 * registered in [QuorumApplication]; verify it strips identity but preserves the
 * crash payload (message) we still need to debug.
 */
class ScrubPiiSpec : FunSpec({
    test("scrubPii strips device hostname and user identity") {
        val event = SentryEvent().apply {
            serverName = "kenneth-pixel-8"
            user = User().apply {
                id = "user-123"
                username = "field-agent"
                ipAddress = "203.0.113.7"
            }
        }

        val scrubbed = scrubPii(event)

        scrubbed.serverName.shouldBeNull()
        scrubbed.user.shouldBeNull()
    }

    test("scrubPii preserves the crash message (not PII)") {
        val event = SentryEvent().apply {
            message = io.sentry.protocol.Message().apply { formatted = "panic: index out of bounds" }
        }

        val scrubbed = scrubPii(event)

        scrubbed.message?.formatted shouldBe "panic: index out of bounds"
    }
})
