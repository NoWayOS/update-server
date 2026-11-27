# Security policy

The Update Server is the server NoWayOS devices check in to for their
updates. If you find a security flaw in it, please tell us privately, so that
we can fix it before anyone else can use it.

## Reporting a vulnerability

**Please do not open a public issue, pull request or discussion.** Report it
privately, either:

- through GitHub's private vulnerability reporting: **Report a vulnerability**
  on this repository's **Security** tab; or
- by email to **security@nowayos.com**.

Please tell us what is affected (crate, command, commit or release), how to
reproduce it — ideally with the smallest bundle or request that shows it — and
what an attacker could do with it. Use synthetic records and throwaway keys
only: never send a real device's records, a fleet's data or any private key.

## Scope

In scope: the code in this repository — the server, its crates and its build
tooling — on `main` and in each supported release. That includes a flaw that
lets the server send a device something other than the release it was given,
answer a device it should not, or leak what it knows about a fleet.

Out of scope:

- **A device installing what it should not.** That is the device's own check,
  in NoWayOS, which has its own security policy: report it there.
- **Third-party code**, including our Rust dependencies. Please report those
  to their maintainers — and tell us too, if the server's use of them is
  affected.

## What we aim to do

- To acknowledge your report within **5 working days**.
- To give you our assessment — whether we consider it a vulnerability, and how
  severe — within **15 working days**.
- To keep you informed at least every **30 days** until it is resolved.
- To agree a disclosure date with you: by default when the fix is released, and
  **no later than 90 days** after your report unless we both agree otherwise.
- To credit you in the advisory, by the name you choose, unless you would
  rather not be credited.

## Supported versions

Until the first release, only `main` is supported. Once releases begin, each
release states how long it receives security fixes.

## How fixes are announced

Each fixed vulnerability gets a security advisory where this repository is
hosted, and a line in the release notes' security changes. If a crate from this
repository is published on crates.io, the advisory also goes to the RustSec
advisory database, which `cargo audit` and `cargo deny` read. An advisory says
what is affected, how severe it is, the fixed version, any workaround, and who
reported it.
