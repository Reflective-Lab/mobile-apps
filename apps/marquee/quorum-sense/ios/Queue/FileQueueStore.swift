import Foundation

/// Native durability adapter for offline queue records (M4.6, ADR 0005).
///
/// Stores opaque JSON blobs keyed by `record_id` under Application Support.
/// Rust validates record shape and transitions before every write.
public actor FileQueueStore {
    public enum StoreError: Error {
        case invalidRecordId(String)
        case unreadableRecord(String)
    }

    private let directory: URL

    public init(subdirectory: String = "queue") throws {
        let appSupport = try FileManager.default.url(
            for: .applicationSupportDirectory,
            in: .userDomainMask,
            appropriateFor: nil,
            create: true
        )
        directory = appSupport.appendingPathComponent(subdirectory, isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
    }

    public func save(recordId: String, json: String) throws {
        try validateRecordId(recordId)
        let fileURL = fileURL(for: recordId)
        let tempURL = directory.appendingPathComponent("\(recordId).json.tmp")
        try json.write(to: tempURL, atomically: true, encoding: .utf8)
        if FileManager.default.fileExists(atPath: fileURL.path) {
            try FileManager.default.removeItem(at: fileURL)
        }
        try FileManager.default.moveItem(at: tempURL, to: fileURL)
    }

    public func load(recordId: String) throws -> String? {
        try validateRecordId(recordId)
        let fileURL = fileURL(for: recordId)
        guard FileManager.default.fileExists(atPath: fileURL.path) else { return nil }
        return try String(contentsOf: fileURL, encoding: .utf8)
    }

    public func allRecordIds() throws -> [String] {
        let urls = try FileManager.default.contentsOfDirectory(
            at: directory,
            includingPropertiesForKeys: nil
        )
        return urls
            .filter { $0.pathExtension == "json" && !$0.lastPathComponent.hasSuffix(".tmp") }
            .map { $0.deletingPathExtension().lastPathComponent }
            .sorted()
    }

    public func loadAllJSON() throws -> [String: String] {
        var records: [String: String] = [:]
        for recordId in try allRecordIds() {
            if let json = try load(recordId: recordId) {
                records[recordId] = json
            }
        }
        return records
    }

    public func remove(recordId: String) throws {
        try validateRecordId(recordId)
        let fileURL = fileURL(for: recordId)
        if FileManager.default.fileExists(atPath: fileURL.path) {
            try FileManager.default.removeItem(at: fileURL)
        }
    }

    private func fileURL(for recordId: String) -> URL {
        directory.appendingPathComponent("\(recordId).json")
    }

    private func validateRecordId(_ recordId: String) throws {
        guard !recordId.isEmpty,
              !recordId.contains("/"),
              !recordId.contains("..")
        else {
            throw StoreError.invalidRecordId(recordId)
        }
    }
}
