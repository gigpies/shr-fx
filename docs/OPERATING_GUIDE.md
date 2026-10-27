# Use the effects rack

[Project home](../README.md) · [Screens](screens/README.md) · [Engineering](ENGINEERING.md)

This is the standalone two-engine rack. GigPies currently offers a smaller
[integrated stereo FX surface](https://github.com/gigpies/shr-desk/blob/main/docs/USING.md#effects).

## Build and open

Use the checkout's pinned Rust toolchain. Linux also needs a C compiler,
`pkg-config`, JACK and ALSA development headers (Debian: `build-essential
pkg-config libjack-jackd2-dev libasound2-dev`).

```sh
cargo build --release --locked
./target/release/shr-fx
```

Startup is offline. The 40×13 interface, routing drafts and saved sounds work
without audio. For live use, start your JACK server separately and run:

```sh
./target/release/shr-fx --audio --midi
```

Omit `--midi` for audio only. Configure exact audio ports and controller input
in Menu. The rack does not connect guessed ports or start JACK for you.

## Set up a sound

1. Choose engine A or B and open a slot.
2. Select the effect and adjust its controls. Empty slots remain empty.
3. Under **Menu → Routing**, select the source and mono/stereo return layout.
4. Assign physical connections under **Ports**, then bring up the wet return.
5. Save under **Sounds**; replacing an occupied sound requires Replace.

For a starting MultiFX setup, use **Configure → Mode → + → Apply** for Delay,
Room, Chorus and Exciter. Slots run in parallel, not in series.

## Controls

| Action | Control |
|---|---|
| Browse / open or confirm | Rotary 1 / click 1 |
| Adjust / back or cancel | Rotary 9 / click 9 |
| Bypass a slot without changing selection | Its pad, 1–8 |
| Edit an effect | Touch its card |
| Set tempo | TAP or manual BPM |

Only rotaries 1 and 9 have click actions. Absolute controls may show a pickup
direction; turn toward the current value before they take over.

Keyboard: Tab/Up/Down to focus, Enter to activate, Left/Right to adjust,
`a`/`b` to choose an engine, `g` for Menu, `t` for TAP, `x` for slot bypass,
`w` for engine wet bypass, `m` for mute/resume, Space for PANIC and `q` to exit.

## Bypass, mute and routing

Bypass stops new input to the effect and lets the tail drain. Mute clears the
selected engine; PANIC clears and mutes both. Resume releases a mute.
These outputs are wet-only; keep the dry signal at your mixer.

A and B can have separate mono/stereo returns or share a stereo pair. Shared
returns sum, so watch the receiving mixer as well as each engine's meters.
Mono folding can cancel opposite-polarity content.
[Routing options and exact headroom](reference/OPERATING_GUIDE.md#routing-and-headroom).

## Save and recover

Menu drafts use Apply/Cancel. If an edited value also changed live, **Keep live**
retains the sounding value for review. Saved sounds live under your private data
root; physical connections and controller settings are separate.

If audio is unavailable, check Ports and use **Retry audio**. Do not start a
second JACK server. [Storage, timing and shutdown reference](reference/OPERATING_GUIDE.md).
