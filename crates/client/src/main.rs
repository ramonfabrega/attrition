//! **The client: first pixels** (`docs/RENDERER.md` §3.6, §6 item 5; item
//! 4036). A window over the free runner's game on the Aztec Modern pair:
//! the terrain heightfield in flat land-type colours, each figure's decoded
//! mesh at its draw-list anchor and facing, untextured, at the camera the
//! original held on run1559b's frame 3294 — the observer's view with a debug
//! overlay, and nothing else (§7.2 items 7 and 8).
//!
//! ```text
//! client [--install DIR] [--logs DIR] [--frame N] [--size WxH]
//!        [--still OUT.bmp] [--frames N] [--play] [--trace OUT.json]
//!        [--look X,Y] [--zoom F] [--silhouette OUT.bmp] [--rest]
//! client --read-trace SESSION.json
//! ```
//!
//! `--still` draws one frame off-screen and writes it; otherwise a window
//! opens (arrows pan, `+`/`-` zoom, space plays, `Q` or Escape quits), and
//! `--frames N` quits after N presented frames. `--silhouette` scores the
//! frame's silhouettes against run1559b's photograph (`silhouette`) and
//! writes the picture of the scoring. Each figure is drawn posed at its
//! animation slot and clock (item 4067, the default since item 4069);
//! `--rest` draws it in its model's rest pose. Every session writes its
//! trace (`trace`), and `--read-trace` prints the clock it holds.
//!
//! The simulation stays headless: this crate depends on `sim`, `rondata`
//! and `render`, and none of them on it (`tests::the_sim_takes_no_graphics`).

mod gpu;
mod overlay;
mod silhouette;
mod trace;
mod world;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use trace::{Clock, Session};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};
use world::World;

/// **The pinned camera**: the game's own on run1559b's packet at logger
/// frame 3294 on this lobby (`tools/render/packet_points.py`, the
/// `state 0` line, bits as it prints them) — the default zoom, tilt 60,
/// dir 45, parallel, looking at (2104.28, 30816), beside the human's
/// capital — at the packet's 1920×1080: the camera of the photograph the
/// same take made of the same frame (`docs/RUNS.md`, run1559b). run1441's
/// packet of the same frame of the same game held (3168, 29744.68): the
/// camera is not a function of the frame (`docs/RENDERER.md` §3.6).
pub fn pinned_camera() -> render::Camera {
    let f = f32::from_bits;
    render::Camera {
        zoom_distance: f(0x46a2_f59c),
        camera_scale: f(0x3f80_0000),
        tilt_angle: f(0x4270_0000),
        dir_angle: f(0x4234_0000),
        loc_x: f(0x4503_847c),
        loc_y: f(0x46f0_c000),
        loc_x_adjust: f(0x8000_0000),
        loc_y_adjust: f(0x8000_0000),
        loc_z_adjust: 0.0,
        angle: f(0x4234_0000),
        near_plane: f(0x45cd_eb38),
        far_plane: f(0x470d_a61b),
        view_ratio: f(0x3f10_2222),
        parallel: true,
        parallel_distance: f(0x4622_f59c),
    }
}

/// The screen the camera's matrix is read at: `graphsys`'s on the packet.
pub fn pinned_screen() -> render::Screen {
    render::Screen {
        scaled_xs: 1920,
        scaled_ys: 1080,
        natural_ratio: f32::from_bits(0x3f80_1e57),
        console_y: 0,
    }
}

struct Args {
    install: PathBuf,
    logs: PathBuf,
    frame: i64,
    size: (u32, u32),
    still: Option<PathBuf>,
    frames: Option<u64>,
    play: bool,
    trace: PathBuf,
    /// Look at another point, or closer: the camera's `loc` and its
    /// `parallel_distance` divided by the zoom.
    look: Option<(f32, f32)>,
    zoom: f32,
    /// Score the frame's silhouettes against the photograph and write the
    /// picture of the scoring (`silhouette`, item 4038).
    silhouette: Option<PathBuf>,
    /// Pose the figures (`world::World::pose`, item 4067).
    pose: bool,
}

