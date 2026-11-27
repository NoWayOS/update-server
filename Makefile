# SPDX-License-Identifier: MIT OR Apache-2.0
# The Update Server: developer commands. `make` (or `make help`) lists them.
#
# Every target runs in the NoWayOS dev container by default, through the
# shared helper in the sibling nowayos clone (nowayos/tools/make/container.mk),
# so no host Rust is needed. Build the image once with `make container` in
# nowayos/. The server builds against ../nowayos at the commit nowayos.rev
# records, so it reads and writes the records devices decode with the
# devices' own code: `make ci` checks that first.
#
# The everyday targets - fmt, lint, test, doc, run and clean - can run on the
# host instead with IN=host, using the toolchain pinned in rust-toolchain.toml
# (Rust 1.93.1), or the host's stable toolchain where that already is 1.93.1:
# `make test IN=host CARGO="cargo +stable"`. Nothing here installs a
# toolchain: RUSTUP_AUTO_INSTALL=0 makes rustup report a missing toolchain
# instead of downloading it.
#
# Written for GNU Make 3.81 (the version macOS ships) and later.

CARGO ?= cargo
# Never let a cargo, rustc or rustup call made from here install anything.
RUSTUP_AUTO_INSTALL ?= 0
export RUSTUP_AUTO_INSTALL
# `make ci` sets --locked so the committed Cargo.lock is exactly what is checked.
LOCKED ?=
# make sbom writes a CycloneDX SBOM here.
SBOM_DIR := build/sbom
# make bundles-check writes NoWayOS's update-test bundles here.
BUNDLES_DIR := build/update-bundles

# Shared dev-container helper: IN, ENGINE, IMAGE, WS, IN_CONTAINER. Optional,
# so a missing nowayos clone never breaks the host targets (with IN=host).
-include ../nowayos/tools/make/container.mk

.DEFAULT_GOAL := help
.PHONY: help nowayos-rev fmt lint test doc run bundles-check deny sbom secrets ci ci-local clean

help:
	@echo "Update Server make targets (in the dev container; IN=host runs the everyday ones on the host)"
	@echo "  make nowayos-rev    check ../nowayos is at the commit nowayos.rev records, with no local changes"
	@echo "  make fmt            format the code (cargo fmt)"
	@echo "  make lint           cargo fmt --check, then clippy on all targets with warnings denied"
	@echo "  make test           unit, property and command-line tests"
	@echo "  make doc            build the API docs, warnings as errors"
	@echo "  make run ARGS=\"check FILE.bundle\"   run the update-server command (ARGS: its command line)"
	@echo "  make bundles-check  NoWayOS's update-test bundles (real keys, made and deleted with the run) checked as the server checks them (dev container only)"
	@echo "  make deny           the NoWayOS baseline, then licence, source and advisory checks with cargo-deny (dev container only)"
	@echo "  make sbom           a CycloneDX SBOM: $(SBOM_DIR)/update-server.cdx.json (dev container only)"
	@echo "  make secrets        scan the tree and every commit for keys and credentials, offline (dev container only)"
	@echo "  make ci             nowayos-rev, secrets, lint, test, doc, bundles-check, deny and sbom in the dev container, with --locked"
	@echo "  make clean          remove build output (the container's, or the host's with IN=host)"
	@echo "Current settings: IN=\"$(IN)\" CARGO=\"$(CARGO)\" ENGINE=\"$(ENGINE)\" IMAGE=\"$(IMAGE)\""

# Read-only: git takes no lock. Run on the host or in the container alike.
nowayos-rev:
	@sh tools/check-nowayos-rev ci

# The everyday targets run here inside the dev container or with IN=host;
# otherwise they start the container.
ifneq ($(NOWAYOS_CONTAINER)$(filter host,$(IN)),)
fmt:
	$(CARGO) fmt --all

lint:
	@sh tools/check-nowayos-rev build
	$(CARGO) fmt --all -- --check
	$(CARGO) clippy --workspace --all-targets $(LOCKED) -- -D warnings

test:
	@sh tools/check-nowayos-rev build
	$(CARGO) test --workspace $(LOCKED)

doc:
	@sh tools/check-nowayos-rev build
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --workspace --no-deps $(LOCKED)

# The update-server command itself; paths in ARGS are relative to this folder.
run:
	@$(CARGO) run $(LOCKED) --quiet -p update-server -- $(ARGS)

clean:
	$(CARGO) clean
else ifdef IN_CONTAINER
fmt lint test doc clean:
	$(IN_CONTAINER) make $@ CARGO=cargo LOCKED="$(LOCKED)"
run:
	@$(IN_CONTAINER) make --no-print-directory run CARGO=cargo LOCKED="$(LOCKED)" ARGS='$(ARGS)'
else
fmt lint test doc run clean:
	@echo "error: make $@ needs ../nowayos/tools/make/container.mk - clone nowayos beside update-server - or IN=host to run on the host" >&2
	@exit 1
endif

ci-local: override LOCKED := --locked
ci-local: nowayos-rev secrets lint test doc bundles-check deny sbom

ifeq ($(NOWAYOS_CONTAINER),1)
# NoWayOS's update-test bundles - throwaway keys made and deleted with the
# run, real ECDSA signatures - each checked as the server checks a bundle
# before serving it (tests/nowayos_bundles.rs says which it refuses).
bundles-check:
	@sh tools/check-nowayos-rev build
	@mkdir -p build && { test -f build/.gitignore || printf '*\n' > build/.gitignore; }
	@rm -rf $(BUNDLES_DIR)
	python3 ../nowayos/tools/update-bundles --out $(BUNDLES_DIR)
	UPDATE_SERVER_BUNDLES=$(CURDIR)/$(BUNDLES_DIR) $(CARGO) test $(LOCKED) -p update-server --test nowayos_bundles -- --ignored

# The NoWayOS baseline (lints, profiles, deny settings) first, then cargo-deny.
deny:
	python3 ../nowayos/tools/check-baseline .
	$(CARGO) deny $(LOCKED) check

# A CycloneDX SBOM, by NoWayOS's generator (../nowayos/tools/sbom), with the
# rule every repo uses. The server runs on an operator's host, never on a
# device: what it is built from is `required`; dev-dependencies, build-time
# tools and other platforms' dependencies are `excluded`, with the reason.
sbom:
	@mkdir -p $(SBOM_DIR) && { test -f build/.gitignore || printf '*\n' > build/.gitignore; }
	python3 ../nowayos/tools/sbom --workspace . --name update-server --kind application \
		--out $(SBOM_DIR)/update-server.cdx.json

# The secrets scan, by NoWayOS's scanner (../nowayos/tools/secrets-scan): the
# tree, and every version of every file and every commit message on every
# ref, offline. A finding names its file and line, never the text; reviewed
# false positives go in secrets-allow.toml, one exact entry each with its
# reason. It needs the whole history: a shallow clone fails.
secrets:
	python3 ../nowayos/tools/secrets-scan --repo . --allow secrets-allow.toml
ci: ci-local
else ifdef IN_CONTAINER
# On the host: run them in the dev container (the secrets scan with no network).
secrets: override CONTAINER_RUN_ARGS += --network=none
bundles-check deny sbom secrets:
	$(IN_CONTAINER) make $@ CARGO=cargo LOCKED=$(LOCKED)
ci:
	$(IN_CONTAINER) make ci-local CARGO=cargo
else
bundles-check deny sbom secrets ci:
	@echo "error: make $@ needs ../nowayos/tools/make/container.mk - clone nowayos beside update-server" >&2
	@exit 1
endif
