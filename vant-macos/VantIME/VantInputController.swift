import Cocoa
import InputMethodKit

@objc(VantInputController)
class VantInputController: IMKInputController {

    override func handle(_ event: NSEvent!, client sender: Any!) -> Bool {
        // Phase 2 will implement full Telex processing here
        // For now, pass all events through
        return false
    }

    override func activateServer(_ sender: Any!) {
        super.activateServer(sender)
        NSLog("VantIME: Input controller activated")
    }

    override func deactivateServer(_ sender: Any!) {
        NSLog("VantIME: Input controller deactivated")
        super.deactivateServer(sender)
    }
}
