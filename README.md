# Matcher-rs

An order matching engine written in Rust that can do sub-microsecond order processing/matching times.
I hope to add more orderbook-api-type functionality eventually.

## Motivation

I am really interested in real-time/low-latency systems like what is used in the
financial industry. I heard a talk given by Jane Street describing some of the
software/hardware that runs their exchange and was inspired to give it a try myself.
I picked Rust because I had not used in on a project before, and I thought that
this was reasonable enough in size/complexity for me to do.

## Description

An order matching engine with sub-microsecond order processing/matching times.


## Quick Start

```bash
git clone https://github.com/hallmason17/matcher-rs.git
cd matcher-rs
cargo run #--release
```

## TODO

* [ ] Modify placed orders
* [ ] Cancel placed orders
* [ ] API to get orderbook snapshots
