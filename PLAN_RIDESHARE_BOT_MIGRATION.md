# Rideshare Bot Swift-to-Rust dependency map

Source baseline: `Biddlebaddleboo/rideshare-bot` `main` at
`bb72e8eccdd73305fa5e7582e59b4da059ec389b` (verified equal to `origin/main`). This map records
reusable framework dependencies, not app migration work or Rideshare-specific protocol scope.

| Swift source | Existing reusable Rust coverage | Remaining reusable gap / priority |
| --- | --- | --- |
| `ios/RideshareBot/Networking/DatagramTransport.swift`, `ios/RideshareBot/Networking/ProtocolTransport.swift` | `framework-connection` / `ios-connection` provide outbound secure TCP streams; B497 adds bounded IPv4 UDP sessions in `framework-datagram` / `ios-datagram` | App protocol framing, UDP request semantics and message limits remain app-owned. No TCP listener, Bonjour, or `Network.framework` usage was found in Rideshare Bot; those are not current prerequisites (P0 transport now covered) |
| `ios/RideshareBot/Networking/PeerNetworking.swift` | B497 supplies bind/send/receive/source endpoint and cancellation primitives for the app's passive UDP socket | F2/F3 probe/reply behavior, peer registry and retry policy remain app-owned. The observed reply is unicast to the source endpoint; no framework peer-discovery or multicast API is currently required |
| `ios/RideshareBot/Networking/BotAPIClient.swift`, `ios/RideshareBot/RemoteHopp/RemoteHoppCoordinator.swift`, `ios/RideshareBot/WazeRelay/WazeRelayServer.swift` | `framework-network` / `ios-network` provide foreground HTTP; B497 supplies generic UDP socket operations | Bot/Waze payloads, keepalive and mailbox semantics remain app-owned. Connection recovery/background lifecycle is a separate P1 gap only where coordinator behavior requires it; no TCP listener or `Network.framework` dependency was found |
| `ios/RideshareBot/RemoteHopp/RemoteHoppLocationCoordinator.swift`, `ios/RideshareBot/RemoteHopp/RemoteHoppModels.swift` | `framework-location` / `ios-location` provide location contracts and foreground update foundations; B496 is the active background-location increment | Reconcile B496 before selecting any next location lifecycle gap; verify authorization transitions, continuous updates, cancellation and background behavior (P0) |
| `ios/RideshareBot/WazeRelay/WazeRelayNotificationManager.swift` | `framework-notifications` / `ios-notifications` provide local authorization and scheduling; `ios-notification-responses` provides an opt-in tap-response bridge | Reusable foreground `willPresent` delegate support for banner/sound/badge is missing (P0); the app has no `didReceive` tap-routing handler |
| `ios/RideshareBot/RemoteHopp/RemoteHoppCallKitCoordinator.swift`, `ios/RideshareBot/RemoteHopp/RemoteHoppCoordinator.swift` | `ios-call-observer` provides only a one-shot aggregate call snapshot | Provider lifecycle, incoming/outgoing call reporting, action callbacks and background-task cleanup (P1) |
| `ios/RideshareBot/RideshareBotApp.swift`, `ios/RideshareBot/ContentView.swift` | `ios-app-lifecycle`, `ios-app-scenes`, `ios-background-execution` cover selected lifecycle/background primitives | Reconcile app/scene ownership and connection recovery against actual coordinator use (P1) |
| `ios/RideshareBot/ContentView.swift`, `ios/RideshareBot/Views/MainDashboardView.swift`, `ios/RideshareBot/Views/ConnectionSettingsView.swift`, `ios/RideshareBot/WazeRelay/WazeRelayView.swift` | `ios-ui` has UIKit-native view/control increments, including `IosTextField` | Navigation, state observation, screen composition and product-specific UI state remain incomplete (P1) |
| `ios/RideshareBot/Logging/AppLog.swift`, settings and persisted state in `ios/RideshareBot/Networking/BotAPIClient.swift` / coordinators | Preferences and structured-data foundations exist | Compare exact persistence/config and logging behavior before expanding; no new app-specific API is in scope here (P2) |

B497 implements only the reusable UDP socket/session row above. It does not add F2/F3, any
Rideshare type/protocol, peer discovery, multicast, a TCP listener, Bonjour, `Network.framework`,
permission UI, or app integration, and does not claim that any Swift component has been migrated.
