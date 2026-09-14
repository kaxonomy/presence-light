# Presence Light

Presence Light is a two-state presence client for one Cloudflare Worker room. It includes a Tauri desktop overlay and a browser controller.

## Configuration

Open the controller page. Then enter the server address (WebSocket URL) and the controller token.

The browser stores the configuration in local storage. The token does not enter the Vite build or the source code.

Choose **Controller** only for a device that can change the status. The Worker must enforce authorization.

Select **Start when I sign in** to register the desktop application for startup. An automatic start keeps the dot hidden.

After setup, open **Configuration** from the tray to change the connection and indicator appearance.
Click **Configure chime** to open the chime dialog. Changes save automatically.
The main page fits the default 760 × 540 window. Both the page and dialog can scroll if the content does not fit.

Both clients add the token as the `token` query parameter. The Worker must accept this token format.

Use `ws://127.0.0.1:8787/...` for a local Worker. Use `wss://...` for production.

For browser development, `VITE_PRESENCE_WS_URL` can supply the initial URL. The user can replace this URL in the controller page.

## Chime through your microphone

Only the desktop controller sends the Busy chime. Viewer clients stay silent.
The controller combines your voice and the chime in a virtual audio cable, which your call app uses as its microphone.
The browser controller can change status, but desktop audio routing requires the native application.

1. Open **Configuration** on the desktop controller.
2. Click **Configure chime**.
3. Open **Set up a virtual microphone**.
4. If you do not have a cable, use the setup button for your platform:
   - **Windows:** Install [VB-CABLE](https://vb-audio.com/Cable/) from the official page that opens.
   - **macOS:** Install [BlackHole 2ch](https://existential.audio/blackhole/) from the official page that opens.
   - **Linux:** Click **Create virtual microphone**. This uses PulseAudio or PipeWire with PulseAudio support.
5. Click **Refresh devices**.
6. Choose **CABLE Input**, **BlackHole 2ch**, or **Presence Light Cable** as the virtual cable output.
7. Choose your physical microphone under **Your microphone**. Select **Chime only (no voice)** for chimes without your voice.
8. In your call app, select **CABLE Output**, **BlackHole 2ch**, or **Presence Light Microphone** as the microphone.
9. Keep your usual speakers or headphones selected in the call app.
10. Click **Test chime**. Make sure that the microphone meter in your call app shows activity.

Keep Presence Light open while you use this microphone. If your operating system requests microphone access, allow it.
On Linux, the setup requires `pactl` and the ALSA PulseAudio plugin. Debian and Ubuntu provide `pulseaudio-utils` and `libasound2-plugins`.
Presence Light recreates its selected Linux cable when the controller starts.

When you become Busy, Presence Light plays the chime, then mutes your microphone.
When you become Available, Presence Light restores your microphone.
If the chime is off or cannot play, Presence Light still mutes your microphone.
The mute control has no separate configuration. Old configuration files remain compatible.
With a virtual cable, Presence Light mutes only your voice. The test chime still plays.
Keep the microphone on in your call app. Its mute button also stops the chime.
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
