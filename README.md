# Presence Light

Presence Light is a two-state presence client for one Cloudflare Worker room. It includes a Tauri desktop overlay and a browser controller.

## Configuration

Open the controller page. Then enter the Worker WebSocket URL and the controller token.

The browser stores the configuration in local storage. The token does not enter the Vite build or the source code.

Choose **Controller** only for a device that can change the status. The Worker must enforce authorization.

Select **Auto launch on startup** to register the desktop application for startup. An automatic start keeps the dot hidden.

After setup, use **Edit Config** from the tray to update the connection and dot appearance.

Both clients add the token as the `token` query parameter. The Worker must accept this token format.

Use `ws://127.0.0.1:8787/...` for a local Worker. Use `wss://...` for production.

For browser development, `VITE_PRESENCE_WS_URL` can supply the initial URL. The user can replace this URL in the controller page.

## Chime through your microphone

Only the desktop controller sends the Busy chime. Viewer clients stay silent.
The controller combines your voice and the chime in a virtual audio cable, which your call app uses as its microphone.
The browser controller can change status, but desktop audio routing requires the native application.

1. Open **Edit Config** on the desktop controller.
2. Open **Set up a virtual microphone**.
3. If you need a cable, use the setup button for your platform:
   - **Windows:** Install [VB-CABLE](https://vb-audio.com/Cable/) from the official page that opens.
   - **macOS:** Install [BlackHole 2ch](https://existential.audio/blackhole/) from the official page that opens.
   - **Linux:** Click **Create virtual microphone**. This uses PulseAudio or PipeWire with PulseAudio support.
4. Click **Refresh devices**.
5. Choose **CABLE Input**, **BlackHole 2ch**, or **Presence Light Cable** as the virtual cable output.
6. Choose your physical microphone under **Your microphone**. Leave it empty for chimes without your voice.
7. In your call app, select **CABLE Output**, **BlackHole 2ch**, or **Presence Light Microphone** as the microphone.
8. Keep your usual speakers or headphones selected in the call app.
9. Click **Test chime** and check the microphone meter in your call app.

Keep Presence Light running while you use this microphone. Allow microphone access if your operating system asks.
On Linux, the setup requires `pactl` and the ALSA PulseAudio plugin; Debian and Ubuntu provide `pulseaudio-utils` and `libasound2-plugins`.
Presence Light recreates its selected Linux cable when the controller starts.

**Mute microphone while Busy** silences only your voice in the configured cable. Chimes still play, including the test chime.
Keep the call app unmuted: its own mute button silences the complete microphone feed, including chimes.
If the cable is unavailable, the application reports an error and does not send the chime to your speakers.

## Development

Install dependencies:

```sh
pnpm install
```

Linux native builds also require the Tauri system dependencies and `libasound2-dev` for audio.

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

The tray menu works on Linux. Tauri does not support tray-icon clicks on Linux. Use the tray menu or shortcuts on Linux.
