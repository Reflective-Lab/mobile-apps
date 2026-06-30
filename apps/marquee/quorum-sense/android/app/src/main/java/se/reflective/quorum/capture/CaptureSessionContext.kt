package se.reflective.quorum.capture

/** Session-scoped capture context injected from the host app (M5.7 / M3.2 parity). */
object CaptureSessionContext {
    private const val DEFAULT_INQUIRY_THREAD_ID = "inq_mobile_launch_risks"

    fun inquiryThreadId(): String =
        System.getenv("QUORUM_INQUIRY_THREAD_ID") ?: DEFAULT_INQUIRY_THREAD_ID
}
