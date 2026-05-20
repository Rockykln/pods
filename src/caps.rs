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
    Max,
    MaxUsbC,
    Unknown,
}

impl Model {
    /// Bluetooth Modalias product code → model. 0x2024 (Pro 2 USB-C) is
    /// hardware-verified; the rest are from public reverse-engineering.
    /// The AirPods 4 codes (0x2025/0x2026) are unconfirmed — an unknown
    /// code still detects as AirPods (icon) and falls back to
    /// `Capabilities::conservative`, so a wrong guess degrades safely.
    pub fn from_code(code: u16) -> Self {
        match code {
            0x2002 => Model::Gen1,
            0x200F => Model::Gen2,
            0x2013 => Model::Gen3,
            0x2025 => Model::Gen4,
            0x2026 => Model::Gen4Anc,
            0x200E => Model::Pro,
            0x2014 => Model::Pro2,
            0x2024 => Model::Pro2UsbC,
            0x200A => Model::Max,
            0x201F => Model::MaxUsbC,
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
            Model::Max => "AirPods Max",
            Model::MaxUsbC => "AirPods Max (USB-C)",
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
            Model::Pro2 | Model::Pro2UsbC => Capabilities {
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
            Model::Max | Model::MaxUsbC => Capabilities {
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
