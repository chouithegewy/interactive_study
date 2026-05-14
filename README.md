# Interactive Study

A fast, keyboard-driven terminal study app for the CS4470 final review.

This project turns the review deck and HW3 solution PDF into a focused study
TUI with flashcards, quiz mode, topic filters, and review tracking. It is built
in Rust with no external crates, so it stays small, portable, and easy to run.

## What It Covers

The card set is built from:

- `review_fin (1).ppt`
- `hw3 sol_ CS 4470-01 (35218).pdf`

Topics include:

- Network-layer routing
- RIP, OSPF, and BGP
- ICMP, NAT, and DHCP
- UDP services, headers, and checksum logic
- Reliable transfer, ARQ, stop-and-wait, and Go-Back-N
- TCP headers, reliability, flow control, connection management, and congestion
  control
- HTTP, FTP, and email protocols
- HW3 TCP practice problems for windows, sequence numbers, ACKs, RTT, RTO, and
  retransmissions

## Features

- Card mode for quick review
- Quiz mode with answer reveal
- Outline mode for scanning the full study set
- Section filters for targeted practice
- Reviewed/open marking during a session
- Arrow keys plus `j/k` navigation
- 80-column friendly rendering for laptop terminals
- No network, database, or dependency setup

## Quick Start

```bash
cargo run
```

For an optimized binary:

```bash
cargo build --release
./target/release/interactive_study
```

## Controls

| Key | Action |
| --- | --- |
| `j`, `n`, `Enter`, `Right`, `Down` | Next card |
| `k`, `p`, `Left`, `Up` | Previous card |
| `Space` | Reveal or hide answer |
| `m` | Mark current card reviewed/open |
| `c` | Card mode |
| `z` | Quiz mode |
| `o` | Outline mode |
| `0` | Show all sections |
| `1` | Network Layer |
| `2` | Transport Utilities |
| `3` | UDP |
| `4` | Reliable Transfer |
| `5` | TCP |
| `6` | HTTP/FTP |
| `7` | Email |
| `8` | HW3 TCP Practice |
| `q` | Quit |

## Why This Exists

Final-review slides are useful, but they are passive. This app reshapes the same
material into active recall prompts, then adds homework-derived TCP practice so
the arithmetic-heavy topics are not buried in a PDF.

The goal is simple: keep your hands on the keyboard and move through the exact
topics you need until the material is automatic.

## Terminal Notes

The UI is intentionally plain:

- ASCII-only output
- No full-width box drawing
- Hard-capped at 80 columns
- Single-key input without breaking newline rendering

That makes it behave well in common laptop terminals, SSH sessions, and terminal
multiplexers.

## Verification

```bash
cargo fmt --check
cargo test
```

## Project Layout

```text
.
|-- Cargo.toml
|-- Cargo.lock
|-- README.md
|-- review_fin (1).ppt
|-- hw3 sol_ CS 4470-01 (35218).pdf
`-- src
    `-- main.rs
```
