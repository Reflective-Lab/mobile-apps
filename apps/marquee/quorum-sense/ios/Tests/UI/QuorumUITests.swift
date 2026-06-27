import XCTest

/// End-to-end UI automation of the capture → consent → queue flow, driving the
/// real app (which runs against the Rust core via QuorumCoreBridgeFFI).
///
/// `@MainActor` because XCUIApplication/XCUIElement are main-actor-isolated under
/// Swift 6 (Xcode 16+); without it the XCUI calls fail to compile.
@MainActor
final class QuorumUITests: XCTestCase {
    override func setUp() {
        super.setUp()
        continueAfterFailure = false
    }

    func testCaptureFlowProducesDraftThenQueuesEvent() {
        let app = XCUIApplication()
        app.launch()

        let signalCapture = app.buttons["Signal Capture"]
        XCTAssertTrue(signalCapture.waitForExistence(timeout: 10), "Signal Capture entry point should be present")
        signalCapture.tap()

        // 1. Draft a signal.
        let createDraft = app.buttons["Create Draft"]
        XCTAssertTrue(createDraft.waitForExistence(timeout: 10), "Create Draft button should be present")
        createDraft.tap()

        // 2. Consent and queue it.
        let consentAndQueue = app.buttons["Consent And Queue"]
        XCTAssertTrue(consentAndQueue.waitForExistence(timeout: 10), "Draft section should appear after drafting")
        consentAndQueue.tap()

        // 3. The Quorum form is a UICollectionView: the Queued Event section sits
        //    below the fold, and off-screen cells aren't instantiated. Scroll it
        //    into view before asserting. LabeledContent merges label+value into a
        //    single element, so match on its text.
        let queued = app.staticTexts.containing(
            NSPredicate(format: "label CONTAINS[c] %@", "Queued")
        ).firstMatch
        for _ in 0..<4 where !queued.exists {
            app.swipeUp()
        }
        XCTAssertTrue(queued.waitForExistence(timeout: 5), "Queued Event section should appear after consent")
    }
}
