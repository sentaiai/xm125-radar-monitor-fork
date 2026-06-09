# XM125 Radar Monitor

[![Build Status](https://github.com/DynamicDevices/xm125-radar-monitor/workflows/CI/badge.svg)](https://github.com/DynamicDevices/xm125-radar-monitor/actions)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-ARM64%20Linux-green.svg)](https://github.com/DynamicDevices/xm125-radar-monitor)

Production-ready CLI tool for Acconeer XM125 radar modules with verified 7m detection range and comprehensive configuration control.

**Version**: 2.0.16  
**Maintainer**: Alex J Lennon (ajlennon@dynamicdevices.co.uk)  
**Copyright**: © 2025 Dynamic Devices Ltd. All rights reserved.

## ⚠️ Testing Status

**Currently Tested & Verified**: ✅ **Presence Detection Mode**
- Fully tested and verified in production environments
- 7m detection range confirmed and working reliably
- All features (continuous monitoring, CSV export, FIFO integration) validated

**Supported but Untested**: ⚠️ **Distance Detection & Breathing Monitor Modes**
- Code implementation complete with full firmware support
- **May not be working correctly** - requires testing and validation
- Use with caution in production environments
- Testing and validation contributions welcome

## Features

- **Verified 7m Detection Range**: Properly configured Profile 5 with Auto Profile disabled
- **Measurement-Centric CLI**: Clean, intuitive commands for distance and presence detection
- **Advanced Testing Framework**: Range, angle, and false positive analysis with signal strength indicators
- **Automatic Firmware Management**: Auto-detects and updates firmware via `stm32flash`
- **Comprehensive Configuration**: Direct parameter control with custom ranges up to 7m
- **Enhanced Monitoring**: Continuous operation with detailed CSV export and confidence analysis
- **FIFO Integration**: Extended JSON on `/tmp/presence` for systemd and container readers; simple format remains BGT60-compatible
- **I2C Register Snapshot**: `registers` command dumps all presence-firmware registers with `R_` / `RW_` access prefixes
- **Internal GPIO Control**: Hardware reset and bootloader control without external scripts
- **Register-Level Debugging**: Complete register dumps with descriptions for optimization
- **Cross-compilation**: Native ARM64 builds for embedded targets

## Quick Start

```bash
# Device status and information
sudo xm125-radar-monitor status
sudo xm125-radar-monitor info

# Presence detection (verified 7m range capability)
sudo xm125-radar-monitor presence --range long                    # 0.3m-5.5m preset (auto profile)
sudo xm125-radar-monitor presence --min-range 0.5 --max-range 7.0 # Full 7m range (auto profile)
sudo xm125-radar-monitor presence --min-range 0.5 --max-range 7.0 --profile manual # 7m range (manual Profile 5)

# Distance measurement
sudo xm125-radar-monitor distance --min-range 0.2 --max-range 3.0

# Continuous monitoring with CSV export
sudo xm125-radar-monitor presence --min-range 0.5 --max-range 7.0 --continuous --count 100 --save-to data.csv

# Register debugging (verify configuration)
sudo xm125-radar-monitor --debug-registers presence --min-range 0.5 --max-range 7.0 --profile manual

# Full I2C register snapshot (presence firmware only)
sudo systemctl stop xm125-radar-monitor.service   # avoid I2C contention
sudo xm125-radar-monitor -q registers
```

## Hardware Requirements

- **Target**: ARM64 Linux (tested on Sentai i.MX8MM)
- **I2C**: `/dev/i2c-2` at address `0x52` (run mode) / `0x48` (bootloader)
- **GPIO**: 124 (reset), 125 (interrupt), 139 (wake), 141 (boot)
- **Firmware**: `/lib/firmware/acconeer/*.bin`

## Command Structure

### Core Commands

```bash
# Device information and status
sudo xm125-radar-monitor status          # Connection and firmware status
sudo xm125-radar-monitor info            # Detailed device information

# Measurement commands
sudo xm125-radar-monitor distance        # Distance measurement mode
sudo xm125-radar-monitor presence        # Presence detection mode
sudo xm125-radar-monitor registers       # One-shot I2C register snapshot (presence FW)

# Hardware and firmware management
sudo xm125-radar-monitor firmware        # Firmware operations (check, update, verify, erase)
sudo xm125-radar-monitor gpio            # GPIO control (init, status, reset, test)
```

### Presence Detection Configuration

#### Range Options

```bash
# Preset ranges (default: long)
--range short    # 6cm - 70cm (close proximity)
--range medium   # 20cm - 2m (balanced)  
--range long     # 30cm - 5.5m (room occupancy, default)

# Custom ranges (both required, conflicts with presets)
--min-range 0.5 --max-range 7.0   # Full 7m range (verified)
--min-range 0.3 --max-range 5.0   # Custom 30cm - 5m range
```

#### Detection Parameters

```bash
# Sensitivity control (0.1 = low, 5.0 = high)
--sensitivity 1.5

# Frame rate control (1.0 - 60.0 Hz)
--frame-rate 20.0
```

#### Profile Mode Configuration

```bash
# Automatic profile selection (default, recommended)
--profile auto      # Firmware selects optimal profile based on range

# Manual profile selection (advanced users)
--profile manual    # Force Profile 5 for maximum 7m range capability
```

#### Continuous Monitoring

```bash
# Enable continuous mode
--continuous

# Number of measurements (omit for infinite)
--count 100

# Measurement interval in milliseconds
--interval 500

# Save to CSV file
--save-to presence_data.csv
```

## Complete Usage Examples

### Single Measurements

```bash
# Basic presence detection (default long range: 0.5m - 7.0m)
sudo xm125-radar-monitor presence

# High-sensitivity close proximity detection
sudo xm125-radar-monitor presence --presence-range short --sensitivity 2.5

# Custom range with balanced settings
sudo xm125-radar-monitor presence --min-range 0.5 --max-range 3.0 --sensitivity 1.2
```

### Continuous Monitoring

```bash
# Continuous monitoring with register debugging
sudo xm125-radar-monitor --debug-registers presence --min-range 0.3 --max-range 5.0 --continuous --count 100 --interval 500

# Long range room occupancy monitoring with CSV output
sudo xm125-radar-monitor presence --presence-range long --continuous --save-to occupancy.csv

# FIFO output for system integration (spi-lib compatible)
sudo xm125-radar-monitor presence --continuous --fifo-output --fifo-format simple --fifo-interval 5.0

# Real-time FIFO output with enhanced JSON data
sudo xm125-radar-monitor presence --continuous --fifo-output --fifo-format json --fifo-interval 0

# Power-efficient infinite monitoring (2 second intervals)
sudo xm125-radar-monitor presence --presence-range long --frame-rate 5.0 --continuous --interval 2000

# High-frequency monitoring for 50 measurements
sudo xm125-radar-monitor presence --presence-range short --sensitivity 2.0 --continuous --count 50 --interval 200
```

## Firmware Management

```bash
# Check current firmware
sudo xm125-radar-monitor firmware check

# Update to presence detector firmware
sudo xm125-radar-monitor firmware update presence

# Verify firmware integrity
sudo xm125-radar-monitor firmware verify

# Erase chip (requires confirmation)
sudo xm125-radar-monitor firmware erase --confirm

# Calculate firmware checksums
sudo xm125-radar-monitor firmware checksum --verbose
```

## GPIO Control

Internal GPIO management without external script dependencies:

```bash
# Initialize GPIO pins
sudo xm125-radar-monitor gpio init

# Show GPIO status
sudo xm125-radar-monitor gpio status

# Reset to run mode
sudo xm125-radar-monitor gpio reset-run

# Reset to bootloader mode
sudo xm125-radar-monitor gpio reset-bootloader
```

## Register Debugging

Compare configuration with Acconeer evaluation tools:

```bash
# Debug presence registers with default configuration
sudo xm125-radar-monitor --debug-registers presence

# Debug with custom configuration
sudo xm125-radar-monitor --debug-registers presence --min-range 0.3 --max-range 5.0 --sensitivity 1.8

# Debug during continuous monitoring
sudo xm125-radar-monitor --debug-registers presence --presence-range medium --continuous --count 10
```

**Example Register Output:**
```
================================================================================
XM125 Register Dump - Presence Mode
================================================================================

👤 Presence Detector Configuration:
────────────────────────────────────────────────────────────────────────────────
  0x0040 ( 64) │ Start Range               │ 0x0000012C (        300) │ Presence detection start distance (mm)
  0x0041 ( 65) │ End Range                 │ 0x00001388 (       5000) │ Presence detection end distance (mm)
  0x0042 ( 66) │ Intra Threshold           │ 0x00000708 (       1800) │ Fast motion detection threshold
  0x0043 ( 67) │ Inter Threshold           │ 0x00000640 (       1600) │ Slow motion detection threshold
================================================================================
```

## Detection Modes

| Mode | Range | Update Rate | Primary Use |
|------|-------|-------------|-------------|
| **Distance** | 0.1-3.0m | ~100ms | Precise range measurement |
| **Presence** | 0.06-7.0m | ~100ms | Motion/occupancy detection |
| **Breathing** | 0.3-1.5m | 5-20s | Breathing pattern analysis |

## FIFO Integration (System Integration)

The XM125 radar monitor writes presence data to **`/tmp/presence`** (named pipe). The default **systemd service** uses extended JSON every 5 seconds:

```bash
# Service ExecStart (reference)
xm125-radar-monitor --fifo-output --fifo-format json --fifo-interval 5.0 --quiet \
  presence --min-range 0.5 --max-range 7.0 --continuous
```

Stop the service before manual CLI use on the same I2C bus:

```bash
sudo systemctl stop xm125-radar-monitor.service
```

### FIFO Configuration

```bash
# Basic FIFO output (spi-lib compatible simple format, 5-second intervals)
sudo xm125-radar-monitor presence --continuous --fifo-output

# Custom FIFO path
sudo xm125-radar-monitor presence --continuous --fifo-output --fifo-path /tmp/custom_fifo

# Real-time mode (every measurement)
sudo xm125-radar-monitor presence --continuous --fifo-output --fifo-interval 0

# Extended JSON format (used by xm125-radar-monitor.service)
sudo xm125-radar-monitor presence --continuous --fifo-output --fifo-format json --fifo-interval 5.0
```

### FIFO Output Formats

#### Simple Format (BGT60TR13C Compatible)

```
1 2.45
0 0.00
STATUS Starting up
STATUS App exit
```

#### JSON Format (`--fifo-format json`) — `/tmp/presence` only

This is the **extended** format written to the FIFO. CLI `--format json` on `presence` still uses the shorter console JSON (see [Output Formats](#output-formats)).

All floating-point values are formatted to **3 decimal places**.

```json
{
  "confidence": "HIGH",
  "detection_mode": "presence",
  "intra_score": 7.781,
  "inter_score": 10.379,
  "presence_detected": true,
  "presence_distance_m": 0.640,
  "sensor_type": "XM125",
  "signal_quality": "STRONG",
  "timestamp": "2026-06-09 11:09:54.106",
  "presence_sticky": true,
  "R_start_m": 0.500,
  "R_end_m": 7.000,
  "distance_zone": "near",
  "intra_threshold": 1.300,
  "inter_threshold": 1.000,
  "actual_frame_rate_hz": 5.263,
  "detector_error": false,
  "internal_temperature_c": 47,
  "movement_type": "slow"
}
```

| Field | Description |
|-------|-------------|
| `confidence` | `NONE` / `LOW` / `MEDIUM` / `HIGH` (derived from scores when presence detected) |
| `intra_score` | Fast motion score |
| `inter_score` | Slow motion score |
| `presence_distance_m` | Estimated target distance (m) |
| `presence_sticky` | Latched presence since last read (I2C register bit) |
| `R_start_m` / `R_end_m` | Configured detection range from hardware registers |
| `distance_zone` | `near` (0.5 m to under 1.0 m), `mid` (1.0 m to under 3.0 m), `far` (3.0 m to 7.0 m), or `out_of_range` |
| `intra_threshold` / `inter_threshold` | Active detection thresholds from config registers |
| `actual_frame_rate_hz` | Achieved frame rate (may be lower than configured rate) |
| `internal_temperature_c` | A121 chip internal temperature (not room temp) |
| `movement_type` | See table below |

**`movement_type` logic**

| Condition | Value |
|-----------|-------|
| `presence_detected == false` | `none` |
| `intra_score > inter_score` | `fast` |
| otherwise | `slow` |

### Reading FIFO Data

```bash
cat /tmp/presence
tail -f /tmp/presence
your_existing_reader < /tmp/presence
```

## I2C Register Snapshot (`registers`)

Reads **all** I2C registers exposed by `i2c_presence_detector.bin` in one shot. Requires **presence firmware** (application ID `2`). Best used while the detector is running, or after configuring with `presence`.

```bash
# Stop service first to avoid I2C contention
sudo systemctl stop xm125-radar-monitor.service

# JSON snapshot (default human mode also prints JSON)
sudo xm125-radar-monitor -q registers
sudo xm125-radar-monitor --format json registers

# Global flags must come before the subcommand
xm125-radar-monitor -q --format json registers   # correct
xm125-radar-monitor registers --format json    # wrong (--format ignored)
```

### Register JSON field prefixes

| Prefix | Meaning |
|--------|---------|
| `R_` | Read-only register (or derived read-only field) |
| `RW_` | Read/write configuration register (change via I2C, then stop → apply → start) |
| *(none)* | Host metadata (`timestamp`) or grouping keys (`version`, `results`, `configuration`) |

### Example `registers` output

```json
{
  "timestamp": "2026-06-05T08:25:36.705998644Z",
  "R_application_id": 2,
  "R_application_name": "presence_detector",
  "version": {
    "R_major": 1,
    "R_minor": 11,
    "R_patch": 1,
    "R_raw": 68353
  },
  "R_protocol_status": 0,
  "R_measure_counter": 3356,
  "detector_status": {
    "R_raw": 255,
    "R_busy": false,
    "R_rss_register_ok": true,
    "R_config_apply_ok": true,
    "R_detector_error": false
  },
  "results": {
    "R_presence_detected": true,
    "R_presence_sticky": true,
    "R_detector_error": false,
    "R_internal_temperature_c": 51,
    "R_presence_distance_m": 0.64,
    "R_intra_presence_score": 2.553,
    "R_inter_presence_score": 5.24,
    "R_actual_frame_rate_hz": 5.263
  },
  "configuration": {
    "RW_sweeps_per_frame": 16,
    "RW_frame_rate_hz": 12.0,
    "RW_intra_detection_threshold": 1.3,
    "RW_inter_detection_threshold": 1.0,
    "RW_start_mm": 500,
    "RW_end_mm": 7000,
    "R_start_m": 0.5,
    "R_end_m": 7.0,
    "RW_manual_profile": 5,
    "RW_hwaas": 32,
    "RW_signal_quality": 20000,
    "RW_detection_on_gpio": false
  }
}
```

The snapshot includes every readable presence register (status, live results, and full configuration). Register map: Acconeer `presence_reg_protocol.h`. Command register `256` is write-only and is not included.

**Difference from FIFO JSON:** `registers` is a one-time diagnostic dump with `R_`/`RW_` keys and nested sections. FIFO JSON is a flat stream optimized for consumers on `/tmp/presence` (`distance_zone`, `movement_type`, etc.).

## Configuration Options

### I2C & Hardware

```bash
# Custom I2C bus and address
sudo xm125-radar-monitor -b 1 -a 0x53 status

# Custom GPIO pins for different hardware
sudo xm125-radar-monitor --gpio-reset 100 --gpio-boot 101 status

# Disable auto-reconnect for debugging
sudo xm125-radar-monitor --no-auto-reconnect status
```

### Output Formats

```bash
# Human-readable (default)
sudo xm125-radar-monitor presence

# JSON output for APIs (short format — not the same as FIFO JSON)
sudo xm125-radar-monitor --format json presence

# CSV output for data analysis
sudo xm125-radar-monitor --format csv presence
```

**CLI presence JSON** (console / `--format json`):

```json
{
  "timestamp": "2026-06-09 11:09:54.106",
  "presence_detected": true,
  "presence_distance_m": 0.64,
  "intra_score": 7.781,
  "inter_score": 10.379,
  "signal_quality": "STRONG",
  "confidence": "HIGH"
}
```

For the **extended** JSON with `movement_type`, `distance_zone`, register-backed fields, etc., use `--fifo-output --fifo-format json` (see [FIFO Integration](#fifo-integration-system-integration)).

## Build & Deploy

```bash
# Cross-compile for ARM64
./build-aarch64.sh

# Deploy to target
scp target/aarch64-unknown-linux-gnu/release/xm125-radar-monitor user@target:/usr/local/bin/
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Device not found | `i2cdetect -y 2` to verify I2C bus |
| Permission denied | Run with `sudo` for I2C/GPIO access |
| Unknown command errors | Reset device: `sudo xm125-radar-monitor gpio reset-run` |
| Calibration timeout | Check hardware connections and power |
| Firmware update fails | Ensure device in bootloader mode: `sudo xm125-radar-monitor bootloader` |
| Register values incorrect | Use `--debug-registers` or `registers` to verify configuration |
| `registers` fails / wrong app ID | Flash presence firmware: `firmware update presence` |
| Empty or stale `/tmp/presence` | Ensure service running; reader must open FIFO (`tail -f /tmp/presence`) |

Use `--verbose` for detailed I2C transaction logs and debugging information.

## Dependencies

- **Runtime**: `stm32flash`, `i2cdetect`, `i2cget`
- **Build**: Rust 1.70+, cross-compilation toolchain for ARM64, `csv` crate
- **Hardware**: Linux GPIO sysfs interface

## 📚 Documentation

Comprehensive documentation is available in the [`docs/`](docs/) directory:

- **[📋 Project Context](docs/PROJECT_CONTEXT.md)** - Architecture and high-level overview
- **[📡 FIFO Interface Specification](docs/XM125_FIFO_INTERFACE_SPECIFICATION.md)** - Complete interface spec for container developers
- **[🧪 Hardware Testing Guide](docs/HARDWARE_TEST_GUIDE.md)** - Testing procedures and protocols
- **[⚡ Testing Quick Reference](docs/TESTING_QUICK_REFERENCE.md)** - Common testing commands
- **[📊 Performance Analysis](docs/RADAR_PERFORMANCE_ANALYSIS_REPORT.md)** - Benchmarking and analysis results

See the [documentation index](docs/README.md) for the complete list.

## License

Licensed under GNU General Public License v3.0. See [LICENSE](LICENSE) for details.

---

**Keywords**: Acconeer XM125, radar sensor, I2C communication, firmware management, distance detection, presence detection, embedded Linux, ARM64, cross-compilation, stm32flash, continuous monitoring, CSV export