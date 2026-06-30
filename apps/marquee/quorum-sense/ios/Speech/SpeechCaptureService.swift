import AVFoundation
import Speech

/// Native speech permission + transcript capture (M3.3, M3.4).
@MainActor
@Observable
public final class SpeechCaptureService {
    public enum AuthorizationState: Equatable {
        case undetermined
        case authorized
        case denied
        case restricted
    }

    public private(set) var authorizationState: AuthorizationState = .undetermined
    public private(set) var isRecording = false
    public private(set) var transcript = ""
    public private(set) var errorMessage: String?

    private let speechRecognizer = SFSpeechRecognizer()
    private var recognitionRequest: SFSpeechAudioBufferRecognitionRequest?
    private var recognitionTask: SFSpeechRecognitionTask?
    private let audioEngine = AVAudioEngine()

    public init() {}

    public func refreshAuthorization() async {
        let speechStatus = SFSpeechRecognizer.authorizationStatus()
        let micGranted: Bool
        if #available(iOS 17.0, *) {
            micGranted = AVAudioApplication.shared.recordPermission == .granted
        } else {
            micGranted = AVAudioSession.sharedInstance().recordPermission == .granted
        }

        switch (speechStatus, micGranted) {
        case (.authorized, true):
            authorizationState = .authorized
        case (.denied, _), (_, false):
            authorizationState = .denied
        case (.restricted, _):
            authorizationState = .restricted
        default:
            authorizationState = .undetermined
        }
    }

    public func requestAuthorization() async {
        let speechGranted = await withCheckedContinuation { continuation in
            SFSpeechRecognizer.requestAuthorization { status in
                continuation.resume(returning: status == .authorized)
            }
        }

        let micGranted: Bool
        if #available(iOS 17.0, *) {
            micGranted = await AVAudioApplication.requestRecordPermission()
        } else {
            micGranted = await withCheckedContinuation { continuation in
                AVAudioSession.sharedInstance().requestRecordPermission { granted in
                    continuation.resume(returning: granted)
                }
            }
        }

        authorizationState = speechGranted && micGranted ? .authorized : .denied
    }

    public func startRecording() throws {
        guard authorizationState == .authorized else {
            throw CaptureError.permissionDenied
        }
        guard let speechRecognizer, speechRecognizer.isAvailable else {
            throw CaptureError.recognizerUnavailable
        }

        stopRecording()
        transcript = ""
        errorMessage = nil

        let audioSession = AVAudioSession.sharedInstance()
        try audioSession.setCategory(.record, mode: .measurement, options: .duckOthers)
        try audioSession.setActive(true, options: .notifyOthersOnDeactivation)

        let request = SFSpeechAudioBufferRecognitionRequest()
        request.shouldReportPartialResults = true
        recognitionRequest = request

        let inputNode = audioEngine.inputNode
        recognitionTask = speechRecognizer.recognitionTask(with: request) { [weak self] result, error in
            Task { @MainActor in
                guard let self else { return }
                if let result {
                    self.transcript = result.bestTranscription.formattedString
                }
                if error != nil || result?.isFinal == true {
                    self.stopRecording()
                }
            }
        }

        let format = inputNode.outputFormat(forBus: 0)
        inputNode.installTap(onBus: 0, bufferSize: 1024, format: format) { buffer, _ in
            request.append(buffer)
        }

        audioEngine.prepare()
        try audioEngine.start()
        isRecording = true
    }

    public func stopRecording() {
        if audioEngine.isRunning {
            audioEngine.stop()
            audioEngine.inputNode.removeTap(onBus: 0)
        }
        recognitionRequest?.endAudio()
        recognitionTask?.cancel()
        recognitionRequest = nil
        recognitionTask = nil
        isRecording = false
        try? AVAudioSession.sharedInstance().setActive(false, options: .notifyOthersOnDeactivation)
    }

    public enum CaptureError: Error {
        case permissionDenied
        case recognizerUnavailable
    }
}