fn parse() -> Result<Args, String> {
    let mut a = Args {
        install: std::env::var_os("RON_INSTALL")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("game")),
        logs: world::gamelog_dir(),
        frame: 3294,
        size: (1920, 1080),
        still: None,
        frames: None,
        play: false,
        look: None,
        zoom: 1.0,
        silhouette: None,
        pose: true,
        trace: std::env::temp_dir().join(format!("attrition-client-{}.json", std::process::id())),
    };
    let mut it = std::env::args().skip(1);
    while let Some(k) = it.next() {
        let mut v = || it.next().ok_or(format!("{k} wants a value"));
        match k.as_str() {
            "--install" => a.install = v()?.into(),
            "--logs" => a.logs = v()?.into(),
            "--frame" => a.frame = v()?.parse().map_err(|e| format!("--frame: {e}"))?,
            "--size" => {
                let s = v()?;
                let (w, h) = s.split_once('x').ok_or("--size WxH")?;
                a.size = (
                    w.parse().map_err(|e| format!("--size: {e}"))?,
                    h.parse().map_err(|e| format!("--size: {e}"))?,
                );
            }
            "--still" => a.still = Some(v()?.into()),
            "--frames" => a.frames = Some(v()?.parse().map_err(|e| format!("--frames: {e}"))?),
            "--play" => a.play = true,
            "--rest" => a.pose = false,
            "--silhouette" => a.silhouette = Some(v()?.into()),
            "--zoom" => a.zoom = v()?.parse().map_err(|e| format!("--zoom: {e}"))?,
            "--look" => {
                let s = v()?;
                let (x, y) = s.split_once(',').ok_or("--look X,Y")?;
                a.look = Some((
                    x.parse().map_err(|e| format!("--look: {e}"))?,
                    y.parse().map_err(|e| format!("--look: {e}"))?,
                ));
            }
            "--trace" => a.trace = v()?.into(),
            "--read-trace" => {
                let text = std::fs::read_to_string(v()?).map_err(|e| e.to_string())?;
                let c = Clock::read(&text).ok_or("no `frame zero` mark in the trace")?;
                println!(
                    "CLOCK client: {:.0} ms to frame zero, {:.0} ms to first pixels; {} frames, \
                     p99 {:.2} ms, slowest {:.2} ms; peak RSS {} MiB (sampled)",
                    c.to_frame_zero_ms,
                    c.to_first_pixels_ms,
                    c.frames,
                    c.p99_frame_ms,
                    c.max_frame_ms,
                    c.peak_rss_kib / 1024
                );
                std::process::exit(0);
            }
            _ => return Err(format!("unknown argument {k}")),
        }
    }
    Ok(a)
}

/// The scene's triangles at the client's state, the overlay with them.
struct Built {
    figures: world::Soup,
    overlay: (Vec<[f32; 3]>, Vec<[f32; 3]>),
}

fn build(world: &mut World, camera: &render::Camera, size: (u32, u32), frame_ms: f64) -> Built {
    let (figures, buildings, list) =
        world.draw_list(camera, &pinned_screen(), (size.0 as i32, size.1 as i32));
    let figures = world.figures(&figures, &buildings, &list);
    let lines = overlay::Lines {
        frame: world.runner.frame(),
        frame_ms,
        camera: *camera,
    }
    .text();
    let overlay = overlay::triangles(&lines, (size.0 as i32, size.1 as i32), 2);
    Built { figures, overlay }
}

fn draw(
    gpu: &gpu::Gpu,
    world: &mut World,
    target: Option<&wgpu::TextureView>,
    camera: &render::Camera,
    size: (u32, u32),
    frame_ms: f64,
) -> Option<Vec<u8>> {
    let b = build(world, camera, size, frame_ms);
    let version = world.runner.terrain().version;
    let t = world.terrain();
    let frame = gpu::Frame {
        terrain_version: version,
        terrain: (&t.positions, &t.colours),
        figures: (&b.figures.positions, &b.figures.colours),
        overlay: (&b.overlay.0, &b.overlay.1),
    };
    match target {
        Some(view) => {
            gpu.draw(view, size, camera, &frame);
            None
        }
        None => Some(gpu.still(size, camera, &frame)),
    }
}

