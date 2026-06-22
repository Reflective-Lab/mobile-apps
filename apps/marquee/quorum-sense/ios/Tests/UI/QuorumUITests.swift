import XCTest

/// End-to-end UI automation of the capture → consent → queue flow, driving the
/// real app (which runs against the Rust core via QuorumCoreBridgeFFI).
final class QuorumUITests: XCTestCase {
    override func setUp() {
        super.setUp()
        continueAfterFailure = false
    }

    func testCaptureFlowProducesDraftThenQueuesEvent() {
        let app = XCUIApplication()
        app.launch()

        let createDraft = app.buttons["Create Draft"]
        XCTAssertTrue(createDraft.waitForExistence(timeout: 10), "Create Draft button should be present")
        createDraft.tap()

        // A draft renders a "Consent And Queue" button.
        let consentAndQueue = app.buttons["Consent And Queue"]
        XCTAssertTrue(consentAndQueue.waitForExistence(timeout: 10), "Draft section should appear after drafting")
        consentAndQueue.tap()

        // The queued-event section renders the sync state. LabeledContent merges
        // label+value into one element, so match on its text rather than an id.
        let queued = app.staticTexts.containing(
            NSPredicate(format: "label CONTAINS[c] %@", "Queued")
        ).firstMatch
        XCTAssertTrue(queued.waitForExistence(timeout: 10), "Queued Event section should appear after consent")
    }
}
