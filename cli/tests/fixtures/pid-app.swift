import Cocoa

final class Fixture: NSObject {
    let window = NSWindow(contentRect: NSRect(x: 100, y: 200, width: 320, height: 180),
                          styleMask: [.titled, .closable], backing: .buffered, defer: false)
    let button = NSButton(title: "Count 0", target: nil, action: nil)
    let field = NSTextField(frame: NSRect(x: 30, y: 30, width: 260, height: 30))
    var count = 0

    override init() {
        super.init()
        window.title = CommandLine.arguments[1]
        button.frame = NSRect(x: 30, y: 90, width: 260, height: 40)
        button.setAccessibilityIdentifier("pid-button")
        button.target = self
        button.action = #selector(increment)
        field.setAccessibilityIdentifier("pid-field")
        field.stringValue = "untouched"
        window.contentView!.addSubview(button)
        window.contentView!.addSubview(field)
        window.makeKeyAndOrderFront(nil)
    }

    @objc func increment() {
        count += 1
        button.title = "Count \(count)"
    }
}

let app = NSApplication.shared
app.setActivationPolicy(.regular)
let menu = NSMenu()
let edit = NSMenuItem(title: "Edit", action: nil, keyEquivalent: "")
let editMenu = NSMenu(title: "Edit")
editMenu.addItem(withTitle: "Select All", action: #selector(NSText.selectAll(_:)), keyEquivalent: "a")
edit.submenu = editMenu
menu.addItem(edit)
app.mainMenu = menu
let fixture = Fixture()
app.activate(ignoringOtherApps: true)
app.run()
