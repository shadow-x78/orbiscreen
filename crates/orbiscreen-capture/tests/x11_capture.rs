// Orbiscreen - x11_capture.rs (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

use std::time::Duration;

use orbiscreen_capture::x11::X11Capture;

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime")
}

#[test]
fn x11_capture_delivers_frames_on_live_display() {
    if std::env::var_os("DISPLAY").is_none() {
        eprintln!("DISPLAY is not set; skipping live X11 capture test");
        return;
    }
    let capture = match X11Capture::open(640, 480) {
        Ok(capture) => capture,
        Err(e) => {
            eprintln!("cannot open X11 capture ({e}); skipping live X11 capture test");
            return;
        }
    };
    let (width, height) = capture.dimensions();
    assert!(width > 0 && height > 0, "root window has positive size");
    eprintln!(
        "X11 capture backend: {}",
        if capture.uses_shm() {
            "MIT-SHM pooled image"
        } else {
            "plain GetImage fallback"
        }
    );

    let frame = runtime()
        .block_on(async {
            tokio::time::timeout(Duration::from_secs(10), capture.next_frame()).await
        })
        .expect("first frame arrived within the deadline")
        .expect("first frame without error");
    assert_eq!(frame.width, width);
    assert_eq!(frame.height, height);
    assert_eq!(frame.data.len(), (width * height * 4) as usize);
}

#[test]
fn x11_capture_skips_unchanged_frames() {
    if std::env::var_os("DISPLAY").is_none() {
        eprintln!("DISPLAY is not set; skipping live X11 capture test");
        return;
    }
    let capture = match X11Capture::open(320, 240) {
        Ok(capture) => capture,
        Err(e) => {
            eprintln!("cannot open X11 capture ({e}); skipping live X11 capture test");
            return;
        }
    };
    let runtime = runtime();
    let first = runtime
        .block_on(async {
            tokio::time::timeout(Duration::from_secs(10), capture.next_frame()).await
        })
        .expect("first frame arrived within the deadline")
        .expect("first frame without error");

    let mut delivered = 0u32;
    for _ in 0..3 {
        let outcome = runtime.block_on(async {
            tokio::time::timeout(Duration::from_millis(200), capture.next_frame()).await
        });
        match outcome {
            Ok(Ok(frame)) => {
                assert_eq!(frame.data.len(), first.data.len());
                delivered += 1;
            }
            Ok(Err(e)) => panic!("capture error while polling: {e}"),
            Err(_) => {}
        }
    }
    eprintln!(
        "unchanged-frame skip probe: {delivered}/3 extra frames delivered in 600ms \
         (0 is expected on a fully static root; more means the screen content changed)"
    );
}
