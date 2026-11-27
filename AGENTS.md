# Instructions for AI agents

For any AI coding assistant working in this repository. Read
[CONTRIBUTING.md](CONTRIBUTING.md) too: these rules come from it and from the
gates in [Defence Agent](https://github.com/NoWayOS/defence-agent).

The Update Server is the server NoWayOS devices check in to for their
updates. It serves each release as the signed records a device checks for
itself, and it holds no key that signs a release.

## Building

- **Install nothing on the host:** no toolchains, no `rustup`, no
  `cargo install`, no package managers. Every `make` target runs in NoWayOS's
  dev container, built in the NoWayOS clone beside this one.
- The NoWayOS clone must be at the commit `nowayos.rev` records, with no local
  changes: `make nowayos-rev` checks it.
- Never run plain `cargo` on the host; the host loop is opt-in, for a person
  who already has Rust 1.93.1: `make test IN=host CARGO="cargo +stable"`.
- Before proposing a change: `make test`, `make lint`, then `make ci`.

## Rules the code keeps

- **The server decides nothing a signature decides.** It never holds an image
  key, never signs a manifest or trust list, and never asks a device to skip
  a check.
- **The records are NoWayOS's:** read and written with `nowayos-update` from
  the clone beside this one, never with a copy kept here.
- **Everything from a device or a file is hostile until checked:** bounded
  before it is read, decoded with typed errors, never a panic. Every decoder
  gets property tests.
- `unsafe` is forbidden; no `unwrap`, `expect`, `panic!`, unchecked indexing
  or unchecked arithmetic outside tests; every `#[allow]` gives a reason.
- No keys or secrets anywhere, not even for tests: test keys are made for the
  run and deleted with it.
- Dependencies are MIT, Apache-2.0, BSD or ISC only (`make deny`); ask before
  adding any.
- Every source file starts with `// SPDX-License-Identifier: MIT OR Apache-2.0`
  (`#` in scripts), with no copyright line.

## Pull requests

- Small, one change each, with its tests; fill in the template, saying which
  parts an assistant wrote.
- Never add a `Signed-off-by` line for anyone: a contributor signs off their
  own commits.
