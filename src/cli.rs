use clap::{Parser, Subcommand, ValueEnum};

use crate::fifo;
use crate::firmware;

/// Logging and debug configuration
#[derive(Parser, Debug, Clone)]
pub struct LoggingArgs {
    /// Enable verbose debug logging (shows I2C transactions and internal state)
    #[arg(short = 'v', long, help = "Enable verbose debug logging")]
    pub verbose: bool,

    /// Log all register values after configuration for comparison with evaluation tools
    #[arg(long, help = "Debug register configuration (global option)")]
    pub debug_registers: bool,
}

/// Output configuration and formatting
#[derive(Parser, Debug, Clone)]
pub struct OutputArgs {
    /// Output format for measurement data
    #[arg(short = 'f', long, default_value = "human", help = "Output format")]
    pub format: OutputFormat,

    /// Suppress startup banner and configuration info
    #[arg(short = 'q', long, help = "Suppress startup messages")]
    pub quiet: bool,

    /// Enable FIFO output (compatible with spi-lib readers)
    #[arg(
        long,
        help = "Enable FIFO output to /tmp/presence for compatibility with existing readers"
    )]
    pub fifo_output: bool,

    /// FIFO output path
    #[arg(
        long,
        default_value = "/tmp/presence",
        help = "FIFO output path [default: /tmp/presence for spi-lib compatibility]"
    )]
    pub fifo_path: String,

    /// FIFO output format
    #[arg(
        long,
        default_value = "json",
        help = "FIFO output format: 'simple' (BGT60TR13C compatible) or 'json' (enhanced)"
    )]
    pub fifo_format: fifo::FifoFormat,

    /// FIFO output interval in seconds
    #[arg(
        long,
        default_value = "5.0",
        help = "FIFO output interval in seconds (5.0=spi-lib compatible, 0=every measurement)"
    )]
    pub fifo_interval: f32,
}

/// Parse I2C address from string, supporting both decimal and hex formats
fn parse_i2c_address(s: &str) -> Result<u16, String> {
    if let Some(hex_str) = s.strip_prefix("0x") {
        u16::from_str_radix(hex_str, 16).map_err(|_| format!("Invalid hex I2C address: {s}"))
    } else {
        s.parse::<u16>()
            .map_err(|_| format!("Invalid I2C address: {s}"))
    }
}

impl Cli {
    /// Get the I2C device path, using bus number if device path not specified
    pub fn get_i2c_device_path(&self) -> String {
        if let Some(device) = &self.i2c_device {
            device.clone()
        } else {
            format!("/dev/i2c-{}", self.i2c_bus)
        }
    }

    /// Get GPIO pin configuration from command line arguments
    pub fn get_gpio_pins(&self) -> crate::gpio::XM125GpioPins {
        crate::gpio::XM125GpioPins {
            reset: self.gpio_reset,
            mcu_interrupt: self.gpio_mcu_int,
            wake_up: self.gpio_wake,
            boot: self.gpio_boot,
        }
    }
}

#[derive(Parser)]
#[command(
    author = "Dynamic Devices Ltd",
    version,
    about = "XM125 Radar Module Monitor v2.0.0 - Clean CLI for Acconeer XM125 radar modules",
    long_about = "XM125 Radar Module Monitor v2.0.0

Production-ready CLI tool for Acconeer XM125 radar modules with automatic firmware 
management and streamlined commands for technicians.

⚠️  TESTING STATUS:
  ✅ PRESENCE DETECTION: Fully tested and production-ready (7m range verified)
  ⚠️  DISTANCE DETECTION: Code complete but untested - use with caution
  ⚠️  BREATHING MONITOR: Code complete but untested - use with caution

QUICK START:
  1. Check device status:        xm125-radar-monitor status
  2. Distance measurement:       xm125-radar-monitor distance
  3. Presence detection:         xm125-radar-monitor presence
  4. Continuous monitoring:      xm125-radar-monitor presence --continuous --count 100
  5. Firmware management:        xm125-radar-monitor firmware check

DISTANCE MEASUREMENT:
  # Single distance reading
  xm125-radar-monitor distance

  # Continuous distance monitoring with CSV export
  xm125-radar-monitor distance --continuous --count 100 --interval 500 --save-to distance.csv

  # Custom range configuration
  xm125-radar-monitor distance --range 0.1:3.0

