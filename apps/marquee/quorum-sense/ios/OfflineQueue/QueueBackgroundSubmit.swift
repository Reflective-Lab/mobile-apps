import BackgroundTasks
import Foundation

/// BGTaskScheduler hook for durable queue submission (M4.7).
public enum QueueBackgroundSubmit {
    public static let taskIdentifier = "se.reflective.quorum.queue-submit"

    public static func register(submitHandler: @escaping @Sendable () async -> Int) {
        BGTaskScheduler.shared.register(
            forTaskWithIdentifier: taskIdentifier,
            using: nil
        ) { task in
            guard let processing = task as? BGProcessingTask else {
                task.setTaskCompleted(success: false)
                return
            }
            processing.expirationHandler = {
                processing.setTaskCompleted(success: false)
            }
            Task {
                let count = await submitHandler()
                processing.setTaskCompleted(success: count >= 0)
                if count > 0 {
                    schedule()
                }
            }
        }
    }

    public static func schedule() {
        let request = BGProcessingTaskRequest(identifier: taskIdentifier)
        request.requiresNetworkConnectivity = true
        request.requiresExternalPower = false
        try? BGTaskScheduler.shared.submit(request)
    }
}
