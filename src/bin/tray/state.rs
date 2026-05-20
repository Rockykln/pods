use podctl::caps::Capabilities;
use podctl::model::{Battery, ConvAwareness, Mode};

#[derive(Clone, Debug, Default)]
pub struct TrayState {
    pub connected: bool,
    pub name: Option<String>,
    pub model: Option<String>,
    pub battery: Battery,
    pub mode: Option<Mode>,
    pub conv_awareness: Option<ConvAwareness>,
    pub capabilities: Capabilities,
    pub low_battery_threshold: u8,
}

impl TrayState {
    pub fn new(low_battery_threshold: u8) -> Self {
        Self {
            low_battery_threshold,
            ..Self::default()
        }
    }

    pub fn battery_min_buds(&self) -> Option<u8> {
        [self.battery.left, self.battery.right]
            .into_iter()
            .flatten()
            .min()
    }

    pub fn buds_charging(&self) -> bool {
        self.battery.left_charging || self.battery.right_charging
    }

    pub fn icon_name(&self) -> &'static str {
        // Always a headphones icon — never battery-*, which the theme
        // renders as the system's PC battery glyph. Charging / low state
        // is conveyed through the tooltip and the NeedsAttention status.
        if self.connected {
            "audio-headphones"
        } else {
            "audio-headphones-symbolic"
        }
    }

    pub fn status(&self) -> &'static str {
        if self.connected
            && self
                .battery_min()
                .is_some_and(|p| p < self.low_battery_threshold)
            && !self.any_charging()
        {
            "NeedsAttention"
        } else {
            "Active"
        }
    }

    pub fn title(&self) -> String {
        self.model
            .clone()
            .or_else(|| self.name.clone())
            .unwrap_or_else(|| "AirPods".into())
    }

    pub fn tooltip(&self) -> (String, String) {
        let title = self.title();
        if !self.connected {
            return (title, "Not connected".into());
        }
        let mut parts = Vec::with_capacity(3);
        if let Some(l) = self.battery.left {
            parts.push(format_part("L", l, self.battery.left_charging));
        }
        if let Some(r) = self.battery.right {
            parts.push(format_part("R", r, self.battery.right_charging));
        }
        if let Some(c) = self.battery.case {
            parts.push(format_part("Case", c, self.battery.case_charging));
        }
        let body = if parts.is_empty() {
            "Connected".into()
        } else {
            parts.join(" · ")
        };
        (title, body)
    }

    fn battery_min(&self) -> Option<u8> {
        [self.battery.left, self.battery.right]
            .into_iter()
            .flatten()
            .min()
    }

    pub fn any_charging(&self) -> bool {
        self.battery.left_charging || self.battery.right_charging || self.battery.case_charging
    }
}

fn format_part(label: &str, pct: u8, charging: bool) -> String {
    if charging {
        format!("{label} {pct}% ⚡")
    } else {
        format!("{label} {pct}%")
    }
}
