# NoWayOS Update Server

**Updates a device checks for itself.**

> Part of NoWayOS: no way to exploit what was never built in.

The Update Server is the server that devices built on
[NoWayOS](https://github.com/NoWayOS/nowayos), the Single Purpose OS, check in
to for their updates. It serves each release as the signed records the device
checks byte by byte, and it holds no key that signs a release. Whoever runs it,
or breaks into it, can hold a release back, but can never install one of
their own.

NoWayOS is in development, and so is this: it is for learning and building,
not yet for production.

## How an update is secured

A NoWayOS device takes an update as one whole system image. Its update PD
checks the image against the trust anchors built into the running image, and
against its rollback index. It writes the image to the slot it is not running
and boots it once on trial. The update is kept only if the new system proves
healthy; otherwise the device falls back to the slot it was running before.
That part runs today in QEMU, in NoWayOS.

The Update Server is the other half:

- **What may run is signed offline**, by the product's image key, in the
  release pipeline. The server never holds that key.
- **When, and on which device,** will be signed online by the server's
  deployment key, for a product whose trust list asks for one. The device then
  needs both signatures, so no single key or server installs anything.
- **Fresh without a clock.** Each answer will echo the nonce the device sent
  in its check-in, so a device can tell when it is being held back, with no
  trusted time.
- **Every device runs the same checks.** The server reads and writes the
  device's own record formats with the device's own code (NoWayOS's
  `nowayos-update`), at the NoWayOS commit `nowayos.rev` records.

**Built so far:** checking a release's bundle whole before any of it is served:
every record decodes, the parts come in order with no gap, and the image has
its manifest's length and SHA-256. **Next:** answering a device's check-in over
TLS, then rolling a release out to groups of devices in stages.

## Try it

The Update Server builds against NoWayOS, at the commit `nowayos.rev` records.
The dev container mounts the folder above the clones, so give the two a folder
of their own:

```sh
mkdir nowayos-work && cd nowayos-work
git clone https://github.com/NoWayOS/nowayos.git
git clone https://github.com/NoWayOS/update-server.git
git -C nowayos checkout "$(cat update-server/nowayos.rev)"   # the NoWayOS commit the server is tested against
cd update-server
make -C ../nowayos container   # the dev image, once
make test                      # unit, property and command-line tests
make bundles-check             # NoWayOS's update-test bundles, with real keys and signatures, checked as the server checks them
make ci                        # everything a pull request must pass
```

`make` lists every target. Everything runs in the dev container: nothing is
installed on your machine.

`make run ARGS="check FILE.bundle"` checks a bundle file as the server does,
and prints what release it holds or why it is refused.

## Licence

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT licence](LICENSE-MIT), at your option. Every crate declares its licence,
and every source file carries an SPDX identifier.

Contributions are licensed the same way: see [CONTRIBUTING.md](CONTRIBUTING.md).
Security problems: [SECURITY.md](SECURITY.md).

NoWayOS is sponsored by Adappt.ai.
