# rusty-shaper

Low-RAM Rust input shaper calibration for 3D printers.

A re-implementation of Kalico's (Klipper) core shaper calibration logic in pure
Rust, with a focus on minimal RAM usage, extensibility, and correctness.

## Why?

The stock Python-based calibration in Kalico/Klipper works well, but:

- It requires NumPy, which is heavy on embedded/limited-RAM hosts
- It loads entire accelerometer datasets into memory
- Adding new shaper types requires modifying the core codebase

`rusty-shaper` solves these by:

- **Streaming PSD computation**: Raw data is processed in overlapping windows
  without materializing the full sample matrix
- **Trait-based extensibility**: New shaper types implement a single `ShaperModel`
  trait without touching the scoring engine
- **Zero Python/NumPy dependency**: All math is explicit and deterministic
- **Small binary**: Release builds strip to ~1-2 MB with LTO

## Quick Start

```bash
# Default shapers: zv, mzv, ei, 2hump_ei, 3hump_ei.
# Matches Klipper's AUTOTUNE_SHAPERS (klippy/extras/shaper_calibrate.py).
# ZVD is excluded by default.
cargo run --release -- /path/to/resonances.csv

# Output formats (can specify multiple --output for multiple outputs)
# --output cfg        Printer.cfg block to stdout
# --output csv        Kalico-style CSV to workdir
# --output json       Compact JSON to workdir
# --output json-pretty  Pretty-printed JSON to workdir
# --output klippy     Live printer update via Moonraker

# CSV calibration data (Kalico-compatible format)
cargo run --release -- /path/to/resonances.csv --output csv

# Multiple outputs at once
cargo run --release -- /path/to/resonances.csv --output csv --output json --output klippy

# Custom workdir (default: /tmp)
cargo run --release -- /path/to/resonances.csv --output csv --workdir .

# Custom name for output files (default: timestamp %Y%m%d_%H%M%S)
cargo run --release -- /path/to/resonances.csv --output csv --name mycal

# Live printer update via Moonraker
cargo run --release -- /path/to/resonances.csv --output klippy

# Persist to printer.cfg (requires --output klippy)
cargo run --release -- /path/to/resonances.csv --output klippy --commit

# MZV only
cargo run --release -- /path/to/resonances.csv --shapers mzv

# Include ZVD explicitly
cargo run --release -- /path/to/resonances.csv --shapers mzv,zvd,ei,zv,2hump_ei,3hump_ei

# Custom damping ratio
cargo run --release -- /path/to/resonances.csv --damping-ratio 0.15

# Override frequency range (default: shaper-specific min to 150 Hz, step 0.2)
cargo run --release -- /path/to/resonances.csv --shaper-freq 20:100:0.5

# Limit smoothing (default: no limit — picks lowest score)
cargo run --release -- /path/to/resonances.csv --max-smoothing 0.15
```

## Defaults

| Parameter | Default | Notes |
|-----------|---------|-------|
| `--output` | *none* | Output format(s): `cfg`, `csv`, `json`, `json-pretty`, `klippy`. Can specify multiple times |
| `--workdir` | `/tmp` | Directory for output files (CSV, JSON) |
| `--name` | *timestamp* | Name suffix for output files (`%Y%m%d_%H%M%S`) |
| `--shapers` | `zv,mzv,ei,2hump_ei,3hump_ei` | Matches Klipper's `AUTOTUNE_SHAPERS` (see `klippy/extras/shaper_calibrate.py`); ZVD excluded |
| `--damping-ratio` | `0.1` | Primary damping ratio for coefficient generation |
| `--test-damping-ratios` | `0.075,0.1,0.15` | Tested to find worst-case vibrations |
| `--scv` | `5.0` | Square corner velocity (mm/s) |
| `--max-freq` | `200.0` | Maximum PSD frequency to analyze (Hz) |
| `--shaper-freq` | *shaper min* to `150.0`, step `0.2` | Frequency search range per shaper |
| `--max-smoothing` | *none* | Optional hard limit. If unset, Kalico-style selection picks lowest score without smoothing constraint |
| `--window-t` | `0.5` | Welch window length for raw→PSD conversion (seconds) |

**Note on smoothing:** `max_accel` is always computed using an internal `TARGET_SMOOTHING = 0.12` as the "comfortable operating point" (same as Kalico). This is independent of `--max-smoothing`, which only affects shaper selection. If you don't specify `--max-smoothing`, the scorer uses Kalico's default behavior: pick the lowest score without an explicit smoothing cap.

## Input Formats

### PSD CSV (default)

The CSV saved by Kalico/Klipper's shaper calibration tooling:

