# SHR FX: plan and implementation

SHR FX owns wet DSP and prepared embedding. GigPies owns source-clock transport,
dry/wet composition and final integration; Desk owns operator controls.

The independently integrated source-timeline delay uses `shr_fx_delay_v2_*`;
see [its contract](ARCHITECTURE.md#prepared-stereo-wet-delay-v2). It coexists with
the published palette v2 ABI under distinct symbols and types.

## Delivered contract and evidence

FX-01 exposes fixed stereo f64 delay and read-only capabilities/status. The actual
library is consumed by GigPies; [acceptance](ACCEPTANCE.md) preserves its exact
scope. The two-engine/eight-slot standalone rack does not establish writable
embedding. [Header](../include/shr_fx.h) and [architecture](ARCHITECTURE.md) are authoritative.

## FX-02 — prepared writable embedding

State: IMPLEMENTED (offline); target acceptance pending. Owner: SHR FX. Outcome: a small versioned f64 wet-effects
interface that GigPies can prepare, control and render without owning FX DSP.
Progress/evidence: native f64 prepared embedding delivered; the owner-authorized
2026-10-29 expansion adds Small Room, Chamber, Plate, Hall, tape echo and
Warm/Bright exciter to Digital Delay, Room and chorus. Musical-time conversion
adds straight/dotted/triplet notes from whole through 1/64 within the explicit
2-second delay capacity. See [Acceptance](ACCEPTANCE.md) and the v2 contract.
Next: separately authorized target-specific aggregate budget and live acceptance;
GigPies/Desk consumer adapters remain owned by those projects.

2026-10-29 initial provider audit (before the subsequently authorized palette
expansion): implementation verified against actual native f64 DSP,
versioned header and synthetic Rust/C wet-output checks; no DSP or ABI-layout
change required. [Consumer requirements](ARCHITECTURE.md#gp-fx-consumer-requirements)
clarify units, immediate bypass, replacement fades, retry and serialized retirement.
[Dated evidence and exact artifact identity](ACCEPTANCE.md#2026-10-29--gp-fx-provider-readiness)
supersede neither historical hardware limits nor the separate aggregate budget gate.
Host/Desk wiring was explicitly deferred during the provider audit because REC
owned GigPies and SHR Desk; both were read-only in that parallel phase. The owner
subsequently released them and authorized GP-FX integration: four ordinary stereo
FX aux buses with independent wet returns, not the standalone A/B slot rack.
Current consumer work is owned by the existing shared GP-FX card: four stereo
aux sends travel to Brain's four palette instances and return wet to Main;
monitor buses remain independent. The explicit local-host variant is retained.

### Scope and reuse

The standalone application already owns two engines, eight parallel slots per
engine, Delay/Reverb/Chorus/Exciter, controller UI and rack persistence. Preserve
that implementation and its f32 behavior. Embedded v1 exposes only fixed f64
delay; standalone features are not evidence that writable embedding exists.

1. Expose a writable stereo digital delay using the existing shared delay core.
2. Extend shared primitives to f64 where required for one existing room/chamber
   reverb and the existing chorus. Preserve standalone f32 regressions and
   independently verify the new precision path; casting f32 output is insufficient.
3. Reuse existing reverb/tape variants and specialize the shared exciter to f64,
   with explicit units, wet semantics and bounded memory/work. One accepted effect can be integrated
   without waiting for the whole palette or standalone rack exposure.

Start with one effect per independently owned instance. Keep instance limits
explicit in capabilities and host admission. If embedded parallel slots become
necessary, reuse the established bounded parallel model and specify aggregate
headroom/cost first. This plan does not introduce series/mixed chains, replace the
A/B rack, or rebuild its UI. SHR-DAW adoption is a separate consumer task.

GigPies owns sends, return levels, dry/wet summation, source clocks, transport and
final composition. SHR PA remains downstream speaker processing/protection;
it does not host FX under this plan. Desk controls the provider through GigPies.
No FX-owned devices, MIDI ports, transport workers or lighting-parameter interface.

### Contract checklist — before new ABI implementation

Record the reviewed contract in [Architecture](ARCHITECTURE.md) and the versioned
[header](../include/shr_fx.h). Keep this card as the execution/acceptance owner.

- [x] Define stable algorithm IDs and one typed parameter representation: units,
  finite ranges, defaults, sample-rate-dependent bounds and invalid-value policy.
  Separate algorithm identity from settings; reject unsupported/malformed edits
  without silently changing another parameter or the sounding state.
- [x] Define interleaved stereo `double` buffers and native f64 state/coefficients,
  frame capacity, rates, amplitude/headroom bounds, aliasing/in-place rules,
  zero-frame behavior and failure output. Finite values above unity are not
  automatically faults; document the actual supported bound.
- [x] Specify version/size checks, fixed-width C fields, errors and symbol names.
  Preserve every existing v1 signature, layout and sample/error behavior; v2 is
  additive, not a reinterpretation of v1 handles or structs. Contain Rust panics
  at the ABI boundary and state allocation-failure behavior.
- [x] Define create/prepare, validate, cancel, publish, process, reset, retire and
  destroy ownership. Preparation allocates off-thread. Publication transfers a
  complete validated state at a processing boundary. A rejected/stale edit leaves
  active state intact; an accepted edit is distinct from an applied transition.
- [x] Give every active instance one processing owner. Control work must not
  mutate its DSP concurrently. Specify ownership on every success/error path,
  bounded pending/retired capacity, backpressure, cancellation and safe shutdown.
  Free retired state off-thread only after processing relinquishes it; keep the
  library loaded until its objects are destroyed. Do not pass mutable chains or
  unbounded maps into process. No callback allocation/free, locks, I/O or waiting.
- [x] Define continuous smoothing and delay-time changes; specify structural
  replacement and bounded old/new overlap. When overlap exceeds admission limits,
  use deliberate fade-out/swap/in. Include transient memory and CPU in admission.
- [x] Define excitation-off wet bypass with tails, explicit mute/panic/reset,
  independent-instance faults, and tail policy for effect replacement. No dry
  passthrough. Define source-gap/epoch and rate-change handling with the host;
  never report a draining tail as measured silence without evidence.
- [x] Define free-time and tempo-derived settings without adding an FX clock
  owner. GigPies supplies tempo/control updates; source samples advance DSP.
  Report intentional effect delay separately from buffering/transport/device delay.

### Delivery and acceptance

- [x] Deliver writable delay plus a real C caller: prepare, apply, render, query,
  cancel/refuse, reset, retire and destroy. Verify impulse onset, channel isolation,
  feedback/decay, parameter transitions and block-partition behavior against
  independent expected samples; retain exact fixed-v1 regressions.
- [x] Deliver the selected reverb and chorus through the same interface, with
  appropriate independent impulse/decay/modulation checks, deterministic seeds,
  finite bounds and unchanged standalone behavior. Reuse DSP, not UI/device code.
- [x] Cover malformed parameters, unsupported versions, permitted pointer shapes,
  capacity errors, non-finite samples, failed preparation, stale edits, exhausted
  queues, source discontinuities and shutdown with pending/retired states.
  Test actual wet output and preserved healthy instances, not acknowledgements alone.
- [x] Instrument allocation/deallocation freedom for processing, publication,
  transitions and reset. Check preparation memory, maximum work and bounded
  reclamation; preserve standalone storage/controller/routing regressions.
- [ ] Establish a target-specific aggregate budget before claiming live readiness.
  At 48 kHz/48 frames the period is 1 ms; the FX share must leave room for other
  scheduled work and jitter. Record hardware/build, admitted instances/effects,
  worst-case parameters/transitions, memory and p50/p95/p99/max processing cost.
  Do not substitute a per-effect guess or test-count target for this budget.
- [x] Keep normal correctness/recovery tests distinct from opt-in cost matrices.
  Authorized on-device release testing separately measures deadlines, xruns,
  thermal behavior and dry-plus-wet listening. Offline timing does not prove them.

## FX-03 — owner control/status artifacts

State: IMPLEMENTED (offline); reviewed FX-02 contract delivered. Owner: SHR FX.
Outcome: versioned capabilities, parameter descriptors, validated edits and
correlated current/target/fault readback. Progress/evidence: real C caller and
normal recovery/output tests; v1 retained separately (see Acceptance).
Consumer integration and its software acceptance belong to shared GP-FX.

The subsequent owner-authorized palette expansion retains this lifecycle and
adds capability bits/descriptors for the new effects plus an optional musical-time
helper. Actual wet-output and lifecycle checks cover all advertised IDs; see
[final expansion evidence](ACCEPTANCE.md#2026-10-29--authorized-embedded-palette-expansion).
No provider blocker remains for these effects' offline consumer integration.
Target aggregate admission remains separate. GP-FX uses four ordinary stereo auxes;
the standalone A/B slot rack is not the requested mixer interface.

2026-10-29 audit: capabilities/descriptors and accepted/current/applied/refused
readback verified with actual C wet output, including changed gain, stale-history
preservation and excitation-off tails. No provider blocker remains for bounded
offline GP-FX integration. Consumer work was deferred under REC's sibling ownership;
the subsequent ownership release moves integration to the existing shared GP-FX card.

GP-FX consumer contract order (the existing shared card owns implementation and evidence):
1. Bind the complete palette symbol set and version/size layouts independently
   of the existing fixed-v1 and GP21 delay-v2 adapters; retain their fallback.
2. Define host instance/send/return admission and source epoch/rate ownership;
   bound preparations and retired-token queues, with off-thread destruction.
3. Publish validated copied settings at source block boundaries with request/base
   correlation; distinguish acceptance, swap, fade completion and interruption.
   Reclaim tokens before another edit; preserve failure ownership and backpressure.
4. Expose only advertised units/joint bounds in Desk; keep target/current/fault
   visible. Host owns persistent mute/rearm, reset on discontinuity and shutdown.
5. Test actual provider wet samples through host/Desk edits, stale/refused edits,
   fallback, reconnect, tails, reset and quiesced teardown before separate target
   aggregate performance and live/listening acceptance.

- [x] Advertise only supported algorithms/parameters, units/ranges, precision,
  rate/block/instance limits and transition resources. Distinguish unavailable
  features from values legitimately equal to zero.
- [x] Correlate requests with accepted/applied/rejected state and active revision.
  Define stale-base refusal, duplicate/retry behavior and bounded history.
  Report target versus current settings and transition/fault state without
  inventing hardware health. Queries must respect instance ownership.
- [x] Keep machine/device assignments out of effect settings. If serialized
  settings are needed, define one strict versioned representation and round-trip
  fixtures; do not reuse standalone rack version numbers for a different format.
  File loading/saving remains control-thread work, outside the processing ABI.
- [x] Verify capabilities and readback against actual wet output, including failed
  edits, interrupted transitions and recovery; use a real C caller. Unsupported
  changes must preserve the active instance and produce actionable error status.

Shared integration belongs to [GP-FX](https://github.com/gigpies/gigpies/blob/main/docs/development/MODULE_IMPLEMENTATION_PLAN.md#gp-fx--writable-owner-effects-in-the-console).
Provide reviewed artifacts and evidence there; GigPies and Desk own their adapters.
Neither module implementation nor this plan authorizes sibling edits or hardware use.

## Palette scope and later consumer exposure

The owner selected reverb variety, tape echo and exciter after the initial
provider-only readiness audit. They use the same prepared lifecycle, with new
algorithm IDs and no changes to the original settings/status layouts. Dark and
bright reverb variation uses damping, not an extra algorithm bank. Native f64
exciter shares the standalone oversampled residual implementation. A pure helper
converts host BPM and note divisions; unsupported long times refuse explicitly.
Full standalone A/B/eight-slot composition is still not exposed by this
single-instance ABI. GigPies/Desk now map this palette into four stereo auxes
through Brain, with bounded instance admission and no copied DSP; the shared
GP-FX card owns the actual transport and Desk evidence. Existing
standalone MultiTap/Diffused delay, ping-pong and ensemble options remain separate
from this bounded palette increment; exposing every rack option will require
additional owner descriptors and tests, not merely relabelled Desk controls.

Host/Desk wiring was deferred while REC owned those repositories; the subsequent
release authorizes the four-aux consumer integration in the shared GP-FX card. No further
palette or arbitrary series-chain UI is authorized by this implementation.
Target performance, dry-plus-wet listening and phase assessment (especially
exciter) are separate gates; Hall availability is not a live admission claim.

## Tracking and verification

This is the owning plan for module-only GigPies work. Keep each new task's plan,
implementation state, checklist, evidence and next action together here. Shared
integration tasks live only in the [GigPies integration plan](https://github.com/gigpies/gigpies/blob/main/docs/development/MODULE_IMPLEMENTATION_PLAN.md);
link to their cards instead of copying status. Follow its task lifecycle and this
repository's AGENTS.md. STATUS/acceptance files hold dated evidence, not another queue.
Preserve closed milestone details in the linked archive; do not reopen old launch cards.
A documentation reconciliation does not rerun tests or qualify hardware.

[Historical plan and milestone evidence](archive/tracking-before-2026-10-09/GIGPIES_IMPLEMENTATION.md). Its launch instructions are retired.
