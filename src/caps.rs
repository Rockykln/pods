use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Model {
    Gen1,
    Gen2,
    Gen3,
    Gen4,
    Gen4Anc,
    Pro,
    Pro2,
    Pro2UsbC,
    Pro3,
    Max,
    MaxUsbC,
    Max2,
    Unknown,
}

impl Model {
    /// Bluetooth Modalias product code → model. 0x2024 (Pro 2 USB-C) is
    /// hardware-verified; the rest cross-referenced against The Apple Wiki
    /// Bluetooth PIDs page and OpenPods' BLE proximity-pair table.
    ///
    /// AirPods 4 (non-ANC) is intentionally not mapped — no public PID has
    /// surfaced yet. An unknown code still detects as AirPods via the BlueZ
    /// icon and falls back to `Capabilities::conservative`, so the bud
    /// still shows up; only model-specific features are gated.
    pub fn from_code(code: u16) -> Self {
        match code {
            0x2002 => Model::Gen1,
            0x200F => Model::Gen2,
            0x2013 => Model::Gen3,
            0x201B => Model::Gen4Anc,
            0x200E => Model::Pro,
            0x2014 => Model::Pro2,
            0x2024 => Model::Pro2UsbC,
            0x2027 => Model::Pro3,
            0x200A => Model::Max,
            0x201F => Model::MaxUsbC,
            0x202D => Model::Max2,
            _ => Model::Unknown,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Model::Gen1 => "AirPods (1st gen)",
            Model::Gen2 => "AirPods (2nd gen)",
            Model::Gen3 => "AirPods (3rd gen)",
            Model::Gen4 => "AirPods 4",
            Model::Gen4Anc => "AirPods 4 (ANC)",
            Model::Pro => "AirPods Pro",
            Model::Pro2 => "AirPods Pro (2nd gen)",
            Model::Pro2UsbC => "AirPods Pro (2nd gen, USB-C)",
            Model::Pro3 => "AirPods Pro (3rd gen)",
            Model::Max => "AirPods Max",
            Model::MaxUsbC => "AirPods Max (USB-C)",
            Model::Max2 => "AirPods Max (2nd gen)",
            Model::Unknown => "AirPods (unknown model)",
        }
    }

