import Cocoa
import InputMethodKit

@objc(VantInputController)
class VantInputController: IMKInputController {

    private var engine: OpaquePointer?

    override init!(server: IMKServer!, delegate: Any!, client inputClient: Any!) {
        super.init(server: server, delegate: delegate, client: inputClient)
        engine = vant_engine_create()
        NSLog("VantIME: Controller created")
    }

    deinit {
        if let engine = engine {
            vant_engine_destroy(engine)
            NSLog("VantIME: Controller destroyed")
        }
    }

    // MARK: - IMK Event Handling

    override func handle(_ event: NSEvent!, client sender: Any!) -> Bool {
        guard let event = event, event.type == .keyDown else {
            return false
        }

        guard let client = sender as? (any IMKTextInput) else {
            return false
        }

        // Pass through modifier combos (Cmd, Ctrl, Option)
        let modifiers = event.modifierFlags
        if modifiers.contains(.command) || modifiers.contains(.control) || modifiers.contains(.option) {
            return false
        }

        let keyCode = event.keyCode
        let isComposing = vant_engine_is_composing(engine) == 1

        // Backspace (keyCode 51)
        if keyCode == 51 {
            guard isComposing else { return false }
            let result = vant_engine_process_key(engine, 0, true, false)
            NSLog("VantIME: Backspace -> event_type=%d", result.event_type.rawValue)
            return handleResult(result, client: client)
        }

        // Escape (keyCode 53)
        if keyCode == 53 {
            guard isComposing else { return false }
            let result = vant_engine_process_key(engine, 0, false, true)
            NSLog("VantIME: Escape -> event_type=%d", result.event_type.rawValue)
            return handleResult(result, client: client)
        }

        // Character keys
        guard let chars = event.characters, let scalar = chars.unicodeScalars.first else {
            return false
        }

        let result = vant_engine_process_key(engine, scalar.value, false, false)
        NSLog("VantIME: key='%@' (U+%04X) -> event_type=%d text='%@'",
              chars, scalar.value, result.event_type.rawValue, extractText(from: result))
        return handleResult(result, client: client)
    }

    // MARK: - Focus / Composition Lifecycle

    override func activateServer(_ sender: Any!) {
        super.activateServer(sender)
        NSLog("VantIME: Input controller activated")
    }

    override func deactivateServer(_ sender: Any!) {
        forceCommit(client: sender as? (any IMKTextInput))
        NSLog("VantIME: Input controller deactivated")
        super.deactivateServer(sender)
    }

    override func commitComposition(_ sender: Any!) {
        guard let client = sender as? (any IMKTextInput) else {
            NSLog("VantIME: commitComposition sender is not IMKTextInput, resetting engine")
            if vant_engine_is_composing(engine) == 1 {
                let _ = vant_engine_reset(engine)
            }
            return
        }
        forceCommit(client: client)
    }

    // MARK: - Helpers

    private func extractText(from result: VantKeyResult) -> String {
        guard result.text_len > 0, let ptr = result.text else { return "" }
        let data = Data(bytes: ptr, count: Int(result.text_len))
        return String(data: data, encoding: .utf8) ?? ""
    }

    private func markedTextAttributes(_ text: String) -> NSAttributedString {
        let attrs: [NSAttributedString.Key: Any] = [
            .underlineStyle: NSUnderlineStyle.single.rawValue,
        ]
        return NSAttributedString(string: text, attributes: attrs)
    }

    private func handleResult(_ result: VantKeyResult, client: any IMKTextInput) -> Bool {
        let text = extractText(from: result)

        switch result.event_type {
        case VantEventType_Composing:
            client.setMarkedText(
                markedTextAttributes(text),
                selectionRange: NSRange(location: text.utf16.count, length: 0),
                replacementRange: NSRange(location: NSNotFound, length: 0)
            )
            return true

        case VantEventType_Committed:
            var insertString = text
            if result.committed_char != 0, let scalar = Unicode.Scalar(result.committed_char) {
                insertString += String(scalar)
            }
            client.insertText(
                insertString as NSString,
                replacementRange: NSRange(location: NSNotFound, length: 0)
            )
            return true

        case VantEventType_Reset:
            client.setMarkedText(
                markedTextAttributes(""),
                selectionRange: NSRange(location: 0, length: 0),
                replacementRange: NSRange(location: NSNotFound, length: 0)
            )
            return true

        case VantEventType_Passthrough:
            return false

        default:
            return false
        }
    }

    private func forceCommit(client: (any IMKTextInput)?) {
        guard let client = client, vant_engine_is_composing(engine) == 1 else { return }
        let result = vant_engine_force_commit(engine)
        let text = extractText(from: result)
        if !text.isEmpty {
            client.setMarkedText(
                markedTextAttributes(""),
                selectionRange: NSRange(location: 0, length: 0),
                replacementRange: NSRange(location: NSNotFound, length: 0)
            )
            client.insertText(
                text as NSString,
                replacementRange: NSRange(location: NSNotFound, length: 0)
            )
            NSLog("VantIME: Force-committed '%@'", text)
        }
    }
}
