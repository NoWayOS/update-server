# Contributing to the Update Server

Thank you for helping. The Update Server is the server NoWayOS devices check
in to for their updates. Every change to it is
judged by the same gates as NoWayOS's, written down in
[Defence Agent](https://github.com/NoWayOS/defence-agent); read its `GATES.md`
before your first pull request. This page is the short version.

## Before you start

- **Start from an issue** a maintainer has accepted (gate T1). Open one with
  the bug or feature template, or say on an existing one that you would like
  to take it.
- **A security problem is never an issue:** report it privately, as
  [SECURITY.md](SECURITY.md) says.
- Read the [README](README.md): how an update is secured, and the commands.

## Build and check

Everything runs in NoWayOS's dev container, so clone NoWayOS beside this
repository, at the commit `nowayos.rev` records, as the README's "Try it"
shows. Then:

```sh
make -C ../nowayos container   # the dev image, once
make test lint                 # while you work
make ci                        # before you open a pull request
```

`make ci` is the gates' build stage: a pull request must pass it.

## What a change keeps

- **The server is trusted for nothing a signature decides.** It never holds a
  key that signs a release, and never tells a device to skip a check: whether
  a release is installed is the device's decision alone.
- **The records are NoWayOS's.** The server reads and writes them with
  `nowayos-update` from the NoWayOS clone beside it, never with a copy of its
  own. Moving to another NoWayOS commit is a one-line change to `nowayos.rev`.
- **Everything from a device or a file is hostile until checked.** Inputs are
  bounded before they are read, decoders return typed errors and never panic,
  and every decoder has property tests.
- `unsafe` is forbidden; `unwrap`, `expect`, `panic!`, unchecked indexing and
  unchecked arithmetic are refused outside tests (the NoWayOS baseline, which
  `make deny` checks).
- Dependencies are MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause or ISC. Anything
  else needs a maintainer's written agreement first: ask in the issue.
- No secrets, keys or credentials anywhere, not even for tests: test keys are
  made for the run and deleted with it (`make bundles-check`).

## Pull requests

- Keep each one small and about one thing, with its tests.
- **Every commit carries a `Signed-off-by` line** (`git commit -s`): with it
  you certify the [Developer Certificate of Origin 1.1](https://developercertificate.org/),
  that you wrote the change or have the right to submit it under the
  project's licence. Gate P1 fails a commit without one.
- Fill in the pull-request template, AI assistance included.
- Your contribution is licensed as the project is, MIT OR Apache-2.0
  ([README](README.md#licence)).

## AI assistants

You may use one. Say so in the pull request, and which parts it wrote: you
answer for every line as if you had written it. [AGENTS.md](AGENTS.md) holds
the instructions for an assistant working in this repository.

## For maintainers

Until outside contributors join, the maintainers' own commits need no
`Signed-off-by` line; every contributor's commit does. An assistant never
signs off on anyone's behalf.