    pub fn capabilities(self) -> Capabilities {
        match self {
            Model::Gen1 | Model::Gen2 | Model::Gen3 | Model::Gen4 => Capabilities {
                model: self,
                has_anc: false,
                has_transparency: false,
                has_adaptive: false,
                has_conv_awareness: false,
                has_case_battery: true,
                has_in_ear_detection: true,
                has_ear_detection_setting: true,
                has_loud_sound_reduction: false,
                has_spatial_audio: matches!(self, Model::Gen3 | Model::Gen4),
                has_press_volume: false,
                has_press_and_hold: false,
                has_mic_selection: true,
                has_rename: true,
                has_tone_on_press: false,
            },
            Model::Gen4Anc => Capabilities {
                model: self,
                has_anc: true,
                has_transparency: true,
                has_adaptive: true,
                has_conv_awareness: true,
                has_case_battery: true,
                has_in_ear_detection: true,
                has_ear_detection_setting: true,
                has_loud_sound_reduction: true,
                has_spatial_audio: true,
                has_press_volume: false,
                has_press_and_hold: true,
                has_mic_selection: true,
                has_rename: true,
                has_tone_on_press: true,
            },
            Model::Pro => Capabilities {
                model: self,
                has_anc: true,
                has_transparency: true,
                has_adaptive: false,
                has_conv_awareness: false,
                has_case_battery: true,
                has_in_ear_detection: true,
                has_ear_detection_setting: true,
                has_loud_sound_reduction: false,
                has_spatial_audio: true,
                has_press_volume: false,
                has_press_and_hold: true,
                has_mic_selection: true,
                has_rename: true,
                has_tone_on_press: true,
            },
            Model::Pro2 | Model::Pro2UsbC | Model::Pro3 => Capabilities {
                model: self,
                has_anc: true,
                has_transparency: true,
                has_adaptive: true,
                has_conv_awareness: true,
                has_case_battery: true,
                has_in_ear_detection: true,
                has_ear_detection_setting: true,
                has_loud_sound_reduction: true,
                has_spatial_audio: true,
                has_press_volume: true,
                has_press_and_hold: true,
                has_mic_selection: true,
                has_rename: true,
                has_tone_on_press: true,
            },
            Model::Max | Model::MaxUsbC | Model::Max2 => Capabilities {
                model: self,
                has_anc: true,
                has_transparency: true,
                // Adaptive (listening mode + Adaptive Audio Noise) is
                // AirPods Pro 2 / AirPods 4 ANC only per the documented
                // AAP protocol — AirPods Max has ANC + Transparency only.
                has_adaptive: false,
                has_conv_awareness: false,
                has_case_battery: false,
                has_in_ear_detection: false,
                has_ear_detection_setting: false,
                has_loud_sound_reduction: true,
                has_spatial_audio: true,
                has_press_volume: false,
                has_press_and_hold: true,
                has_mic_selection: false,
                has_rename: true,
                has_tone_on_press: false,
            },
            Model::Unknown => Capabilities::conservative(self),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Capabilities {
    pub model: Model,
    pub has_anc: bool,
    pub has_transparency: bool,
    pub has_adaptive: bool,
    pub has_conv_awareness: bool,
    pub has_case_battery: bool,
    pub has_in_ear_detection: bool,
    pub has_ear_detection_setting: bool,
    pub has_loud_sound_reduction: bool,
    pub has_spatial_audio: bool,
    pub has_press_volume: bool,
    pub has_press_and_hold: bool,
    pub has_mic_selection: bool,
    pub has_rename: bool,
    pub has_tone_on_press: bool,
}

impl Capabilities {
    pub fn conservative(model: Model) -> Self {
        Self {
            model,
            has_anc: false,
            has_transparency: false,
            has_adaptive: false,
            has_conv_awareness: false,
            has_case_battery: true,
            has_in_ear_detection: false,
            has_ear_detection_setting: false,
            has_loud_sound_reduction: false,
            has_spatial_audio: false,
            has_press_volume: false,
            has_press_and_hold: false,
            has_mic_selection: false,
            has_rename: true,
            has_tone_on_press: false,
        }
    }
}

impl Default for Capabilities {
    fn default() -> Self {
        Capabilities::conservative(Model::Unknown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_product_codes_resolve_to_their_model() {
        assert_eq!(Model::from_code(0x2002), Model::Gen1);
        assert_eq!(Model::from_code(0x200F), Model::Gen2);
        assert_eq!(Model::from_code(0x2013), Model::Gen3);
        assert_eq!(Model::from_code(0x200E), Model::Pro);
        assert_eq!(Model::from_code(0x2014), Model::Pro2);
        assert_eq!(Model::from_code(0x2024), Model::Pro2UsbC);
        assert_eq!(Model::from_code(0x2027), Model::Pro3);
        assert_eq!(Model::from_code(0x201B), Model::Gen4Anc);
        assert_eq!(Model::from_code(0x200A), Model::Max);
        assert_eq!(Model::from_code(0x201F), Model::MaxUsbC);
        assert_eq!(Model::from_code(0x202D), Model::Max2);
    }

    #[test]
    fn beats_pids_are_not_misread_as_airpods() {
        // 0x2025 is Beats Solo 4, 0x2026 is Beats Solo Buds. They were
        // previously (and wrongly) mapped to AirPods 4 / 4 ANC. Make sure
        // neither resolves to a known AirPods model anymore.
        assert_eq!(Model::from_code(0x2025), Model::Unknown);
        assert_eq!(Model::from_code(0x2026), Model::Unknown);
    }

    #[test]
    fn pro3_inherits_pro2_capabilities() {
        let caps = Model::Pro3.capabilities();
        assert!(caps.has_anc);
        assert!(caps.has_adaptive);
        assert!(caps.has_conv_awareness);
        assert!(caps.has_loud_sound_reduction);
    }

    #[test]
    fn max2_uses_max_capabilities() {
        let caps = Model::Max2.capabilities();
        assert!(caps.has_anc);
        assert!(!caps.has_case_battery);
        assert!(!caps.has_in_ear_detection);
    }
}
