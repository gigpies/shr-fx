# Prepared v2 C caller

Run `flock -xn /home/shome/p/.gigpies-build.lock scripts/check_c_api_v2.sh`
from the repository. It builds the offline
shared library and compiles/runs `caller.c` with C11 warning errors, then runs
the retained v1 and namespaced delay-v2 callers. Temporary executables are removed on exit.
The caller verifies actual wet samples against independently computed delay,
Room comb/allpass and fixed-depth chorus samples, plus ownership/status recovery.
Changed settings are compared with a freshly prepared reference, stale edits
preserve wet history, and bypass rejects fresh excitation while retaining tails.
The lifecycle loop covers all ten advertised algorithms, plus musical-time
conversion and refusal. Rust normal tests independently cover each reverb onset,
dark/bright tail variation, tape saturation/modulation, exciter harmonics and
alias suppression, deterministic reset/partitioning and allocation freedom.

The Architecture v2 contract and public header own settings/lifetime semantics.
No v2 settings file format, machine assignments or hardware admission is claimed.

The older `sample.c` and `corpus.json` are frozen evidence for the independently
developed source-timeline delay ABI before its symbols were separated. Use
[the current delay caller](../delay-v2/README.md) with a current library.
