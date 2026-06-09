// Presence Detection Module
// Implements presence detection functionality with proper datasheet compliance

#![allow(clippy::pedantic)]

use super::registers::{
    CMD_PRESENCE_APPLY_CONFIGURATION, CMD_PRESENCE_RESET_MODULE, CMD_PRESENCE_START_DETECTOR,
    CMD_PRESENCE_STOP_DETECTOR, CMD_RESET_MODULE, PRESENCE_REG_ACTUAL_FRAME_RATE_ADDRESS,
    PRESENCE_REG_AUTO_PROFILE_ADDRESS, PRESENCE_REG_AUTO_STEP_LENGTH_ADDRESS,
    PRESENCE_REG_AUTO_SUBSWEEPS_ADDRESS, PRESENCE_REG_COMMAND_ADDRESS,
    PRESENCE_REG_DETECTOR_STATUS_ADDRESS, PRESENCE_REG_END_ADDRESS, PRESENCE_REG_FRAME_RATE_ADDRESS,
    PRESENCE_REG_HWAAS_ADDRESS, PRESENCE_REG_INTER_DETECTION_THRESHOLD_ADDRESS,
    PRESENCE_REG_INTRA_DETECTION_THRESHOLD_ADDRESS, PRESENCE_REG_MANUAL_PROFILE_ADDRESS,
    PRESENCE_REG_MANUAL_STEP_LENGTH_ADDRESS, PRESENCE_REG_SIGNAL_QUALITY_ADDRESS,
    PRESENCE_REG_START_ADDRESS, PRESENCE_RESULT_DETECTED_MASK,
    PRESENCE_RESULT_DETECTOR_ERROR_MASK, PRESENCE_RESULT_STICKY_MASK,
    PRESENCE_RESULT_TEMPERATURE_MASK, REG_INTER_PRESENCE_SCORE, REG_INTRA_PRESENCE_SCORE,
    REG_PRESENCE_DISTANCE, REG_PRESENCE_RESULT, STATUS_BUSY_MASK, STATUS_ERROR_MASK,
};
use crate::error::{RadarError, Result};
use crate::i2c::I2cDevice;
use log::{info, warn};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PresenceRange {
    Short,  // 0.06m - 0.7m (6cm - 70cm)
    Medium, // 0.2m - 2.0m (20cm - 2m)
    Long,   // 0.3m - 5.5m (30cm - 5.5m) - Updated to match Philip's working config
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceMeasurement {
    pub presence_detected: bool,
    pub presence_sticky: bool,
    pub presence_distance: f32,
    pub intra_presence_score: f32, // Fast motion score
    pub inter_presence_score: f32, // Slow motion score
    pub detector_error: bool,
    pub internal_temperature_c: i16,
    pub actual_frame_rate_hz: f32,
    pub start_m: f32,
    pub end_m: f32,
    pub intra_threshold: f32,
    pub inter_threshold: f32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct PresenceDetector<'a> {
    i2c: &'a mut I2cDevice,
}

impl<'a> PresenceDetector<'a> {
    pub fn new(i2c: &'a mut I2cDevice) -> Self {
        Self { i2c }
    }

    /// Calculate optimal profile based on detection range
    fn calculate_optimal_profile(_start_mm: u32, end_mm: u32) -> u32 {
        let max_range_m = end_mm as f32 / 1000.0;

        match max_range_m {
            r if r <= 0.7 => 1, // Profile 1: Short range (0.06m - 0.7m)
            r if r <= 2.0 => 2, // Profile 2: Short-medium range (0.2m - 2.0m)
            r if r <= 3.5 => 3, // Profile 3: Medium range (0.3m - 3.5m)
            r if r <= 6.0 => 4, // Profile 4: Medium-long range (0.5m - 6.0m)
            _ => 5,             // Profile 5: Long range (1.0m - 7.0m+)
        }
    }

    /// Calculate optimal step length based on maximum range
    fn calculate_optimal_step_length(end_mm: u32) -> u32 {
        let max_range_m = end_mm as f32 / 1000.0;

        // Step length should be approximately range_length / 60 for optimal performance
        // But with minimum and maximum bounds
        let calculated = (max_range_m * 1000.0 / 60.0) as u32;

        // Clamp to reasonable bounds
        calculated.clamp(12, 120)
    }

    /// Configure presence range according to datasheet specifications
    pub fn configure_range(
        &mut self,
        range: PresenceRange,
        custom_start_m: Option<f32>,
        custom_length_m: Option<f32>,
    ) -> Result<(u32, u32)> {
        info!("🎯 Configuring presence range preset: {:?}", range);

        // Convert PresenceRange enum to millimeter values according to datasheet
        let (start_mm, end_mm) = match range {
            PresenceRange::Short => {
                // Short range: 6cm to 70cm (60mm to 700mm)
                (60u32, 700u32)
            }
            PresenceRange::Medium => {
                // Medium range: 20cm to 2m (200mm to 2000mm)
                (200u32, 2000u32)
            }
            PresenceRange::Long => {
                // Long range: 30cm to 5.5m (300mm to 5500mm) - Philip's working config
                (300u32, 5500u32)
            }
        };

        // Check for custom range override (when min/max range is explicitly set)
        let (final_start_mm, final_end_mm) =
            if let (Some(start_m), Some(length_m)) = (custom_start_m, custom_length_m) {
                // Custom range specified via CLI --min-range and --max-range
                let start_mm = (start_m * 1000.0) as u32;
                let end_mm = ((start_m + length_m) * 1000.0) as u32;
                info!(
                    "Using custom range: {:.3}m to {:.3}m",
                    start_m,
                    start_m + length_m
                );
                (start_mm, end_mm)
            } else {
                // Use preset range
                info!(
                    "Using preset range: {}mm to {}mm ({:.1}m to {:.1}m)",
                    start_mm,
                    end_mm,
                    start_mm as f32 / 1000.0,
                    end_mm as f32 / 1000.0
                );
                (start_mm, end_mm)
            };

        // Calculate optimal profile based on actual range being used
        let optimal_profile = Self::calculate_optimal_profile(final_start_mm, final_end_mm);
        let optimal_step_length = Self::calculate_optimal_step_length(final_end_mm);

        info!(
            "🎯 Selected Profile {} for range {}mm-{}mm ({:.1}m-{:.1}m)",
            optimal_profile,
            final_start_mm,
            final_end_mm,
            final_start_mm as f32 / 1000.0,
            final_end_mm as f32 / 1000.0
        );

        info!("✅ Presence range parameters calculated");
        Ok((optimal_profile, optimal_step_length))
    }

    /// Configure thresholds and frame rate
    #[allow(clippy::too_many_arguments)]
    pub fn configure_thresholds(
        &mut self,
        intra_threshold: f32,
        inter_threshold: f32,
        frame_rate: f32,
        profile: u32,
        step_length: u32,
        auto_profile_enabled: bool,
        start_mm: u32,
        end_mm: u32,
    ) -> Result<()> {
        // Write threshold and frame rate configuration
        let intra_threshold_scaled = (intra_threshold * 1000.0) as u32;
        let inter_threshold_scaled = (inter_threshold * 1000.0) as u32;
        let frame_rate_scaled = (frame_rate * 1000.0) as u32;

        // CRITICAL: Write Start Point and End Point registers with custom range values
        info!(
            "Writing Start Point register (0x{:04X}): {}mm ({:.1}m)",
            PRESENCE_REG_START_ADDRESS,
            start_mm,
            start_mm as f32 / 1000.0
        );
        self.i2c
            .write_register(PRESENCE_REG_START_ADDRESS, &start_mm.to_be_bytes())?;

        info!(
            "Writing End Point register (0x{:04X}): {}mm ({:.1}m)",
            PRESENCE_REG_END_ADDRESS,
            end_mm,
            end_mm as f32 / 1000.0
        );
        self.i2c
            .write_register(PRESENCE_REG_END_ADDRESS, &end_mm.to_be_bytes())?;

        // Configure Auto Profile based on user preference
        if auto_profile_enabled {
            info!("✅ Enabling Auto Profile (firmware selects optimal profile based on range)");
            self.i2c
                .write_register(PRESENCE_REG_AUTO_PROFILE_ADDRESS, &1u32.to_be_bytes())?;
            self.i2c
                .write_register(PRESENCE_REG_AUTO_STEP_LENGTH_ADDRESS, &1u32.to_be_bytes())?;
        } else {
            info!(
                "🔧 Disabling Auto Profile (using manual Profile {} for 7m range)",
                profile
            );
            self.i2c
                .write_register(PRESENCE_REG_AUTO_PROFILE_ADDRESS, &0u32.to_be_bytes())?;
            self.i2c
                .write_register(PRESENCE_REG_AUTO_STEP_LENGTH_ADDRESS, &0u32.to_be_bytes())?;

            // Set manual profile and step length when auto is disabled
            info!(
                "Applying Manual Profile {} and Step Length {}",
                profile, step_length
            );
            self.i2c
                .write_register(PRESENCE_REG_MANUAL_PROFILE_ADDRESS, &profile.to_be_bytes())?;
            self.i2c.write_register(
                PRESENCE_REG_MANUAL_STEP_LENGTH_ADDRESS,
                &step_length.to_be_bytes(),
            )?;
        }

        // CRITICAL: Enable Auto Subsweeps (Philip's config: automatic_subsweeps: true)
        info!("Enabling Auto Subsweeps (matching Philip's working config)");
        self.i2c
            .write_register(PRESENCE_REG_AUTO_SUBSWEEPS_ADDRESS, &1u32.to_be_bytes())?;

        // Set HWAAS to Philip's value (Philip's config: hwaas: 32)
        let hwaas_philip = 32u32;
        info!(
            "Setting HWAAS to {} (matching Philip's working config)",
            hwaas_philip
        );
        self.i2c
            .write_register(PRESENCE_REG_HWAAS_ADDRESS, &hwaas_philip.to_be_bytes())?;

        // Set Signal Quality to Philip's value (Philip's config: signal_quality: 20.0)
        // Convert to proper units - Philip uses 20.0, which might be scaled differently
        let signal_quality_philip = (20.0 * 1000.0) as u32; // Scale to match register format
        info!(
            "Setting Signal Quality threshold to {} (matching Philip's working config: 20.0)",
            signal_quality_philip
        );
        self.i2c.write_register(
            PRESENCE_REG_SIGNAL_QUALITY_ADDRESS,
            &signal_quality_philip.to_be_bytes(),
        )?;

        // Write thresholds to registers (using datasheet register addresses)
        self.i2c.write_register(
            PRESENCE_REG_INTRA_DETECTION_THRESHOLD_ADDRESS,
            &intra_threshold_scaled.to_be_bytes(),
        )?;
        self.i2c.write_register(
            PRESENCE_REG_INTER_DETECTION_THRESHOLD_ADDRESS,
            &inter_threshold_scaled.to_be_bytes(),
        )?;
        self.i2c.write_register(
            PRESENCE_REG_FRAME_RATE_ADDRESS,
            &frame_rate_scaled.to_be_bytes(),
        )?;

        info!("✅ Thresholds and frame rate configured");
        Ok(())
    }

    /// Apply the complete configuration including range settings
    pub fn apply_complete_configuration(
        &mut self,
        final_start_mm: u32,
        final_end_mm: u32,
    ) -> Result<()> {
        // CRITICAL: Reset module before applying new configuration (from datasheet requirement)
        info!("Resetting presence detector module before configuration...");
        self.reset_module()?;

        // Wait for reset to complete
        info!("Waiting for module reset to complete...");
        self.wait_for_not_busy()?;

        // CRITICAL: Configure Auto Profile settings AFTER reset (reset wipes these settings)
        info!("Disabling Auto Profile and Auto Step Length AFTER reset");
        self.i2c
            .write_register(PRESENCE_REG_AUTO_PROFILE_ADDRESS, &0u32.to_be_bytes())?;
        self.i2c
            .write_register(PRESENCE_REG_AUTO_STEP_LENGTH_ADDRESS, &0u32.to_be_bytes())?;

        // Calculate and set optimal profile for 7m range
        let optimal_profile: u32 = if final_end_mm >= 6500 { 5 } else { 4 }; // Profile 5 for 7m
        let optimal_step_length = ((final_end_mm as f32 / 1000.0) * 1000.0 / 60.0) as u32;
        let optimal_step_length = optimal_step_length.clamp(12, 120);

        info!(
            "Setting Manual Profile {} and Step Length {} for {}mm range",
            optimal_profile, optimal_step_length, final_end_mm
        );
        self.i2c.write_register(
            PRESENCE_REG_MANUAL_PROFILE_ADDRESS,
            &optimal_profile.to_be_bytes(),
        )?;
        self.i2c.write_register(
            PRESENCE_REG_MANUAL_STEP_LENGTH_ADDRESS,
            &optimal_step_length.to_be_bytes(),
        )?;

        // Set Signal Quality to 20000 for long range
        let signal_quality = 20000u32;
        info!(
            "Setting Signal Quality to {} for long range detection",
            signal_quality
        );
        self.i2c.write_register(
            PRESENCE_REG_SIGNAL_QUALITY_ADDRESS,
            &signal_quality.to_be_bytes(),
        )?;

        // CRITICAL: Write range values LAST to prevent them being overwritten by profile settings
        info!(
            "Writing start range to register 0x{:04X} ({}): {} mm",
            PRESENCE_REG_START_ADDRESS, PRESENCE_REG_START_ADDRESS, final_start_mm
        );
        self.i2c
            .write_register(PRESENCE_REG_START_ADDRESS, &final_start_mm.to_be_bytes())?;

        info!(
            "Writing end range to register 0x{:04X} ({}): {} mm",
            PRESENCE_REG_END_ADDRESS, PRESENCE_REG_END_ADDRESS, final_end_mm
        );
        self.i2c
            .write_register(PRESENCE_REG_END_ADDRESS, &final_end_mm.to_be_bytes())?;

        info!("✅ Range configuration written to hardware registers");

        // CRITICAL: Apply configuration by writing CMD_PRESENCE_APPLY_CONFIGURATION to command register 0x0100
        // Without this step, detector uses default values (end point = 2500mm)
        info!("Applying presence detector configuration (CMD_PRESENCE_APPLY_CONFIGURATION to register 0x0100)");
        self.i2c.write_register(
            PRESENCE_REG_COMMAND_ADDRESS,
            &CMD_PRESENCE_APPLY_CONFIGURATION.to_be_bytes(),
        )?;

        // CRITICAL: Wait for the configuration to be done (from example code)
        info!("Waiting for configuration to complete...");
        self.wait_for_not_busy()?;

        // CRITICAL: Test if configuration of detector was OK (from example code)
        info!("Verifying configuration was applied successfully...");
        if !self.configuration_ok()? {
            return Err(RadarError::DeviceError {
                message:
                    "Configuration verification failed - detector did not accept the configuration"
                        .to_string(),
            });
        }
        info!("✅ Configuration verified successfully");

        // CRITICAL: Start the detector after configuration
        info!("Starting presence detector (CMD_PRESENCE_START_DETECTOR to register 0x0100)");
        self.i2c.write_register(
            PRESENCE_REG_COMMAND_ADDRESS,
            &CMD_PRESENCE_START_DETECTOR.to_be_bytes(),
        )?;

        info!("✅ Presence detector configured and started - full range should now be available");
        Ok(())
    }

    /// Wait for detector to not be busy (from example code)
    fn wait_for_not_busy(&mut self) -> Result<()> {
        let timeout = std::time::Duration::from_secs(5);
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            if !self.is_busy()? {
                return Ok(());
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        Err(RadarError::Timeout {
            timeout: timeout.as_secs(),
        })
    }

    /// Check if configuration was applied successfully (from example code)
    fn configuration_ok(&mut self) -> Result<bool> {
        // Read detector status to check for configuration success
        let status_data = self
            .i2c
            .read_register(PRESENCE_REG_DETECTOR_STATUS_ADDRESS, 4)?;
        let status = u32::from_be_bytes([
            status_data[0],
            status_data[1],
            status_data[2],
            status_data[3],
        ]);

        // Check if there are any error bits set (bit 28 and others)
        let has_errors = (status & 0x10000000) != 0; // Error bit

        if has_errors {
            warn!(
                "Configuration failed - detector status shows errors: 0x{:08X}",
                status
            );
            return Ok(false);
        }

        Ok(true)
    }

    /// Reset the presence detector module (needed to make a new configuration)
    fn reset_module(&mut self) -> Result<()> {
        info!(
            "Resetting presence detector module (CMD_PRESENCE_RESET_MODULE: {})...",
            CMD_PRESENCE_RESET_MODULE
        );
        self.i2c.write_register(
            PRESENCE_REG_COMMAND_ADDRESS,
            &CMD_PRESENCE_RESET_MODULE.to_be_bytes(),
        )?;

        // Wait a moment for reset to take effect
        std::thread::sleep(std::time::Duration::from_millis(200));

        Ok(())
    }

    /// Check if presence detector is busy (section 2.3.1 compliance)
    pub fn is_busy(&mut self) -> Result<bool> {
        let status = self
            .i2c
            .read_register(PRESENCE_REG_DETECTOR_STATUS_ADDRESS, 4)?;
        let status_value = u32::from_be_bytes([status[0], status[1], status[2], status[3]]);
        Ok((status_value & STATUS_BUSY_MASK) != 0)
    }

    /// Check if presence detector has errors (section 2.3.1 compliance)
    pub fn has_errors(&mut self) -> Result<bool> {
        let status = self
            .i2c
            .read_register(PRESENCE_REG_DETECTOR_STATUS_ADDRESS, 4)?;
        let status_value = u32::from_be_bytes([status[0], status[1], status[2], status[3]]);
        Ok((status_value & STATUS_ERROR_MASK) != 0)
    }

    /// Write command safely with busy/error checking (section 2.3.1 compliance)
    pub async fn write_command_safe(&mut self, command: u32) -> Result<()> {
        // Check if detector is busy before writing command
        if self.is_busy()? {
            self.wait_for_not_busy()?;
        }

        // Check for errors - if present, only RESET MODULE command is allowed
        if self.has_errors()? && command != CMD_RESET_MODULE {
            warn!("Presence detector has errors, resetting module before command");
            self.reset_module()?;
        }

        // Write the command
        self.i2c
            .write_register(PRESENCE_REG_COMMAND_ADDRESS, &command.to_be_bytes())?;
        Ok(())
    }

    /// Apply configuration (section 2.3.3 compliance)
    pub async fn apply_configuration(&mut self) -> Result<()> {
        info!("Applying presence detector configuration...");
        self.write_command_safe(CMD_PRESENCE_APPLY_CONFIGURATION)
            .await?;

        // Wait for configuration to be applied and check status
        self.wait_for_not_busy()?;

        // Check for configuration errors
        if self.has_errors()? {
            return Err(RadarError::DeviceError {
                message: "Presence detector configuration failed - check register settings"
                    .to_string(),
            });
        }

        info!("✅ Presence detector configured successfully");
        Ok(())
    }

    /// Start presence detector (section 2.3.4 compliance)
    pub async fn start_detector(&mut self) -> Result<()> {
        info!("▶️ Starting presence detector...");

        // Section 2.3.4: Start detector command with proper compliance
        self.write_command_safe(CMD_PRESENCE_START_DETECTOR).await?;

        // Wait for start command to complete
        self.wait_for_not_busy()?;

        // Check for start errors
        if self.has_errors()? {
            return Err(RadarError::DeviceError {
                message: "Failed to start presence detector - check configuration".to_string(),
            });
        }

        info!("✅ Presence detector started successfully");
        Ok(())
    }

    /// Stop presence detector (section 2.3.4 compliance)
    pub async fn stop_detector(&mut self) -> Result<()> {
        info!("⏹️ Stopping presence detector...");

        // Section 2.3.4: Stop detector command with proper compliance
        self.write_command_safe(CMD_PRESENCE_STOP_DETECTOR).await?;

        // Wait for stop command to complete
        self.wait_for_not_busy()?;

        info!("✅ Presence detector stopped successfully");
        Ok(())
    }

    fn read_u32_register(&mut self, address: u16) -> Result<u32> {
        let data = self.i2c.read_register(address, 4)?;
        Ok(u32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    fn register_milli_to_f32(raw: u32) -> f32 {
        raw as f32 / 1000.0
    }

    /// Measure presence detection
    pub async fn measure(&mut self) -> Result<PresenceMeasurement> {
        let presence_value = self.read_u32_register(REG_PRESENCE_RESULT)?;
        let distance_value = self.read_u32_register(REG_PRESENCE_DISTANCE)?;
        let intra_value = self.read_u32_register(REG_INTRA_PRESENCE_SCORE)?;
        let inter_value = self.read_u32_register(REG_INTER_PRESENCE_SCORE)?;
        let actual_frame_rate_raw = self.read_u32_register(PRESENCE_REG_ACTUAL_FRAME_RATE_ADDRESS)?;
        let start_raw = self.read_u32_register(PRESENCE_REG_START_ADDRESS)?;
        let end_raw = self.read_u32_register(PRESENCE_REG_END_ADDRESS)?;
        let intra_threshold_raw =
            self.read_u32_register(PRESENCE_REG_INTRA_DETECTION_THRESHOLD_ADDRESS)?;
        let inter_threshold_raw =
            self.read_u32_register(PRESENCE_REG_INTER_DETECTION_THRESHOLD_ADDRESS)?;

        let temp_bits = (presence_value & PRESENCE_RESULT_TEMPERATURE_MASK) >> 16;

        Ok(PresenceMeasurement {
            presence_detected: presence_value & PRESENCE_RESULT_DETECTED_MASK != 0,
            presence_sticky: presence_value & PRESENCE_RESULT_STICKY_MASK != 0,
            presence_distance: Self::register_milli_to_f32(distance_value),
            intra_presence_score: Self::register_milli_to_f32(intra_value),
            inter_presence_score: Self::register_milli_to_f32(inter_value),
            detector_error: presence_value & PRESENCE_RESULT_DETECTOR_ERROR_MASK != 0,
            internal_temperature_c: temp_bits as i16,
            actual_frame_rate_hz: Self::register_milli_to_f32(actual_frame_rate_raw),
            start_m: Self::register_milli_to_f32(start_raw),
            end_m: Self::register_milli_to_f32(end_raw),
            intra_threshold: Self::register_milli_to_f32(intra_threshold_raw),
            inter_threshold: Self::register_milli_to_f32(inter_threshold_raw),
            timestamp: chrono::Utc::now(),
        })
    }
}
