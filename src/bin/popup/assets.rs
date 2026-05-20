use tiny_skia::{Path, PathBuilder, Rect};

pub static UI_FONT: &[u8] = include_bytes!("../../../assets/fonts/NotoSans-Regular.ttf");

pub struct Earbud {
    pub bud: Path,
    pub stem: Path,
    pub tip: Path,
    pub nozzle: Path,
    pub sensor: Path,
    pub miccap: Path,
}

/// One AirPods-style earbud: a round bud plus a thin stem offset toward
/// the front, drawn as two overlapping shapes (same fill colour, so the
/// overlap is invisible). Built in a unit box, scaled by the caller.
/// Own geometry, no Apple asset.
pub fn earbud(scale: f32, ox: f32, oy: f32) -> Earbud {
    let x = |v: f32| ox + v * scale;
    let y = |v: f32| oy + v * scale;
    let u = |v: f32| v * scale;

    let mut bud = PathBuilder::new();
    bud.push_oval(Rect::from_xywh(x(0.00), y(0.00), u(0.64), u(0.62)).unwrap());

    // Stem: rounded-bottom bar, its top tucked inside the bud.
    let sx0 = x(0.30);
    let sx1 = x(0.50);
    let st = y(0.40);
    let sb = y(0.98);
    let r = u(0.10);
    let mut stem = PathBuilder::new();
    stem.move_to(sx0, st);
    stem.line_to(sx1, st);
    stem.line_to(sx1, sb - r);
    stem.quad_to(sx1, sb, sx1 - r, sb);
    stem.line_to(sx0 + r, sb);
    stem.quad_to(sx0, sb, sx0, sb - r);
    stem.close();

    // Matte silicone ear-tip: the top third of the bud, a softer shade
    // than the glossy body. This is the AirPods Pro signature.
    let mut tip = PathBuilder::new();
    tip.push_oval(Rect::from_xywh(x(0.13), y(0.00), u(0.40), u(0.34)).unwrap());

    // Sound outlet: a fine horizontal vent slit in the tip.
    let mut nozzle = PathBuilder::new();
    nozzle.push_oval(Rect::from_xywh(x(0.27), y(0.10), u(0.12), u(0.06)).unwrap());

    // Force-sensor: a thin recessed stripe down the front of the stem.
    let mut sensor = PathBuilder::new();
    let cxs = 0.5 * (x(0.30) + x(0.50));
    sensor.push_oval(Rect::from_xywh(cxs - u(0.035), y(0.52), u(0.07), u(0.22)).unwrap());

    // Microphone end: the lowest part of the stem, a touch darker.
    let mut miccap = PathBuilder::new();
    let r = u(0.10);
    let (sx0, sx1, sb) = (x(0.30), x(0.50), y(0.98));
    miccap.move_to(sx0, y(0.90));
    miccap.line_to(sx1, y(0.90));
    miccap.line_to(sx1, sb - r);
    miccap.quad_to(sx1, sb, sx1 - r, sb);
    miccap.line_to(sx0 + r, sb);
    miccap.quad_to(sx0, sb, sx0, sb - r);
    miccap.close();

    Earbud {
        bud: bud.finish().unwrap(),
        stem: stem.finish().unwrap(),
        tip: tip.finish().unwrap(),
        nozzle: nozzle.finish().unwrap(),
        sensor: sensor.finish().unwrap(),
        miccap: miccap.finish().unwrap(),
    }
}

/// Lightning bolt for the charging overlay, centred on (cx, cy).
pub fn bolt(size: f32, cx: f32, cy: f32) -> Path {
    let s = size;
    let x = cx - s * 0.42;
    let y = cy - s * 0.5;
    let mut pb = PathBuilder::new();
    pb.move_to(x + 0.52 * s, y);
    pb.line_to(x + 0.10 * s, y + 0.60 * s);
    pb.line_to(x + 0.40 * s, y + 0.60 * s);
    pb.line_to(x + 0.32 * s, y + s);
    pb.line_to(x + 0.84 * s, y + 0.38 * s);
    pb.line_to(x + 0.50 * s, y + 0.38 * s);
    pb.close();
    pb.finish().unwrap()
}
