use std::process::Command;

fn dump(theme: &str, out: &std::path::Path) {
    let status = Command::new(env!("CARGO_BIN_EXE_podctl-popup"))
        .args(["--dump", out.to_str().unwrap(), "--theme", theme])
        .status()
        .expect("spawn podctl-popup");
    assert!(status.success(), "podctl-popup --dump exited {status}");
}

#[test]
fn dump_is_valid_png_and_deterministic() {
    let dir = std::env::temp_dir();
    let a = dir.join("podctl-popup-a.png");
    let b = dir.join("podctl-popup-b.png");

    dump("dark", &a);
    dump("dark", &b);

    let pa = std::fs::read(&a).unwrap();
    let pb = std::fs::read(&b).unwrap();

    assert_eq!(&pa[..8], b"\x89PNG\r\n\x1a\n", "not a PNG");
    assert!(
        pa.len() > 1500,
        "frame suspiciously small: {} bytes",
        pa.len()
    );
    assert!(
        pa.len() < 200_000,
        "frame suspiciously large: {} bytes",
        pa.len()
    );
    assert_eq!(pa, pb, "render is not deterministic across runs");

    let _ = std::fs::remove_file(&a);
    let _ = std::fs::remove_file(&b);
}

#[test]
fn dark_and_light_differ() {
    let dir = std::env::temp_dir();
    let d = dir.join("podctl-popup-dark.png");
    let l = dir.join("podctl-popup-light.png");

    dump("dark", &d);
    dump("light", &l);

    assert_ne!(
        std::fs::read(&d).unwrap(),
        std::fs::read(&l).unwrap(),
        "themes produced identical output"
    );

    let _ = std::fs::remove_file(&d);
    let _ = std::fs::remove_file(&l);
}
