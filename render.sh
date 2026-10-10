#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p passes
cargo build --release
cargo run --release --bin field -- render -o passes/field.mp4
python3 score.py
ffmpeg -v error -i passes/score-raw.wav -af loudnorm=I=-19:TP=-1.5:LRA=8 -ar 48000 -c:a pcm_s16le -y passes/score.wav
cargo run --release --bin cinema-lens -- render -o mahaganapati-wave.mp4