PRESENCE DETECTION:
  # Single presence detection (default long range: 0.5m - 7.0m)
  xm125-radar-monitor presence

  # Custom range with high sensitivity
  xm125-radar-monitor presence --min-range 0.3 --max-range 5.0 --sensitivity 2.0

  # Continuous monitoring with register debugging
  xm125-radar-monitor --debug-registers presence --range long --continuous --count 100 --interval 500

  # Room occupancy monitoring with CSV logging
  xm125-radar-monitor presence --range long --continuous --save-to occupancy.csv

FIRMWARE & HARDWARE:
  # Check device status and firmware
  xm125-radar-monitor status
  xm125-radar-monitor info

  # Firmware management
  xm125-radar-monitor firmware check
  xm125-radar-monitor firmware update presence
  xm125-radar-monitor firmware bootloader

  # GPIO control
  xm125-radar-monitor gpio init
  xm125-radar-monitor gpio status

DEBUGGING & TROUBLESHOOTING:
  # Register debugging (works with any measurement command)
  xm125-radar-monitor --debug-registers presence --range medium

  # Verbose I2C logging
  xm125-radar-monitor --verbose distance

  # JSON output for automation
  xm125-radar-monitor --format json presence --continuous --count 10

All measurement commands automatically handle connection, firmware detection, and calibration.
Use --verbose for detailed I2C transaction logs and --debug-registers to compare with evaluation tools.
"
)]
pub struct Cli {
    /// I2C bus number (will be used as /dev/i2c-N if --i2c-device not specified)
    #[arg(
        short = 'b',
        long,
        default_value = "2",
        help = "I2C bus number [default: 2 for Sentai target]"
    )]
    pub i2c_bus: u8,

    /// I2C device path (e.g., /dev/i2c-2 for Sentai target)
    #[arg(short = 'd', long, help = "I2C device path (overrides --i2c-bus)")]
    pub i2c_device: Option<String>,

    /// I2C address of XM125 module in hex (e.g., 0x52 for standard XM125)
    #[arg(short = 'a', long, default_value = "0x52", value_parser = parse_i2c_address, help = "I2C address of XM125 module")]
    pub i2c_address: u16,

    /// Command timeout in seconds (how long to wait for device responses)
    #[arg(
        short = 't',
        long,
        default_value = "3",
        help = "Command timeout in seconds"
    )]
    pub timeout: u64,

    /// Logging and debug configuration
    #[command(flatten)]
    pub logging: LoggingArgs,

    /// Output configuration and formatting
    #[command(flatten)]
    pub output: OutputArgs,

    /// GPIO pin for XM125 reset control (active-low)
    #[arg(
        long,
        default_value = "124",
        help = "GPIO pin number for XM125 reset control [default: 124 for Sentai]"
    )]
    pub gpio_reset: u32,

    /// GPIO pin for XM125 MCU interrupt (input)
    #[arg(
        long,
        default_value = "125",
        help = "GPIO pin number for XM125 MCU interrupt [default: 125 for Sentai]"
    )]
    pub gpio_mcu_int: u32,

    /// GPIO pin for XM125 wake up control
    #[arg(
        long,
        default_value = "139",
        help = "GPIO pin number for XM125 wake up control [default: 139 for Sentai]"
    )]
    pub gpio_wake: u32,

    /// GPIO pin for XM125 bootloader control (BOOT0)
    #[arg(
        long,
        default_value = "141",
        help = "GPIO pin number for XM125 bootloader control [default: 141 for Sentai]"
    )]
    pub gpio_boot: u32,

    /// Firmware directory path (contains .bin files)
    #[arg(
        long,
        default_value = "/lib/firmware/acconeer",
        help = "Directory containing firmware binaries"
    )]
    pub firmware_path: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Check XM125 radar status and initialization state
    ///
    /// Shows device status flags, initialization progress, and error conditions.
    /// Use this first to verify the device is responding and properly initialized.
    Status,

    /// Get XM125 device information and firmware details
    ///
    /// Displays sensor ID, firmware version, application type, and current configuration.
    /// Useful for verifying correct firmware is loaded and device capabilities.
    Info,

    /// Read all I2C registers from presence detector firmware
    ///
    /// Dumps status, configuration, and measurement registers exposed by
    /// `i2c_presence_detector.bin` as JSON (default) or human-readable text.
    /// Requires presence firmware (application ID 2). Result registers are most
    /// meaningful while the detector is running (e.g. with xm125-radar-monitor.service).
    Registers,

    /// Perform distance measurement
    ///
    /// Measures distance to objects with high precision. Automatically configures
    /// the device for distance detection mode and handles firmware updates if needed.
    Distance {
        /// Detection range in meters (format: start:end, e.g., 0.1:3.0)
        #[arg(long, help = "Detection range in meters (start:end, e.g., 0.1:3.0)")]
        range: Option<String>,

        /// Enable continuous monitoring mode
        #[arg(long, help = "Continuously monitor distance measurements")]
        continuous: bool,

        /// Number of measurements in continuous mode (omit for infinite)
        #[arg(
            long,
            help = "Number of measurements to take (omit for infinite, requires --continuous)"
        )]
        count: Option<u32>,

        /// Measurement interval in milliseconds for continuous mode
        #[arg(
            long,
            default_value = "1000",
            help = "Time between measurements in ms (requires --continuous)"
        )]
        interval: u64,

        /// Save measurements to CSV file (continuous mode only)
        #[arg(
            long,
            help = "Output CSV file path (e.g., distance_data.csv, requires --continuous)"
        )]
        save_to: Option<String>,
    },

    /// Perform presence detection
    ///
    /// Detects motion and presence with configurable range and sensitivity.
    /// Automatically configures the device for presence detection mode.
    Presence {
        /// Presence detection range preset
        #[arg(
            long,
            help = "Detection range: short (6-70cm), medium (20cm-2m), long (50cm-7m)",
            conflicts_with_all = ["min_range", "max_range"]
        )]
        range: Option<PresenceRange>,

        /// Minimum detection range in meters (custom range)
        #[arg(
            long,
            help = "Minimum detection distance in meters (e.g., 0.3 for 30cm)",
            conflicts_with = "range",
            requires = "max_range"
        )]
        min_range: Option<f32>,

        /// Maximum detection range in meters (custom range)
        #[arg(
            long,
            help = "Maximum detection distance in meters (e.g., 5.0 for 5m)",
            conflicts_with = "range",
            requires = "min_range"
        )]
        max_range: Option<f32>,

        /// Detection sensitivity threshold (0.1 = low, 0.5 = medium, 2.0 = high)
        #[arg(
            long,
            help = "Detection sensitivity: lower = less sensitive, higher = more sensitive"
        )]
        sensitivity: Option<f32>,

        /// Frame rate for presence detection in Hz
        #[arg(
            long,
            help = "Measurement frequency in Hz (e.g., 12.0 for 12 measurements/second)"
        )]
        frame_rate: Option<f32>,

        /// Profile selection mode
        #[arg(
            long,
            default_value = "auto",
            help = "Profile mode: auto (firmware selects optimal profile) or manual (force Profile 5 for 7m)"
        )]
        profile: ProfileMode,

        /// Enable continuous monitoring mode
        #[arg(long, help = "Continuously monitor presence detection")]
        continuous: bool,

        /// Number of measurements in continuous mode (omit for infinite)
        #[arg(
            long,
            help = "Number of measurements to take (omit for infinite, requires --continuous)"
        )]
        count: Option<u32>,

        /// Measurement interval in milliseconds for continuous mode
        #[arg(
            long,
            default_value = "1000",
            help = "Time between measurements in ms (requires --continuous)"
        )]
        interval: u64,

        /// Save measurements to CSV file (continuous mode only)
        #[arg(
            long,
            help = "Output CSV file path (e.g., presence_data.csv, requires --continuous)"
        )]
        save_to: Option<String>,
    },

    /// Firmware management commands
    ///
    /// Comprehensive firmware operations including checking, updating, verification,
    /// and bootloader control. Handles automatic firmware detection and updates.
    Firmware {
        #[command(subcommand)]
        action: FirmwareAction,
    },

    /// GPIO control and testing commands
    ///
    /// Direct GPIO control for XM125 hardware management. Provides initialization,
    /// status monitoring, and reset control without external script dependencies.
    Gpio {
        #[command(subcommand)]
        action: GpioAction,
    },
}

