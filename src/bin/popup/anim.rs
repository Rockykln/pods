use std::time::Duration;

pub const FRAME: Duration = Duration::from_millis(16);

pub fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let u = 1.0 - t;
    1.0 - u * u * u
}

/// Discrete slide between two pixel offsets over `anim_ms`, ~60 fps.
/// `rev` plays the same curve backwards for the exit.
pub struct Slide {
    from: f32,
    to: f32,
    frames: u32,
    i: u32,
}

impl Slide {
    pub fn new(from: f32, to: f32, anim_ms: u32) -> Self {
        let frames = (anim_ms / 16).max(1);
        Slide {
            from,
            to,
            frames,
            i: 0,
        }
    }
}

impl Iterator for Slide {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.i > self.frames {
            return None;
        }
        let t = self.i as f32 / self.frames as f32;
        self.i += 1;
        Some(self.from + (self.to - self.from) * ease_out_cubic(t))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ease_endpoints() {
        assert!(ease_out_cubic(0.0).abs() < 1e-6);
        assert!((ease_out_cubic(1.0) - 1.0).abs() < 1e-6);
        assert!(ease_out_cubic(-5.0).abs() < 1e-6);
        assert!((ease_out_cubic(9.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn slide_hits_both_ends() {
        let v: Vec<f32> = Slide::new(100.0, 0.0, 200).collect();
        assert!((v.first().copied().unwrap() - 100.0).abs() < 1e-3);
        assert!((v.last().copied().unwrap() - 0.0).abs() < 1e-3);
        assert!(v.len() >= 12);
    }
}