struct App {
    session: Session,
    world: World,
    args: Args,
    camera: render::Camera,
    instance: wgpu::Instance,
    window: Option<(Arc<Window>, wgpu::Surface<'static>, gpu::Gpu)>,
    presented: u64,
    last_ms: f64,
    playing: bool,
}

impl App {
    fn configure(&self, surface: &wgpu::Surface<'_>, gpu: &gpu::Gpu, size: (u32, u32)) {
        surface.configure(
            &gpu.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: gpu.format(),
                width: size.0.max(1),
                height: size.1.max(1),
                present_mode: wgpu::PresentMode::AutoVsync,
                desired_maximum_frame_latency: 2,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
                color_space: wgpu::SurfaceColorSpace::Auto,
            },
        );
    }

    fn key(&mut self, el: &ActiveEventLoop, key: &Key) {
        let step = 256.0 * self.camera.parallel_distance / 10429.4;
        match key {
            Key::Named(NamedKey::Escape) => el.exit(),
            Key::Character(c) if c.as_str() == "q" => el.exit(),
            Key::Named(NamedKey::Space) => self.playing = !self.playing,
            // The view's own right and up, with `dir` 45: a screen axis is
            // a world diagonal.
            Key::Named(NamedKey::ArrowLeft) => self.pan(-step, 0.0),
            Key::Named(NamedKey::ArrowRight) => self.pan(step, 0.0),
            Key::Named(NamedKey::ArrowUp) => self.pan(0.0, -step),
            Key::Named(NamedKey::ArrowDown) => self.pan(0.0, step),
            Key::Character(c) if c.as_str() == "+" || c.as_str() == "=" => {
                self.camera.parallel_distance /= 1.1
            }
            Key::Character(c) if c.as_str() == "-" => self.camera.parallel_distance *= 1.1,
            _ => {}
        }
    }

    fn pan(&mut self, right: f32, down: f32) {
        let d = self.camera.dir_angle.to_radians();
        let (s, c) = d.sin_cos();
        self.camera.loc_x += right * c + down * s;
        self.camera.loc_y += -right * s + down * c;
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("attrition — first pixels")
            .with_inner_size(winit::dpi::PhysicalSize::new(
                self.args.size.0,
                self.args.size.1,
            ));
        let window = match el.create_window(attrs) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                eprintln!("client: no window: {e}");
                el.exit();
                return;
            }
        };
        let made = self.session.phase("surface and device", || {
            let surface = self
                .instance
                .create_surface(window.clone())
                .map_err(|e| e.to_string())?;
            let gpu = gpu::Gpu::new(
                &self.instance,
                Some(&surface),
                wgpu::TextureFormat::Bgra8UnormSrgb,
            )?;
            Ok::<_, String>((surface, gpu))
        });
        match made {
            Ok((surface, gpu)) => {
                let s = window.inner_size();
                self.configure(&surface, &gpu, (s.width, s.height));
                window.request_redraw();
                self.window = Some((window, surface, gpu));
            }
            Err(e) => {
                eprintln!("client: {e}");
                el.exit();
            }
        }
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::Resized(s) => {
                if let Some((_, surface, gpu)) = &self.window {
                    self.configure(surface, gpu, (s.width, s.height));
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key,
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => self.key(el, &logical_key),
            WindowEvent::RedrawRequested => {
                let t = Instant::now();
                if self.playing {
                    self.world.runner.step();
                }
                let Some((window, surface, gpu)) = self.window.take() else {
                    return;
                };
                let s = window.inner_size();
                let texture = match surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(t)
                    | wgpu::CurrentSurfaceTexture::Suboptimal(t) => Some(t),
                    _ => None,
                };
                if let Some(texture) = texture {
                    let view = texture.texture.create_view(&Default::default());
                    draw(
                        &gpu,
                        &mut self.world,
                        Some(&view),
                        &self.camera,
                        (s.width, s.height),
                        self.last_ms,
                    );
                    gpu.queue.present(texture);
                    self.presented += 1;
                    if self.presented == 1 {
                        self.session.mark("first pixels");
                    }
                }
                self.session.span("frame", t);
                self.last_ms = t.elapsed().as_secs_f64() * 1000.0;
                if self.args.frames.is_some_and(|n| self.presented >= n) {
                    el.exit();
                }
                window.request_redraw();
                self.window = Some((window, surface, gpu));
            }
            _ => {}
        }
    }
}

