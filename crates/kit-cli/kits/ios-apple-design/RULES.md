## Native iOS (Kit: iOS / Apple Design)

- Target the current iOS SDK with Swift 6 and SwiftUI. Use UIKit only where SwiftUI cannot do the job, and say why.
- Build with system components first. Follow the `apple-design` skill, and read the Human Interface Guidelines page it links (through the sosumi MCP server) before inventing a custom pattern.
- A screen is done only when it works in light and dark mode, at the largest accessibility text size, and with VoiceOver. Show a preview or simulator screenshot for each.
- Use `@Observable` models, SwiftData for persistence, and Swift Testing for new tests. Ask before adding a third-party dependency.
- When skills disagree, Apple's guidelines win, then `swiftui-expert-skill`, then `swiftui-pro`.
- Build and test with the XcodeBuildMCP tools, and report the result instead of assuming it compiles.
