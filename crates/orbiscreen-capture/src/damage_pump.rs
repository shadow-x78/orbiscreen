// Orbiscreen - orbiscreen-capture - damage pump module (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen
use std::io::Write as _;
use std::time::Duration;

use std::os::fd::AsFd as _;
use wayland_client::protocol::{
    wl_buffer, wl_compositor, wl_output, wl_region, wl_registry, wl_shm, wl_shm_pool, wl_surface,
};
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::Anchor;
use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1};

const OUTPUT_HINT: &str = "ORBISCREEN";

pub(crate) fn spawn(period: Duration) {
    let _ = std::thread::Builder::new()
        .name("orbiscreen-damage".into())
        .spawn(move || {
            if let Err(e) = run(period) {
                tracing::warn!("damage pump disabled: {e}");
            }
        });
}

fn run(period: Duration) -> Result<(), String> {
    let conn = Connection::connect_to_env().map_err(|e| format!("wayland connect: {e}"))?;
    let (globals, mut queue) = wayland_client::globals::registry_queue_init::<PumpState>(&conn)
        .map_err(|e| format!("registry: {e}"))?;
    let qh = queue.handle();
    let mut state = PumpState::default();

    let compositor: wl_compositor::WlCompositor = globals
        .bind(&qh, 4..=5, ())
        .map_err(|e| format!("compositor: {e}"))?;
    let shm: wl_shm::WlShm = globals
        .bind(&qh, 1..=1, ())
        .map_err(|e| format!("shm: {e}"))?;
    let layer_shell: zwlr_layer_shell_v1::ZwlrLayerShellV1 = globals
        .bind(&qh, 1..=3, ())
        .map_err(|e| format!("layer shell unsupported: {e}"))?;

    for output_global in globals
        .contents()
        .clone_list()
        .into_iter()
        .filter(|g| g.interface == "wl_output")
    {
        let _: wl_output::WlOutput =
            globals
                .registry()
                .bind(output_global.name, output_global.version.min(4), &qh, ());
    }
    queue
        .roundtrip(&mut state)
        .map_err(|e| format!("roundtrip: {e}"))?;

    let target = state
        .output_names
        .iter()
        .find(|(_, name)| name.to_uppercase().contains(OUTPUT_HINT))
        .map(|(proxy, _)| proxy.clone());
    let Some(output) = target else {
        return Err(format!("no virtual output matching '{OUTPUT_HINT}' found"));
    };

    let surface = compositor.create_surface(&qh, ());
    let input_region: wl_region::WlRegion = compositor.create_region(&qh, ());
    input_region.add(0, 0, 0, 0);
    surface.set_input_region(Some(&input_region));

    let layer_surface = layer_shell.get_layer_surface(
        &surface,
        Some(&output),
        zwlr_layer_shell_v1::Layer::Overlay,
        "orbiscreen-damage".to_owned(),
        &qh,
        (),
    );
    layer_surface.set_anchor(Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right);
    layer_surface.set_exclusive_zone(-1);
    surface.commit();

    queue
        .roundtrip(&mut state)
        .map_err(|e| format!("configure roundtrip: {e}"))?;
    let Some((serial, width, height)) = state.configured else {
        return Err("layer surface never configured".into());
    };
    if width == 0 || height == 0 {
        return Err("layer surface configured with empty size".into());
    }
    layer_surface.ack_configure(serial);

    let stride = width * 4;
    let pool_size = stride * height * 2;
    let mut file = tempfile_in_mem()?;
    file.write_all(&vec![0u8; pool_size as usize])
        .map_err(|e| format!("shm write: {e}"))?;
    let pool = shm.create_pool(file.as_fd(), pool_size, &qh, ());
    let buffer_a = pool.create_buffer(0, width, height, stride, wl_shm::Format::Argb8888, &qh, ());
    let buffer_b = pool.create_buffer(
        pool_size / 2,
        width,
        height,
        stride,
        wl_shm::Format::Argb8888,
        &qh,
        (),
    );

    tracing::info!(
        "damage pump active on virtual display ({width}x{height}, every {}ms)",
        period.as_millis()
    );

    surface.attach(Some(&buffer_a), 0, 0);
    surface.damage_buffer(0, 0, width, height);
    surface.commit();

    let mut flip = false;
    let mut tick = 0u64;
    loop {
        std::thread::sleep(period);
        let next = if flip { &buffer_a } else { &buffer_b };
        surface.attach(Some(next), 0, 0);
        surface.damage_buffer(0, 0, width, height);
        surface.commit();
        flip = !flip;
        tick += 1;
        if tick % 8 == 0 {
            let _ = queue.roundtrip(&mut state);
        }
    }
}

fn tempfile_in_mem() -> Result<std::fs::File, String> {
    let path = "/tmp/.orbiscreen-damage-shm";
    let _ = std::fs::remove_file(path);
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("shm temp file: {e}"))
}

#[derive(Default)]
struct PumpState {
    configured: Option<(u32, i32, i32)>,
    output_names: Vec<(wl_output::WlOutput, String)>,
}

impl Dispatch<wl_registry::WlRegistry, wayland_client::globals::GlobalListContents> for PumpState {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &wayland_client::globals::GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for PumpState {
    fn event(
        _: &mut Self,
        _: &wl_compositor::WlCompositor,
        _: wl_compositor::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}
impl Dispatch<wl_shm::WlShm, ()> for PumpState {
    fn event(
        _: &mut Self,
        _: &wl_shm::WlShm,
        _: wl_shm::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}
impl Dispatch<wl_shm_pool::WlShmPool, ()> for PumpState {
    fn event(
        _: &mut Self,
        _: &wl_shm_pool::WlShmPool,
        _: wl_shm_pool::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}
impl Dispatch<wl_buffer::WlBuffer, ()> for PumpState {
    fn event(
        _: &mut Self,
        _: &wl_buffer::WlBuffer,
        _: wl_buffer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}
impl Dispatch<wl_surface::WlSurface, ()> for PumpState {
    fn event(
        _: &mut Self,
        _: &wl_surface::WlSurface,
        _: wl_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}
impl Dispatch<wl_region::WlRegion, ()> for PumpState {
    fn event(
        _: &mut Self,
        _: &wl_region::WlRegion,
        _: wl_region::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}
impl Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1, ()> for PumpState {
    fn event(
        _: &mut Self,
        _: &zwlr_layer_shell_v1::ZwlrLayerShellV1,
        _: zwlr_layer_shell_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}
impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for PumpState {
    fn event(
        state: &mut Self,
        layer: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_layer_surface_v1::Event::Configure {
                serial,
                width,
                height,
            } => {
                state.configured = Some((serial, width as i32, height as i32));
                let _ = layer;
            }
            zwlr_layer_surface_v1::Event::Closed => {
                tracing::warn!("damage pump surface closed by compositor");
            }
            _ => {}
        }
    }
}
impl Dispatch<wl_output::WlOutput, ()> for PumpState {
    fn event(
        state: &mut Self,
        proxy: &wl_output::WlOutput,
        event: wl_output::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_output::Event::Name { name } = event {
            state.output_names.push((proxy.clone(), name));
        }
    }
}
