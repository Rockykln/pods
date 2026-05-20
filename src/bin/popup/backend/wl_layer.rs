use std::os::fd::{AsFd, AsRawFd, FromRawFd, OwnedFd};
use std::ptr;

use anyhow::{Context, Result, anyhow, bail};
use wayland_client::globals::{GlobalListContents, registry_queue_init};
use wayland_client::protocol::{
    wl_buffer::WlBuffer,
    wl_compositor::WlCompositor,
    wl_registry::WlRegistry,
    wl_shm::{self, WlShm},
    wl_shm_pool::WlShmPool,
    wl_surface::WlSurface,
};
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle, delegate_noop};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{self, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::{self, Anchor, ZwlrLayerSurfaceV1},
};

use super::{Backend, Frame};

pub struct WlLayer {
    win: Option<Win>,
}

struct Win {
    conn: Connection,
    queue: EventQueue<State>,
    state: State,
    surface: WlSurface,
    layer: ZwlrLayerSurfaceV1,
    bufs: [Buf; 2],
    cur: usize,
    map: *mut u8,
    map_len: usize,
    w: u32,
    h: u32,
}

struct Buf {
    wl: WlBuffer,
    offset: usize,
    busy: bool,
}

#[derive(Default)]
struct State {
    configured: bool,
    closed: bool,
    released: Vec<u32>,
}

impl WlLayer {
    pub fn new() -> Self {
        WlLayer { win: None }
    }
}

impl Backend for WlLayer {
    fn kind(&self) -> &'static str {
        "wl_layer"
    }

    fn open(&mut self, w: u32, h: u32) -> Result<()> {
        let conn = Connection::connect_to_env().context("connect to Wayland display")?;
        let (globals, mut queue) = registry_queue_init::<State>(&conn).context("init registry")?;
        let qh = queue.handle();

        let compositor: WlCompositor = globals
            .bind(&qh, 4..=6, ())
            .map_err(|e| anyhow!("wl_compositor: {e}"))?;
        let shm: WlShm = globals
            .bind(&qh, 1..=2, ())
            .map_err(|e| anyhow!("wl_shm: {e}"))?;
        let layer_shell: ZwlrLayerShellV1 = globals
            .bind(&qh, 1..=4, ())
            .map_err(|_| anyhow!("compositor has no wlr-layer-shell"))?;

        let surface = compositor.create_surface(&qh, ());
        let layer = layer_shell.get_layer_surface(
            &surface,
            None,
            zwlr_layer_shell_v1::Layer::Overlay,
            "podctl-popup".into(),
            &qh,
            (),
        );
        layer.set_size(w, h);
        layer.set_anchor(Anchor::Bottom);
        layer.set_margin(0, 0, 24, 0);
        layer.set_keyboard_interactivity(zwlr_layer_surface_v1::KeyboardInteractivity::None);
        surface.commit();

        let mut state = State::default();
        queue
            .blocking_dispatch(&mut state)
            .context("await layer configure")?;
        if state.closed {
            bail!("compositor closed the layer surface");
        }
        if !state.configured {
            bail!("no layer configure received");
        }

        let stride = w as usize * 4;
        let one = stride * h as usize;
        let map_len = one * 2;
        let (fd, map) = shm_alloc(map_len)?;
        let pool = shm.create_pool(fd.as_fd(), map_len as i32, &qh, ());
        let mk = |off: usize| {
            pool.create_buffer(
                off as i32,
                w as i32,
                h as i32,
                stride as i32,
                wl_shm::Format::Argb8888,
                &qh,
                off as u32,
            )
        };
        let bufs = [
            Buf {
                wl: mk(0),
                offset: 0,
                busy: false,
            },
            Buf {
                wl: mk(one),
                offset: one,
                busy: false,
            },
        ];

        self.win = Some(Win {
            conn,
            queue,
            state,
            surface,
            layer,
            bufs,
            cur: 0,
            map,
            map_len,
            w,
            h,
        });
        Ok(())
    }

    fn push_frame(&mut self, f: &Frame) -> Result<()> {
        let win = self.win.as_mut().context("push_frame before open")?;
        if win.state.closed {
            bail!("layer surface was closed by the compositor");
        }

        for released in win.state.released.drain(..) {
            for b in &mut win.bufs {
                if b.offset as u32 == released {
                    b.busy = false;
                }
            }
        }

        let slot = if !win.bufs[win.cur].busy {
            win.cur
        } else {
            1 - win.cur
        };
        if win.bufs[slot].busy {
            // Both in flight; let the compositor catch up.
            win.queue
                .blocking_dispatch(&mut win.state)
                .context("dispatch waiting for buffer")?;
            for released in win.state.released.drain(..) {
                for b in &mut win.bufs {
                    if b.offset as u32 == released {
                        b.busy = false;
                    }
                }
            }
        }

        let off = win.bufs[slot].offset;
        let len = win.w as usize * 4 * win.h as usize;
        debug_assert_eq!(f.bgra.len(), len);
        unsafe {
            ptr::copy_nonoverlapping(f.bgra.as_ptr(), win.map.add(off), len.min(f.bgra.len()));
        }

        win.layer.set_margin(0, 0, f.margin_bottom, 0);
        win.surface.attach(Some(&win.bufs[slot].wl), 0, 0);
        win.surface.damage_buffer(0, 0, win.w as i32, win.h as i32);
        win.surface.commit();
        win.bufs[slot].busy = true;
        win.cur = 1 - slot;

        win.conn.flush().context("flush wayland")?;
        win.queue
            .dispatch_pending(&mut win.state)
            .context("dispatch pending")?;
        Ok(())
    }

    fn close(&mut self) -> Result<()> {
        if let Some(win) = self.win.take() {
            win.layer.destroy();
            win.surface.destroy();
            for b in &win.bufs {
                b.wl.destroy();
            }
            let _ = win.conn.flush();
            unsafe {
                munmap(win.map, win.map_len);
            }
        }
        Ok(())
    }
}

