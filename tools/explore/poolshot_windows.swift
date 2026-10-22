// poolshot_windows.swift — list the on-screen windows, one per line:
//   <window id> TAB <owner pid> TAB <layer> TAB <width> TAB <height> TAB <owner name>
// `tools/explore/poolshot.py` runs it with `swift` and picks the game window
// by its owner pid's command line. Window ids, pids, layers, bounds and owner
// names need no Screen Recording grant (only `kCGWindowName` would), so this
// runs from any process tree; the photograph itself goes through
// `tools/gamelog/viadriver.sh`.
import CoreGraphics
import Foundation

let opts: CGWindowListOption = [.optionOnScreenOnly, .excludeDesktopElements]
let info = (CGWindowListCopyWindowInfo(opts, kCGNullWindowID) as? [[String: Any]]) ?? []
for w in info {
    let n = (w[kCGWindowNumber as String] as? NSNumber)?.intValue ?? 0
    let p = (w[kCGWindowOwnerPID as String] as? NSNumber)?.intValue ?? 0
    let l = (w[kCGWindowLayer as String] as? NSNumber)?.intValue ?? 0
    let o = (w[kCGWindowOwnerName as String] as? String ?? "")
        .replacingOccurrences(of: "\t", with: " ")
        .replacingOccurrences(of: "\n", with: " ")
    let b = w[kCGWindowBounds as String] as? [String: Any] ?? [:]
    let wd = (b["Width"] as? NSNumber)?.intValue ?? 0
    let ht = (b["Height"] as? NSNumber)?.intValue ?? 0
    print("\(n)\t\(p)\t\(l)\t\(wd)\t\(ht)\t\(o)")
}
