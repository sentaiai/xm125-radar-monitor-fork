// I2C register snapshot for i2c_presence_detector.bin firmware
// Register map: presence_reg_protocol.h (Acconeer SDK v1.13)

use super::registers::{
    PRESENCE_APP_ID, PRESENCE_REG_ACTUAL_FRAME_RATE_ADDRESS,
    PRESENCE_REG_AUTO_PROFILE_ADDRESS, PRESENCE_REG_AUTO_STEP_LENGTH_ADDRESS,
    PRESENCE_REG_AUTO_SUBSWEEPS_ADDRESS, PRESENCE_REG_DETECTION_ON_GPIO_ADDRESS,
    PRESENCE_REG_END_ADDRESS, PRESENCE_REG_FRAME_RATE_ADDRESS,
    PRESENCE_REG_HWAAS_ADDRESS, PRESENCE_REG_INTER_DETECTION_ENABLED_ADDRESS,
    PRESENCE_REG_INTER_DETECTION_THRESHOLD_ADDRESS,
    PRESENCE_REG_INTER_FRAME_DEVIATION_TIME_CONST_ADDRESS,
    PRESENCE_REG_INTER_FRAME_FAST_CUTOFF_ADDRESS,
    PRESENCE_REG_INTER_FRAME_PRESENCE_TIMEOUT_ADDRESS,
    PRESENCE_REG_INTER_FRAME_SLOW_CUTOFF_ADDRESS,
    PRESENCE_REG_INTER_OUTPUT_TIME_CONST_ADDRESS, PRESENCE_REG_INTRA_DETECTION_ENABLED_ADDRESS,
    PRESENCE_REG_INTRA_DETECTION_THRESHOLD_ADDRESS,
    PRESENCE_REG_INTRA_FRAME_TIME_CONST_ADDRESS, PRESENCE_REG_INTRA_OUTPUT_TIME_CONST_ADDRESS,
    PRESENCE_REG_MANUAL_PROFILE_ADDRESS, PRESENCE_REG_MANUAL_STEP_LENGTH_ADDRESS,
    PRESENCE_REG_RESET_FILTERS_ON_PREPARE_ADDRESS, PRESENCE_REG_SIGNAL_QUALITY_ADDRESS,
    PRESENCE_REG_START_ADDRESS, PRESENCE_REG_SWEEPS_PER_FRAME_ADDRESS,
    PRESENCE_RESULT_DETECTED_MASK, PRESENCE_RESULT_DETECTOR_ERROR_MASK,
    PRESENCE_RESULT_STICKY_MASK, PRESENCE_RESULT_TEMPERATURE_MASK, PRESENCE_STATUS_BUSY,
    PRESENCE_STATUS_CONFIG_APPLY_ERROR, PRESENCE_STATUS_CONFIG_APPLY_OK,
    PRESENCE_STATUS_CONFIG_CREATE_ERROR, PRESENCE_STATUS_CONFIG_CREATE_OK,
    PRESENCE_STATUS_DETECTOR_BUFFER_ERROR, PRESENCE_STATUS_DETECTOR_BUFFER_OK,
    PRESENCE_STATUS_DETECTOR_CREATE_ERROR, PRESENCE_STATUS_DETECTOR_CREATE_OK,
    PRESENCE_STATUS_DETECTOR_ERROR, PRESENCE_STATUS_RSS_REGISTER_ERROR,
    PRESENCE_STATUS_RSS_REGISTER_OK, PRESENCE_STATUS_SENSOR_BUFFER_ERROR,
    PRESENCE_STATUS_SENSOR_BUFFER_OK, PRESENCE_STATUS_SENSOR_CALIBRATE_ERROR,
    PRESENCE_STATUS_SENSOR_CALIBRATE_OK, PRESENCE_STATUS_SENSOR_CREATE_ERROR,
    PRESENCE_STATUS_SENSOR_CREATE_OK, REG_APPLICATION_ID, REG_DETECTOR_STATUS,
    REG_INTER_PRESENCE_SCORE, REG_INTRA_PRESENCE_SCORE, REG_MEASURE_COUNTER,
    REG_PRESENCE_DISTANCE, REG_PRESENCE_RESULT, REG_PROTOCOL_STATUS, REG_VERSION,
};
use crate::error::Result;
use crate::i2c::I2cDevice;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct FirmwareVersion {
    #[serde(rename = "R_major")]
    pub major: u16,
    #[serde(rename = "R_minor")]
    pub minor: u8,
    #[serde(rename = "R_patch")]
    pub patch: u8,
    #[serde(rename = "R_raw")]
    pub raw: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct DetectorStatusFlags {
    #[serde(rename = "R_raw")]
    pub raw: u32,
    #[serde(rename = "R_busy")]
    pub busy: bool,
    #[serde(rename = "R_rss_register_ok")]
    pub rss_register_ok: bool,
    #[serde(rename = "R_config_create_ok")]
    pub config_create_ok: bool,
    #[serde(rename = "R_sensor_create_ok")]
    pub sensor_create_ok: bool,
    #[serde(rename = "R_sensor_calibrate_ok")]
    pub sensor_calibrate_ok: bool,
    #[serde(rename = "R_detector_create_ok")]
    pub detector_create_ok: bool,
    #[serde(rename = "R_detector_buffer_ok")]
    pub detector_buffer_ok: bool,
    #[serde(rename = "R_sensor_buffer_ok")]
    pub sensor_buffer_ok: bool,
    #[serde(rename = "R_config_apply_ok")]
    pub config_apply_ok: bool,
    #[serde(rename = "R_rss_register_error")]
    pub rss_register_error: bool,
    #[serde(rename = "R_config_create_error")]
    pub config_create_error: bool,
    #[serde(rename = "R_sensor_create_error")]
    pub sensor_create_error: bool,
    #[serde(rename = "R_sensor_calibrate_error")]
    pub sensor_calibrate_error: bool,
    #[serde(rename = "R_detector_create_error")]
    pub detector_create_error: bool,
    #[serde(rename = "R_detector_buffer_error")]
    pub detector_buffer_error: bool,
    #[serde(rename = "R_sensor_buffer_error")]
    pub sensor_buffer_error: bool,
    #[serde(rename = "R_config_apply_error")]
    pub config_apply_error: bool,
    #[serde(rename = "R_detector_error")]
    pub detector_error: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PresenceResults {
    #[serde(rename = "R_presence_detected")]
    pub presence_detected: bool,
    #[serde(rename = "R_presence_sticky")]
    pub presence_sticky: bool,
    #[serde(rename = "R_detector_error")]
    pub detector_error: bool,
    /// Internal A121 chip temperature (°C). Poor absolute accuracy — relative use only.
    #[serde(rename = "R_internal_temperature_c")]
    pub internal_temperature_c: i16,
    #[serde(rename = "R_presence_distance_m")]
    pub presence_distance_m: f32,
    #[serde(rename = "R_intra_presence_score")]
    pub intra_presence_score: f32,
    #[serde(rename = "R_inter_presence_score")]
    pub inter_presence_score: f32,
    #[serde(rename = "R_actual_frame_rate_hz")]
    pub actual_frame_rate_hz: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct PresenceConfiguration {
    #[serde(rename = "RW_sweeps_per_frame")]
    pub sweeps_per_frame: u32,
    #[serde(rename = "RW_inter_frame_presence_timeout_s")]
    pub inter_frame_presence_timeout_s: u32,
    #[serde(rename = "RW_intra_detection_enabled")]
    pub intra_detection_enabled: bool,
    #[serde(rename = "RW_inter_detection_enabled")]
    pub inter_detection_enabled: bool,
    #[serde(rename = "RW_frame_rate_hz")]
    pub frame_rate_hz: f32,
    #[serde(rename = "RW_intra_detection_threshold")]
    pub intra_detection_threshold: f32,
    #[serde(rename = "RW_inter_detection_threshold")]
    pub inter_detection_threshold: f32,
    #[serde(rename = "RW_inter_frame_deviation_time_const")]
    pub inter_frame_deviation_time_const: u32,
    #[serde(rename = "RW_inter_frame_fast_cutoff")]
    pub inter_frame_fast_cutoff: u32,
    #[serde(rename = "RW_inter_frame_slow_cutoff")]
    pub inter_frame_slow_cutoff: u32,
    #[serde(rename = "RW_intra_frame_time_const")]
    pub intra_frame_time_const: u32,
    #[serde(rename = "RW_intra_output_time_const")]
    pub intra_output_time_const: u32,
    #[serde(rename = "RW_inter_output_time_const")]
    pub inter_output_time_const: u32,
    #[serde(rename = "RW_auto_profile_enabled")]
    pub auto_profile_enabled: bool,
    #[serde(rename = "RW_auto_step_length_enabled")]
    pub auto_step_length_enabled: bool,
    #[serde(rename = "RW_manual_profile")]
    pub manual_profile: u32,
    #[serde(rename = "RW_manual_step_length")]
    pub manual_step_length: u32,
    #[serde(rename = "RW_start_mm")]
    pub start_mm: u32,
    #[serde(rename = "RW_end_mm")]
    pub end_mm: u32,
    #[serde(rename = "R_start_m")]
    pub start_m: f32,
    #[serde(rename = "R_end_m")]
    pub end_m: f32,
    #[serde(rename = "RW_reset_filters_on_prepare")]
    pub reset_filters_on_prepare: bool,
    #[serde(rename = "RW_hwaas")]
    pub hwaas: u32,
    #[serde(rename = "RW_automatic_subsweeps")]
    pub automatic_subsweeps: bool,
    #[serde(rename = "RW_signal_quality")]
    pub signal_quality: u32,
    #[serde(rename = "RW_detection_on_gpio")]
    pub detection_on_gpio: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PresenceRegisterSnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "R_application_id")]
    pub application_id: u32,
    #[serde(rename = "R_application_name")]
    pub application_name: &'static str,
    pub version: FirmwareVersion,
    #[serde(rename = "R_protocol_status")]
    pub protocol_status: u32,
    #[serde(rename = "R_measure_counter")]
    pub measure_counter: u32,
    pub detector_status: DetectorStatusFlags,
    pub results: PresenceResults,
    pub configuration: PresenceConfiguration,
}

fn read_reg(i2c: &mut I2cDevice, address: u16) -> Result<u32> {
    let data = i2c.read_register(address, 4)?;
    Ok(u32::from_be_bytes([data[0], data[1], data[2], data[3]]))
}

fn parse_version(raw: u32) -> FirmwareVersion {
    FirmwareVersion {
        major: ((raw >> 16) & 0xFFFF) as u16,
        minor: ((raw >> 8) & 0xFF) as u8,
        patch: (raw & 0xFF) as u8,
        raw,
    }
}

fn parse_detector_status(raw: u32) -> DetectorStatusFlags {
    DetectorStatusFlags {
        busy: raw & PRESENCE_STATUS_BUSY != 0,
        rss_register_ok: raw & PRESENCE_STATUS_RSS_REGISTER_OK != 0,
        config_create_ok: raw & PRESENCE_STATUS_CONFIG_CREATE_OK != 0,
        sensor_create_ok: raw & PRESENCE_STATUS_SENSOR_CREATE_OK != 0,
        sensor_calibrate_ok: raw & PRESENCE_STATUS_SENSOR_CALIBRATE_OK != 0,
        detector_create_ok: raw & PRESENCE_STATUS_DETECTOR_CREATE_OK != 0,
        detector_buffer_ok: raw & PRESENCE_STATUS_DETECTOR_BUFFER_OK != 0,
        sensor_buffer_ok: raw & PRESENCE_STATUS_SENSOR_BUFFER_OK != 0,
        config_apply_ok: raw & PRESENCE_STATUS_CONFIG_APPLY_OK != 0,
        rss_register_error: raw & PRESENCE_STATUS_RSS_REGISTER_ERROR != 0,
        config_create_error: raw & PRESENCE_STATUS_CONFIG_CREATE_ERROR != 0,
        sensor_create_error: raw & PRESENCE_STATUS_SENSOR_CREATE_ERROR != 0,
        sensor_calibrate_error: raw & PRESENCE_STATUS_SENSOR_CALIBRATE_ERROR != 0,
        detector_create_error: raw & PRESENCE_STATUS_DETECTOR_CREATE_ERROR != 0,
        detector_buffer_error: raw & PRESENCE_STATUS_DETECTOR_BUFFER_ERROR != 0,
        sensor_buffer_error: raw & PRESENCE_STATUS_SENSOR_BUFFER_ERROR != 0,
        config_apply_error: raw & PRESENCE_STATUS_CONFIG_APPLY_ERROR != 0,
        detector_error: raw & PRESENCE_STATUS_DETECTOR_ERROR != 0,
        raw,
    }
}

fn milli_to_f32(raw: u32) -> f32 {
    raw as f32 / 1000.0
}

fn mhz_to_hz(raw: u32) -> f32 {
    raw as f32 / 1000.0
}

/// Read all I2C registers exposed by `i2c_presence_detector.bin`.
pub fn read_presence_registers(i2c: &mut I2cDevice) -> Result<PresenceRegisterSnapshot> {
    let version_raw = read_reg(i2c, REG_VERSION)?;
    let protocol_status = read_reg(i2c, REG_PROTOCOL_STATUS)?;
    let measure_counter = read_reg(i2c, REG_MEASURE_COUNTER)?;
    let detector_status_raw = read_reg(i2c, REG_DETECTOR_STATUS)?;
    let application_id = read_reg(i2c, REG_APPLICATION_ID)?;

    let presence_result = read_reg(i2c, REG_PRESENCE_RESULT)?;
    let presence_distance_raw = read_reg(i2c, REG_PRESENCE_DISTANCE)?;
    let intra_raw = read_reg(i2c, REG_INTRA_PRESENCE_SCORE)?;
    let inter_raw = read_reg(i2c, REG_INTER_PRESENCE_SCORE)?;
    let actual_frame_rate_raw = read_reg(i2c, PRESENCE_REG_ACTUAL_FRAME_RATE_ADDRESS)?;

    let temp_bits = (presence_result & PRESENCE_RESULT_TEMPERATURE_MASK) >> 16;
    let internal_temperature_c = temp_bits as i16;

    let start_mm = read_reg(i2c, PRESENCE_REG_START_ADDRESS)?;
    let end_mm = read_reg(i2c, PRESENCE_REG_END_ADDRESS)?;

    let configuration = PresenceConfiguration {
        sweeps_per_frame: read_reg(i2c, PRESENCE_REG_SWEEPS_PER_FRAME_ADDRESS)?,
        inter_frame_presence_timeout_s: read_reg(
            i2c,
            PRESENCE_REG_INTER_FRAME_PRESENCE_TIMEOUT_ADDRESS,
        )?,
        intra_detection_enabled: read_reg(i2c, PRESENCE_REG_INTRA_DETECTION_ENABLED_ADDRESS)? != 0,
        inter_detection_enabled: read_reg(i2c, PRESENCE_REG_INTER_DETECTION_ENABLED_ADDRESS)? != 0,
        frame_rate_hz: mhz_to_hz(read_reg(i2c, PRESENCE_REG_FRAME_RATE_ADDRESS)?),
        intra_detection_threshold: milli_to_f32(read_reg(
            i2c,
            PRESENCE_REG_INTRA_DETECTION_THRESHOLD_ADDRESS,
        )?),
        inter_detection_threshold: milli_to_f32(read_reg(
            i2c,
            PRESENCE_REG_INTER_DETECTION_THRESHOLD_ADDRESS,
        )?),
        inter_frame_deviation_time_const: read_reg(
            i2c,
            PRESENCE_REG_INTER_FRAME_DEVIATION_TIME_CONST_ADDRESS,
        )?,
        inter_frame_fast_cutoff: read_reg(i2c, PRESENCE_REG_INTER_FRAME_FAST_CUTOFF_ADDRESS)?,
        inter_frame_slow_cutoff: read_reg(i2c, PRESENCE_REG_INTER_FRAME_SLOW_CUTOFF_ADDRESS)?,
        intra_frame_time_const: read_reg(i2c, PRESENCE_REG_INTRA_FRAME_TIME_CONST_ADDRESS)?,
        intra_output_time_const: read_reg(i2c, PRESENCE_REG_INTRA_OUTPUT_TIME_CONST_ADDRESS)?,
        inter_output_time_const: read_reg(i2c, PRESENCE_REG_INTER_OUTPUT_TIME_CONST_ADDRESS)?,
        auto_profile_enabled: read_reg(i2c, PRESENCE_REG_AUTO_PROFILE_ADDRESS)? != 0,
        auto_step_length_enabled: read_reg(i2c, PRESENCE_REG_AUTO_STEP_LENGTH_ADDRESS)? != 0,
        manual_profile: read_reg(i2c, PRESENCE_REG_MANUAL_PROFILE_ADDRESS)?,
        manual_step_length: read_reg(i2c, PRESENCE_REG_MANUAL_STEP_LENGTH_ADDRESS)?,
        start_mm,
        end_mm,
        start_m: start_mm as f32 / 1000.0,
        end_m: end_mm as f32 / 1000.0,
        reset_filters_on_prepare: read_reg(i2c, PRESENCE_REG_RESET_FILTERS_ON_PREPARE_ADDRESS)?
            != 0,
        hwaas: read_reg(i2c, PRESENCE_REG_HWAAS_ADDRESS)?,
        automatic_subsweeps: read_reg(i2c, PRESENCE_REG_AUTO_SUBSWEEPS_ADDRESS)? != 0,
        signal_quality: read_reg(i2c, PRESENCE_REG_SIGNAL_QUALITY_ADDRESS)?,
        detection_on_gpio: read_reg(i2c, PRESENCE_REG_DETECTION_ON_GPIO_ADDRESS)? != 0,
    };

    let application_name = if application_id == PRESENCE_APP_ID {
        "presence_detector"
    } else {
        "unknown"
    };

    Ok(PresenceRegisterSnapshot {
        timestamp: chrono::Utc::now(),
        application_id,
        application_name,
        version: parse_version(version_raw),
        protocol_status,
        measure_counter,
        detector_status: parse_detector_status(detector_status_raw),
        results: PresenceResults {
            presence_detected: presence_result & PRESENCE_RESULT_DETECTED_MASK != 0,
            presence_sticky: presence_result & PRESENCE_RESULT_STICKY_MASK != 0,
            detector_error: presence_result & PRESENCE_RESULT_DETECTOR_ERROR_MASK != 0,
            internal_temperature_c,
            presence_distance_m: milli_to_f32(presence_distance_raw),
            intra_presence_score: milli_to_f32(intra_raw),
            inter_presence_score: milli_to_f32(inter_raw),
            actual_frame_rate_hz: mhz_to_hz(actual_frame_rate_raw),
        },
        configuration,
    })
}
