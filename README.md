# gwent-rs

[![CI](https://github.com/dcadea/gwent-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/dcadea/gwent-rs/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/dcadea/gwent-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/dcadea/gwent-rs)

A Rust implementation of the Gwent card game engine (from *The Witcher 3*). It
models the board, rows, cards, abilities, weather, and the best-of-three round
flow. Rust edition 2024.

## Commands

```sh
cargo build
cargo test                 # run all tests
cargo test <name>          # run a single test by name substring
just clippy                # cargo fmt + clippy (pedantic/nursery/unwrap_used, --all-targets)
just cov                   # cargo llvm-cov, opens the HTML coverage report
```

## Architecture

The engine is a containment hierarchy, each layer owning the one below:

```
Game ── owns ─▶ Board ── owns 2× ─▶ Side ── owns 3× ─▶ Row ── owns ─▶ Vec<Unit>
                                    (melee / ranged / siege)
```

- **`Game`** drives the best-of-three round loop and turn state, generic over a
  `Controller` that abstracts all player input.
- **`Board`** holds the two players' `Side`s and routes plays and cross-side
  effects (global scorch, weather).
- **`Side`** owns the melee / ranged / siege `Row`s.
- **`Row`** is the strength-calculation core: weather, tight bond, morale boost,
  and Commander's Horn are applied in a fixed order over a dirty-flag cache.

Card abilities (medic, spy, muster, scorch, decoy, summon, berserker, agile
placement, boosts) are threaded through an `Action` queue so one play can
trigger follow-ups. Rounds end when both players pass; the round loser drops a
gem and the game ends when a player runs out of gems.

The bulk of behavioral coverage lives in end-to-end tests in `game.rs`.
