import Cocoa
import InputMethodKit

// Verify the Rust engine is linked and working
let version = String(cString: vant_engine_version())
NSLog("VantIME starting, engine version: \(version)")

let bundleID = Bundle.main.bundleIdentifier!
let connectionName = "\(bundleID)_Connection"

guard let server = IMKServer(name: connectionName, bundleIdentifier: bundleID) else {
    NSLog("VantIME: Failed to create IMKServer with connection: \(connectionName)")
    exit(1)
}

NSLog("VantIME: IMKServer created successfully")
NSApplication.shared.run()
