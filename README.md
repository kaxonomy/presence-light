# Presence Light

Presence Light is a two-state presence client for one Cloudflare Worker room. It includes a Tauri desktop overlay and a browser controller.

## Configuration

Copy `.env.example` to `.env` for browser development. Set `VITE_PRESENCE_WS_URL` to the Worker WebSocket URL.

The browser stores its controller token in local storage. The token does not enter the Vite build or the source code.

Set these runtime environment variables before you start the desktop application:

```text
PRESENCE_WS_URL=wss://worker.example.com/ws/my-room
PRESENCE_TOKEN=replace-with-viewer-or-controller-token
CAN_CONTROL=true
```

Set `CAN_CONTROL=false` for a viewer. The Worker must enforce authorization because this flag only controls the client UI.

Packaged desktop apps read environment variables at process start. Launch the executable from an environment that defines the three values.

Both clients append the token as the `token` query parameter. The Worker must accept this token format. Existing URL paths and query parameters remain unchanged.

Use `ws://127.0.0.1:8787/...` for a local Worker. Use `wss://...` for production.

## Development

Install dependencies:

```sh
pnpm install
```

Start the browser controller:

```sh
pnpm dev
```

Start the desktop application with the required environment variables:

```sh
PRESENCE_WS_URL=ws://127.0.0.1:8787/ws/example-room \
PRESENCE_TOKEN=replace-me \
CAN_CONTROL=true \
pnpm tauri dev
```

## Checks and builds

```sh
pnpm check
pnpm test
pnpm build
pnpm tauri build
```

## Platform limits

The macOS transparent window uses Tauri private APIs. As a result, the macOS build is for direct distribution, not the Mac App Store.

Linux desktop behavior depends on the compositor. Validate the global shortcut, click-through window, tray, and always-on-top behavior on each target desktop. Some Wayland compositors can restrict these features.

The tray menu works on Linux. Tauri does not support Linux tray-icon mouse events, but this application does not use those events.
