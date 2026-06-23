import Testing
@testable import QuorumMobile

/// Property-style tests: invariants checked across many deterministic inputs.
@Suite("Property")
struct PropertyTests {
    static let knownModalities = ["text", "voice_transcript", "image_ocr_text"]

    @Test("Confidence is Some iff the value is finite and within 0...1", arguments: 0..<500)
    func confidenceContract(_ seed: Int) {
        var rng = SeededRNG(seed: UInt64(seed) &+ 1)
        // Mix in-range, out-of-range, and non-finite candidates.
        let pool: [Float] = [
            Float.random(in: 0...1, using: &rng),
            Float.random(in: -5...5, using: &rng),
            [Float.nan, .infinity, -.infinity, -0.0001, 1.0001].randomElement(using: &rng)!,
        ]
        let value = pool.randomElement(using: &rng)!
        let expectedValid = value.isFinite && (0...1).contains(value)
        #expect((Confidence(value) != nil) == expectedValid)
    }

    @Test("a string parses to a modality iff it is a known wire value", arguments: 0..<500)
    func modalityParsing(_ seed: Int) {
        var rng = SeededRNG(seed: UInt64(seed) &+ 99)
        let candidate = Bool.random(using: &rng)
            ? Self.knownModalities.randomElement(using: &rng)!
            : randomWireLikeString(&rng)
        #expect((SignalModality(rawValue: candidate) != nil) == Self.knownModalities.contains(candidate))
    }
}
