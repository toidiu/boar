# boar

We measure congestion control algorithms by running `quiche-client` against `async_http3_server` (from the `deps/quiche` submodule) over a tc-shaped network, then writing stats and plots to a report.

## Build and run

- `git submodule update --init` first: `build.rs` runs `cargo build` inside `deps/quiche`, so the first build compiles BoringSSL and takes minutes.
- `cargo test`, `cargo build`.
- `sudo ./target/debug/boar`: tc/netem and the `ns_s1..ns_c2` netns chain (`scripts/virt_*.sh`) need root. Do not run it without asking.
- On macOS there is no tc; `scripts/test.sh` is the no-op network setup.
- The viewer is a separate bin behind the `boar-viewer` feature (`src/bin/viewer`).

## Architecture

A run goes: parse CLI into an `ExecutionPlan`, build the shaped network, start the server, run the client `count` times, parse metrics from logs, write a report. The viewer then loads reports in the browser.

```
/
├── CLAUDE.md              ← Repo-wide instructions
├── Cargo.toml             ← Crate: lib + `boar` bin + `viewer` bin (feature `boar-viewer`)
├── build.rs               ← Builds quiche-client and async_http3_server in deps/quiche
├── src/                   ← CLI, network setup, endpoints, stats, report writer, viewer app
├── viewer/                ← Trunk + Tailwind build config for the viewer (boar.toidiu.com)
├── scripts/               ← tc/netns scripts that build the shaped network
├── deps/                  ← cloudflare/quiche submodule: client and server binaries
├── report/                ← Run output, one folder per uuid (gitignored)
└── sample_report/         ← Demo reports for the viewer + scenarios.sh that made them
```

## Rust

- No free functions: every function belongs to an `impl` block on a struct or enum. Use a unit struct if there is no state.
- Exceptions: `main` and `#[test]` functions. Test helpers go on a test-only struct.
- A helper truly shared across the codebase goes in `src/util.rs`, still inside an `impl` block. Create `util.rs` only when such a helper exists.
- Error types live in `src/error.rs`, not in the module that returns them.

Existing code predates some of these rules (`args::parse`, `mod default`, `src/stats/error.rs`); follow the rules in new code and do not refactor old code unasked.

## Code comments

- Start with a one-line summary, then explain the decision, constraint, or failure it prevents.
- Assume an expert reader; never explain language mechanics, idioms, or type-system choices.

## Tests

- Keep each test self-contained.
- Give each test a doc comment: what holds (one line), what it sets up, the failure it catches.
- Comment each block in the body with what it does and why.

## Writing style

Applies to docs, comments, commit messages and PR descriptions.

- Use short sentences and plain words.
- Cut any sentence or phrase that adds no new information.
- Start with the point: no preamble or restated question.
- No filler, idioms, reaction labels (surprising, tricky), or metaphors that need explaining.
- Prefer a sentence over a list and a list over a table.
- Put code, commands, paths and identifiers in code spans or blocks.
- Write docs in first person plural ("we build only the proxy").
- Never use em-dashes; use colons.