```csv
freq,psd_x,psd_y,psd_z,psd_xyz,accel_per_hz
0.0,1.23e-4,2.34e-4,3.45e-4,6.02e-4,100.0
0.5,1.25e-4,2.36e-4,3.47e-4,6.08e-4,100.0
...
```

### Raw Accelerometer CSV

Raw accelerometer data is auto-detected and converted to PSD automatically:

```csv
#time,accel_x,accel_y,accel_z
0.0,0.001,0.002,9.81
0.0005,0.0011,0.0021,9.81
...
```

No special flags needed — just pass the file path. `--window-t` still applies
when raw input is auto-detected.

## Output Formats

### `cfg`

Prints a ready-to-paste `[input_shaper]` block for `printer.cfg`:

```ini
[input_shaper]
# Recommended max_accel <= 4000 mm/s²
shaper_type_x = mzv
shaper_freq_x = 36.8
```

Axis is auto-detected from the filename (`_y` or `-y` suffix → Y axis, otherwise X).

### `json` / `json-pretty`

Full structured output with all shaper results, scores, and vibration data.

## Architecture

```
Input ──► Parser ──► PSD ──► ShaperModel ──► Scorer ──► Recommendation
          (CSV)     (Welch)   (MZV, EI...)            (stdout)
```

### Modules

| Module | Purpose |
|--------|---------|
| `input` | CSV parsing for PSD and raw accelerometer data; Welch PSD computation |
| `models` | Shaper coefficient definitions (ZV, MZV, ZVD, EI, 2HUMP_EI, 3HUMP_EI). Default set `zv, mzv, ei, 2hump_ei, 3hump_ei` matches Klipper's `AUTOTUNE_SHAPERS` |
| `scorer` | Frequency response estimation, vibration scoring, smoothing, selection |
| `types`  | Core types: `PsdBin`, `ShaperCoefficients`, `ShaperConfig`, etc. |

### Extending with New Shapers

Implement the `ShaperModel` trait:

```rust
use rusty_shaper::models::ShaperModel;
use rusty_shaper::types::{ShaperCoefficients, Frequency, DampingRatio};

pub struct MyShaper;

impl ShaperModel for MyShaper {
    fn name(&self) -> &'static str { "my_shaper" }
    fn min_freq(&self) -> Frequency { 30.0 }

    fn coefficients(&self, freq: Frequency, damping: DampingRatio) -> ShaperCoefficients {
        let a1 = 1.0;
        let a2 = damping;
        let t1 = 0.5 / freq;

        ShaperCoefficients::new(&[a1, a2], &[0.0, t1])
    }
}
```

Then add it to the calibrator:

```rust
let calibrator = ShaperCalibrator::new()
    .with_shaper(Box::new(MyShaper));
```

## Validation

`rusty-shaper` has been validated against Kalico's `shaper_calibrate.py` on
synthetic resonance data and real printer accelerometer captures. Current
validation runs are within 0.5 Hz of Kalico on the checked captures, well inside
the practical tolerance for 3D printer input shaper tuning.

## Performance

See [BENCHMARKS.md](BENCHMARKS.md) for detailed x64 and ARM runtime/memory benchmarks.

Quick summary on a 5.4 MB raw accelerometer CSV:

| Platform | Time | Memory |
|----------|------|--------|
| x86_64 (Ryzen 9 7940HS) | ~80 ms | ~3.6 MB |
| ARMv7 (live Centauri Carbon 1 running Cosmos) | ~3.3 s | ~4.1 MB |

The ARM binary is ~40× slower but uses only ~13% more RAM than x64, thanks to the streaming raw→PSD path.

## Building

```bash
cd rusty-shaper

# Debug build for the host
cargo build

# Release build for the host
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- /path/to/data.csv

# Run with JSON output
RUST_LOG=debug cargo run -- /path/to/data.csv --output json
```

### Centauri Carbon 1 / ARM target build

Always use `cross` for printer-target ARM builds:

```bash
cd rusty-shaper
cross build --release --target armv7-unknown-linux-musleabihf
```

Do **not** use plain `cargo build --release --target armv7-unknown-linux-musleabihf`
for deployment to a live Centauri Carbon 1 running Cosmos. With Rust 1.96,
plain Cargo produces static ARM musl binaries that segfault before `main()` on
that target—even a minimal hello-world binary does it. `cross build` produces
the working binary.

## License

GPL-3.0 — same as Kalico/Klipper.

## Contributing

Part of the [OpenCentauri](https://github.com/OpenCentauri/OpenCentauri)
project. Open issues or PRs on the main repo.