fn main() {
    let session = Session::start();
    let args = match parse() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("client: {e}");
            std::process::exit(64);
        }
    };
    let opened = session.phase("open the install and the captures", || {
        World::open(&args.install, &args.logs)
    });
    let mut world = match opened {
        Ok(w) => w,
        Err(e) => {
            eprintln!("client: {e}");
            std::process::exit(1);
        }
    };
    world.pose = args.pose;
    session.mark("frame zero");
    session.phase("run to the frame", || world.run_to(args.frame));
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let mut camera = pinned_camera();
    if let Some((x, y)) = args.look {
        (camera.loc_x, camera.loc_y) = (x, y);
    }
    camera.parallel_distance /= args.zoom;
    if let Some(out) = &args.silhouette {
        let path = silhouette::photo_path();
        let photo = std::fs::read(&path)
            .map_err(|e| e.to_string())
            .and_then(|b| render::tga::decode(&b).map_err(|e| e.to_string()))
            .unwrap_or_else(|e| {
                eprintln!("client: {}: {e}", path.display());
                std::process::exit(1);
            });
        let rows = silhouette::score_frame(
            &mut world,
            &camera,
            &photo,
            render::silhouette::Reader::Colour,
        );
        for (f, (who, o, piece, building)) in &rows {
            println!(
                "{who}/{o} piece {piece}{} window ({}, {})..({}, {}) ours {} theirs {} both {} \
                 overlap {:.3}",
                if *building { " building" } else { "" },
                f.window.x0,
                f.window.y0,
                f.window.x1,
                f.window.y1,
                f.ours,
                f.theirs,
                f.both,
                f.overlap()
            );
        }
        let scored: Vec<_> = rows.iter().map(|(f, _)| *f).collect();
        let (k, n) = render::silhouette::score(&scored, &render::silhouette::READING);
        println!("silhouettes {k} of {n} (frame {})", world.runner.frame());
        let rgba = silhouette::explain(&mut world, &camera, &photo, &scored);
        let size = (photo.width as u32, photo.height as u32);
        std::fs::write(out, gpu::bmp(&rgba, size)).unwrap_or_else(|e| {
            eprintln!("client: {}: {e}", out.display());
            std::process::exit(1);
        });
        return;
    }
    if let Some(out) = &args.still {
        let gpu = session
            .phase("device", || {
                gpu::Gpu::new(&instance, None, wgpu::TextureFormat::Rgba8UnormSrgb)
            })
            .unwrap_or_else(|e| {
                eprintln!("client: {e}");
                std::process::exit(1);
            });
        let t = Instant::now();
        let rgba = draw(&gpu, &mut world, None, &camera, args.size, 0.0).unwrap_or_default();
        session.span("frame", t);
        session.mark("first pixels");
        std::fs::write(out, gpu::bmp(&rgba, args.size)).unwrap_or_else(|e| {
            eprintln!("client: {}: {e}", out.display());
            std::process::exit(1);
        });
        println!("still: {} (frame {})", out.display(), world.runner.frame());
    } else {
        let el = EventLoop::new().unwrap_or_else(|e| {
            eprintln!("client: no event loop: {e}");
            std::process::exit(1);
        });
        let mut app = App {
            session: session.clone(),
            world,
            playing: args.play,
            args,
            camera,
            instance,
            window: None,
            presented: 0,
            last_ms: 0.0,
        };
        if let Err(e) = el.run_app(&mut app) {
            eprintln!("client: {e}");
        }
        println!(
            "presented {} frames, at frame {}",
            app.presented,
            app.world.runner.frame()
        );
        let trace = app.args.trace.clone();
        return finish(&session, &trace);
    }
    finish(&session, &args.trace);
}

