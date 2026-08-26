# Presence Light

Presence Light is a two-state presence client for one Cloudflare Worker room. It includes a Tauri desktop overlay and a browser controller.

## Configuration

Open the controller page. Then enter the Worker WebSocket URL and the controller token.

The browser stores the configuration in local storage. The token does not enter the Vite build or the source code.

Choose **Controller** only for a device that can change the status. The Worker must enforce authorization.

Select **Auto launch on startup** to register the desktop application for startup. An automatic start keeps the dot hidden.

After setup, use **Edit config.yml…** from the tray to update the connection and dot appearance.

Both clients add the token as the `token` query parameter. The Worker must accept this token format.

Use `ws://127.0.0.1:8787/...` for a local Worker. Use `wss://...` for production.

For browser development, `VITE_PRESENCE_WS_URL` can supply the initial URL. The user can replace this URL in the controller page.

## Development

Install dependencies:

```sh
pnpm install
```

Start the browser controller:

```sh
pnpm dev
```

Start the desktop application:

```sh
pnpm tauri dev
```

The setup window opens when the desktop application is not configured.

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
