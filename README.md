# Presence Light

Presence Light is a two-state presence client for one Cloudflare Worker room. It includes a Tauri desktop overlay and a browser controller.

## Update an existing installation

Update Presence Light on each computer, including Viewer devices.
Replacing a release on GitHub does not update an installed application.
Version 1.1.1 reads existing configuration files. You do not need to delete them.

1. In the Presence Light tray menu, select **Quit**.
2. Download the new installer from the [v1.1 release](https://github.com/kaxonomy/presence-light/releases/tag/v1.1).
3. Install the update on the same computer.
4. Start Presence Light.
5. On a Viewer device, open **Configuration**. Make sure that **Device role** is **Viewer**.

Controller and Viewer chimes are off by default. The update keeps your saved Controller sound preference.
An old `soundEnabled` value applies only to the Controller. It does not enable the Viewer's chime.

On a Viewer device, **Configuration** shows **Play chime on this device** below the shortcuts.
This control saves automatically. When you turn it off, the local chime stops immediately and stays off after a restart.
The Controller's **Configure chime** dialog controls the sound sent through the virtual cable.

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
A virtual audio cable carries your voice and the chime to your call app.
Loopback sends your active microphone audio to this cable.
The default chime volume is 30%. Saved volume choices stay unchanged.

### Windows

Windows sends your voice to the cable through **Listen to this device**.
Presence Light sends the chime to the same cable.
After the Busy chime, Presence Light mutes the active microphone that you select.
The cable stays active, so it can still carry chimes.

1. Open **Configuration** from the tray.
2. Click **Configure chime**.
3. Open **Set up microphone loopback**.
4. If you do not have VB-CABLE, click **Get virtual audio cable**. Install [VB-CABLE](https://vb-audio.com/Cable/).
5. If the installer requests a restart, restart your computer.
6. Click **Refresh devices**.
7. Under **Virtual cable output**, select **CABLE Input**.
8. Under **Active microphone**, select the microphone that you use to speak.
9. Click **Open microphone controls**.
10. In **Recording**, select the same active microphone.
11. Click **Properties**.
12. Open **Listen**.
13. Select **Listen to this device**.
14. Under **Playback through this device**, select **CABLE Input**.
15. Click **Apply**.
16. In Discord, open **User Settings**, then **Voice & Video**.
17. Under **Input Device**, select **CABLE Output**.
18. Keep your usual speakers or headphones as the output device.
19. Join a call with your friends.
20. Click **Test chime**.
21. Ask your friends if they heard the chime.
22. Speak into your microphone. Ask your friends if they heard your voice.
23. Change your status to **Busy**.
24. After the chime, ask your friends if your voice is silent.
25. Change your status to **Available**. Make sure that your friends can hear your voice again.

For other call apps, select **CABLE Output** as the microphone.
Keep the microphone on in your call app. Its mute button also stops the chime.
Discord documents its input device controls in its [voice troubleshooting guide](https://support.discord.com/hc/en-us/articles/360045138471-Discord-Voice-and-Video-Troubleshooting-Guide).

### macOS and Linux

Presence Light sends your active microphone audio to the virtual cable automatically.
After the Busy chime, it mutes your voice in this cable.

1. Open **Configure chime**, then **Set up microphone loopback**.
2. On macOS, click **Get virtual audio cable**. Install [BlackHole 2ch](https://existential.audio/blackhole/).
3. On Linux, click **Create virtual microphone**.
4. Click **Refresh devices**.
5. Select **BlackHole 2ch** or **Presence Light Cable** as the virtual cable output.
6. Under **Active microphone**, select the microphone that you use to speak.
7. In your call app, select **BlackHole 2ch** or **Presence Light Microphone** as the microphone.
8. Keep your usual speakers or headphones as the output device.
9. Keep Presence Light open while you use this microphone.
10. Click **Test chime**. Ask your friends if they heard the chime and your voice.

If your operating system requests microphone access, allow it.
On Linux, setup requires `pactl` and the ALSA PulseAudio plugin. Debian and Ubuntu provide `pulseaudio-utils` and `libasound2-plugins`.
Presence Light recreates its selected Linux cable when the controller starts.

### Automatic mute

When you become Available, Presence Light restores the microphone that it muted.
On Windows, it restores the same device even if the default microphone changes.
A microphone that was already muted stays muted.
If the chime is off or cannot play, Presence Light still mutes the active microphone.
The test chime does not change your status or mute your microphone.
If the selected microphone is unavailable, reconnect it and click **Refresh devices**.
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
