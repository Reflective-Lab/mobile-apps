import Foundation

/// Session-scoped capture context injected from the host app (M3.2).
public enum CaptureSessionContext {
    private static let defaultInquiryThreadId = "inq_mobile_launch_risks"

    /// Resolve the active inquiry thread from scheme env or fall back to the demo id.
    public static func inquiryThreadId() -> String {
        ProcessInfo.processInfo.environment["QUORUM_INQUIRY_THREAD_ID"] ?? defaultInquiryThreadId
    }
}
