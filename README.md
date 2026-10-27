![SHR FX](docs/assets/banner.svg)

# SHR FX

**A two-engine send-effects rack.** Combine reverb, delay, chorus and harmonic
exciter in up to eight parallel slots per engine. Your mixer carries the dry
signal; SHR FX supplies the wet returns.

<p>
  <img src="docs/screens/00-rack.png" width="48%" alt="SHR FX rack with four effect slots" />
  <img src="docs/screens/02-delay.png" width="48%" alt="Delay controls on the 40 by 13 screen" />
</p>

Actual offline renderer captures. [All screens](docs/screens/README.md).

[**Use the rack**](docs/OPERATING_GUIDE.md) · [**GigPies FX controls**](https://github.com/gigpies/shr-desk/blob/main/docs/USING.md#effects) · [Engineering](docs/ENGINEERING.md)

## Play it

Touch an effect to edit it. Use the pads for bypass, TAP for tempo, and saved
sounds for recall. A and B can use separate sends or share a source, with mono
and stereo returns.

The standalone rack opens offline by default. Live audio requires an existing
JACK server and explicit port assignments; MIDI input is optional.
**GigPies currently exposes a smaller stereo FX control set, not the full rack.**

DSP, routing, storage and controls have software validation. Live listening,
latency and target-device headroom remain to be qualified.

## Start offline

After installing the [build requirements](docs/OPERATING_GUIDE.md#build-and-open):

```sh
cargo build --release --locked
./target/release/shr-fx
```

[Operating guide](docs/OPERATING_GUIDE.md) · [MIT license](LICENSE)
