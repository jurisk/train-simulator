#!/usr/bin/env zsh
bin/housekeeping.sh
cargo nextest run --no-fail-fast
