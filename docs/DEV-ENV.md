# Local development: environment and builds (EN)

> Russian version: [DEV-ENV.ru.md](DEV-ENV.ru.md).
> Date: 2026-09-14. Machine: Windows, user without administrator rights.

## The main rule

**Debug builds only during development.** No `cargo build --release` until a release decision.
CI (`../.github/workflows/ci.yml`) is also switched to `cargo build --workspace` (debug),
artifact — `amc-launcher-windows-x64-debug`.

## Smart App Control blocks the fresh toolchain

This machine has Smart App Control in enforcement mode
(`HKLM\SYSTEM\...\CI\Policy\VerifiedAndReputablePolicyState = 1`,
CodeIntegrity events 3033/3077/3118 in the `Microsoft-Windows-CodeIntegrity/Operational` log).

What this means in practice (verified 2026-09-14):

- `stable` (rustc 1.98) **does not start at all**: `rustc.exe -vV` dies with `0xc0e90002`,
  the log shows the `rustc_driver-*.dll` load blocked ("did not meet the Enterprise
  signing level requirements", Policy ID `{0283ac0f-...}`).
- The SAC verdict is a **per-file-hash lottery**: the same source built by different
  rustc versions sometimes runs, sometimes is blocked (`os error 4551`);
  re-running the same file never changes the verdict. Rebuilding to the same hash does not help.
- Consequence: `cargo` under `stable` is fully dead; under old toolchains it works,
  but every freshly built exe/build-script is a separate lottery ticket.
  The full workspace **cannot be built locally**: the tree has 40+ build scripts
  (`ring`, `serde_json`, `khronos_api`, `icu_*_data`, ...), all must win at once.

## What works locally

Toolchain **`1.85.0`** (kept in `rustup toolchain list` next to `stable`):

```powershell
cargo +1.85.0 check -p amc-core   # ✅ type checking
cargo +1.85.0 build -p amc-core   # ✅ debug artifact target/debug/libamc_core.rlib
cargo +1.85.0 test -p amc-core    # ✅ 5 tests, green (verified 2026-09-14)
```

Why only `amc-core`: its build scripts already won the lottery and are cached in `target/`.
**Do not delete or clean `target/` without need** — there is no way to re-win the lottery.
Do NOT change `rust-toolchain.toml` (it pins `stable` for CI and everyone else) —
locally always use `+1.85.0`.

The other crates (`amc-auth`, `amc-downloader`, `amc-minecraft`, `amc-mods`, `amc-ui`)
and the whole workspace are checked/built **only in CI** (GitHub Actions, Windows).
Local `cargo +1.85.0 check --workspace` hits dependency MSRV
(`icu_* 2.3.0`, `image 0.25.10` require rustc ≥1.88) — downgrading dependencies
for one local machine is **forbidden**, this is a shared codebase.

## Full debug builds and tests — in CI

A push to `main` triggers the "CI" workflow: `cargo check --workspace`,
`cargo test --workspace`, `cargo build --workspace` (all debug).
The ready `amc-launcher.exe` (debug) is in the run's Artifacts (`amc-launcher-windows-x64-debug`).

## Extended logging (already built in)

`crates/amc-core/src/logging.rs`: default filter `info,amc_*=debug`,
output to stdout **and** to `logs/launcher.log` in the launcher data directory.
Run with maximum detail:

```powershell
$env:RUST_LOG = "debug"   # or "trace" for everything
.\amc-launcher.exe
```

Attach `launcher.log` to debugging sessions — it shows what and where to change.

## How to fix permanently (user action, admin rights needed)

Smart App Control is turned off by the user, and cannot be turned back on without reinstalling:

1. "Windows Security" → "App & browser control" →
   "Smart App Control settings" → **Off** (UAC will ask for an administrator password).
2. After turning it off the `stable` toolchain should work — verify with `rustc --version`
   and `cargo check --workspace`.

Without administrator rights there are no other options (WSL, WDAC allowlist, policy change) —
verified: no admin rights (`BUILTIN\Administrators` is deny-only), WSL is not installed.