#[derive(Subcommand)]
pub enum FirmwareAction {
    /// Check current firmware type and version
    ///
    /// Displays the currently loaded firmware information including type
    /// (distance/presence), version, and compatibility status.
    Check,

    /// Update firmware to match the specified detector mode
    ///
    /// Automatically flashes the correct firmware binary for the selected mode.
    /// Uses stm32flash and GPIO control for safe firmware updates.
    Update {
        /// Target firmware type (distance or presence)
        firmware_type: firmware::FirmwareType,

        /// Force update even if firmware already matches
        #[arg(short, long, help = "Force update even if current firmware matches")]
        force: bool,

        /// Verify firmware after update (adds delay and may timeout)
        #[arg(short, long, help = "Verify firmware integrity after update")]
        verify: bool,
    },

    /// Verify firmware integrity using checksums
    ///
    /// Compares the loaded firmware against known good checksums to detect
    /// corruption or version mismatches.
    Verify {
        /// Firmware type to verify against
        firmware_type: Option<firmware::FirmwareType>,
    },

    /// Erase the XM125 chip completely
    ///
    /// This will completely erase all firmware from the XM125 module.
    /// The module will need to be reprogrammed before it can be used again.
    /// Use with caution - this operation cannot be undone.
    Erase {
        /// Confirm the erase operation (required for safety)
        #[arg(long, help = "Confirm chip erase (required for safety)")]
        confirm: bool,
    },