impl Drop for WlLayer {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

const MFD_CLOEXEC: u32 = 0x0001;
const PROT_READ: i32 = 0x1;
const PROT_WRITE: i32 = 0x2;
const MAP_SHARED: i32 = 0x01;

unsafe extern "C" {
    fn memfd_create(name: *const i8, flags: u32) -> i32;
    fn ftruncate(fd: i32, length: i64) -> i32;
    fn mmap(addr: *mut u8, len: usize, prot: i32, flags: i32, fd: i32, off: i64) -> *mut u8;
    fn munmap(addr: *mut u8, len: usize) -> i32;
    fn __errno_location() -> *mut i32;
}

fn last_err() -> std::io::Error {
    std::io::Error::from_raw_os_error(unsafe { *__errno_location() })
}

fn shm_alloc(len: usize) -> Result<(OwnedFd, *mut u8)> {
    let name = c"podctl-popup";
    let raw = unsafe { memfd_create(name.as_ptr(), MFD_CLOEXEC) };
    if raw < 0 {
        return Err(last_err()).context("memfd_create");
    }
    let fd = unsafe { OwnedFd::from_raw_fd(raw) };
    if unsafe { ftruncate(fd.as_raw_fd(), len as i64) } != 0 {
        return Err(last_err()).context("ftruncate");
    }
    let map = unsafe {
        mmap(
            ptr::null_mut(),
            len,
            PROT_READ | PROT_WRITE,
            MAP_SHARED,
            fd.as_raw_fd(),
            0,
        )
    };
    if map as isize == -1 {
        return Err(last_err()).context("mmap");
    }
    Ok((fd, map))
}

impl Dispatch<ZwlrLayerSurfaceV1, ()> for State {
    fn event(
        st: &mut Self,
        layer: &ZwlrLayerSurfaceV1,
        ev: zwlr_layer_surface_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match ev {
            zwlr_layer_surface_v1::Event::Configure { serial, .. } => {
                layer.ack_configure(serial);
                st.configured = true;
            }
            zwlr_layer_surface_v1::Event::Closed => {
                st.closed = true;
            }
            _ => {}
        }
    }
}

impl Dispatch<WlBuffer, u32> for State {
    fn event(
        st: &mut Self,
        _: &WlBuffer,
        ev: wayland_client::protocol::wl_buffer::Event,
        offset: &u32,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if matches!(ev, wayland_client::protocol::wl_buffer::Event::Release) {
            st.released.push(*offset);
        }
    }
}

impl Dispatch<WlRegistry, GlobalListContents> for State {
    fn event(
        _: &mut Self,
        _: &WlRegistry,
        _: wayland_client::protocol::wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

delegate_noop!(State: ignore WlCompositor);
delegate_noop!(State: ignore WlShm);
delegate_noop!(State: ignore WlShmPool);
delegate_noop!(State: ignore WlSurface);
delegate_noop!(State: ignore ZwlrLayerShellV1);