fn finish(session: &Session, path: &std::path::Path) {
    match session.finish(path) {
        Ok(()) => println!("trace: {}", path.display()),
        Err(e) => eprintln!("client: trace {}: {e}", path.display()),
    }
}

#[cfg(test)]
mod tests {
    /// **The simulation takes no graphics** (`CLAUDE.md`'s hard constraint;
    /// `docs/RENDERER.md` §4.2): `sim` has no dependencies at all, so
    /// neither `wgpu` nor `winit` nor an async runtime can reach it; and
    /// `render`, the headless core, has none either. The client is the only
    /// crate that holds them.
    #[test]
    fn the_sim_takes_no_graphics() {
        let manifest = |c: &str| {
            std::fs::read_to_string(format!("{}/../{c}/Cargo.toml", env!("CARGO_MANIFEST_DIR")))
                .unwrap()
        };
        let deps = |text: &str| -> Vec<String> {
            let mut on = false;
            let mut out = Vec::new();
            for l in text.lines() {
                let l = l.trim();
                if l.starts_with('[') {
                    on = l == "[dependencies]";
                } else if on && !l.is_empty() && !l.starts_with('#') {
                    out.push(l.split(['=', ' ']).next().unwrap().to_string());
                }
            }
            out
        };
        assert_eq!(
            deps(&manifest("sim")),
            Vec::<String>::new(),
            "sim has no dependencies"
        );
        assert_eq!(
            deps(&manifest("render")),
            Vec::<String>::new(),
            "render is headless"
        );
        let rondata = deps(&manifest("rondata"));
        for g in ["wgpu", "winit", "tokio", "client"] {
            assert!(!rondata.contains(&g.to_string()), "rondata depends on {g}");
        }
        let client = deps(&manifest("client"));
        assert!(client.contains(&"wgpu".to_string()) && client.contains(&"winit".to_string()));
    }

    /// The pinned camera is run1559b's packet's, bit for bit, and a world
    /// point goes through the GPU's arithmetic — the matrix, the divide by
    /// `w`, the viewport — to the pixel the draw list's anchor names.
    #[test]
    fn the_gpu_s_projection_lands_on_the_draw_list_s_anchor() {
        let c = super::pinned_camera();
        assert_eq!(c.loc_y.to_bits(), 0x46f0_c000);
        let m = c.to_screen_matrix().0;
        for (x, y, z) in [(5016, 30360, 565), (3168, 29744, 300), (0, 0, 0)] {
            let v = [x as f32, y as f32, z as f32, 1.0];
            let clip: Vec<f32> = (0..4)
                .map(|col| (0..4).map(|r| v[r] * m[r * 4 + col]).sum())
                .collect();
            let (nx, ny) = (clip[0] / clip[3], clip[1] / clip[3]);
            let (px, py) = ((nx + 1.0) * 1920.0 * 0.5, (1.0 - ny) * 1080.0 * 0.5);
            let a = render::camera::world_to_screen(
                &c.to_screen_matrix(),
                &super::pinned_screen(),
                x,
                y,
                z,
                Some((1920, 1080)),
            );
            assert!(
                (px - a.x as f32).abs() <= 1.0 && (py - a.y as f32).abs() <= 1.0,
                "({x}, {y}, {z}): gpu ({px}, {py}), anchor ({}, {})",
                a.x,
                a.y
            );
        }
        // run1559b's own pixel for its first SUBOBJECT (packet_points.py's
        // `0 0` row): the screen size is `graphsys`'s, as the original's
        // call takes it — an explicit 1920×1080 lands a row higher, at 624.
        let a = render::camera::world_to_screen(
            &c.to_screen_matrix(),
            &super::pinned_screen(),
            5016,
            30360,
            565,
            None,
        );
        assert_eq!((a.x, a.y), (1489, 625));
    }
}