    /// Calculate and display firmware checksums
    ///
    /// Calculates MD5 checksums for firmware binary files to verify integrity
    /// and compare against known good versions.
    Checksum {
        /// Specific firmware type to checksum (if not specified, shows all)
        firmware_type: Option<firmware::FirmwareType>,

        /// Show detailed information including file paths and sizes
        #[arg(short, long, help = "Show detailed checksum information")]
        verbose: bool,
    },

    /// Put XM125 module into bootloader mode for firmware programming
    ///
    /// Uses GPIO control to reset the module into bootloader mode (I2C address 0x48).
    /// This is required for firmware programming with stm32flash.
    Bootloader {
        /// Reset to run mode after entering bootloader (for testing)
        #[arg(long, help = "Reset back to run mode after bootloader test")]
        test_mode: bool,
    },
}

#[derive(Subcommand)]
pub enum GpioAction {
    /// Initialize GPIO pins and show status
    ///
    /// Exports and configures all XM125 GPIO pins for proper operation.
    /// This is automatically done by measurement commands but can be run manually.
    Init,

    /// Show current GPIO pin status
    ///
    /// Displays the current state of all XM125 GPIO pins including directions
    /// and values. Useful for hardware debugging.
    Status,

    /// Reset XM125 to run mode
    ///
    /// Performs a hardware reset with BOOT0 pin LOW to enter normal run mode.
    /// The device will be available at I2C address 0x52 after reset.
    ResetRun,

    /// Reset XM125 to bootloader mode
    ///
    /// Performs a hardware reset with BOOT0 pin HIGH to enter bootloader mode.
    /// The device will be available at I2C address 0x48 for firmware programming.
    ResetBootloader,

    /// Test bootloader control functionality
    ///
    /// Tests the bootloader pin control by cycling between bootloader and run modes.
    /// Useful for verifying GPIO hardware connections.
    Test,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable output with labels and units (default)
    Human,
    /// JSON format for programmatic processing
    Json,
    /// Comma-separated values for data analysis
    Csv,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum PresenceRange {
    /// Short range: 6cm to 70cm (good for close proximity detection)
    Short,
    /// Medium range: 20cm to 2m (balanced range and sensitivity)
    Medium,
    /// Long range: 50cm to 7m (maximum detection range)
    Long,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum ProfileMode {
    /// Automatic profile selection based on range (default, recommended)
    Auto,
    /// Manual profile selection for advanced users (Profile 5 for 7m range)
    Manual,
}

impl From<PresenceRange> for crate::radar::PresenceRange {
    fn from(cli_range: PresenceRange) -> Self {
        match cli_range {
            PresenceRange::Short => crate::radar::PresenceRange::Short,
            PresenceRange::Medium => crate::radar::PresenceRange::Medium,
            PresenceRange::Long => crate::radar::PresenceRange::Long,
        }
    }
}
