//! What the client draws, in world space: the free runner's game
//! (`rondata::free`), the heightfield it reports, and each drawn figure's
//! decoded mesh at its draw-list anchor and facing (`docs/RENDERER.md`
//! §3.6). Nothing here touches the GPU; `gpu` takes the triangles.

use render::bh3;
use render::draw::{Drawn, Placed, PlacedBuilding, Pose};
use render::figure::{apply, placement, rest_pose};
use render::terrain::{Heightfield, TerrainMesh};
use rondata::artdata::{BuildLook, build_art, resolve_art, unit_art};
use rondata::free::{FreeRunner, placed, placed_buildings};
use rondata::{Install, TribeDef};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The pair the client opens by default: the Aztecs at a held Modern age,
/// Great Lakes (`rondata::diff::aztec_modern_pair`; RENDERER §5.4's M is
/// counted on it).
pub const START_DUMP: &str = "gamelog-run1428-greatlakes-aztec-modern-start.txt";
pub const LONG_DUMP: &str = "gamelog-run1429-greatlakes-aztec-modern-trace.txt";
pub const LONG_TRACE: &str = "rontrace-run1429.log";

/// Where the captures sit: `$RON_GAMELOG_DIR`, else the bottle's `Logs\`,
/// as the test environment finds them.
pub fn gamelog_dir() -> PathBuf {
    std::env::var_os("RON_GAMELOG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let home = std::env::var_os("HOME").unwrap_or_default();
            Path::new(&home).join("ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/Logs")
        })
}

/// **`guy_scale`** (`?guy_scale@@3MA`, `0x00c06244` by the linker map),
/// read off run1441's packet at logger frame 3294: 4.8. `GuyOut::draw`
/// multiplies a figure's piece scale (`RData +0x88`, the `<UNIT>`'s
/// `scale=`) by it unless `RData +0x92` has bit 2, which only a building's
/// piece carries; run1559b's packet reads both for the drawn pieces
/// (`docs/RENDERER.md` §3.6, item 4065).
pub const GUY_SCALE: f32 = 4.8;

/// A posed model at rest: its vertices in model space and its triangles.
pub struct Mesh {
    pub rest: Vec<[f32; 3]>,
    pub faces: Vec<[u16; 3]>,
    /// The `<UNIT>`'s `scale=` times [`GUY_SCALE`]; 1 for a building.
    pub scale: f32,
}

/// One coloured triangle soup, world space.
#[derive(Default)]
pub struct Soup {
    pub positions: Vec<[f32; 3]>,
    pub colours: Vec<[f32; 3]>,
}

/// The game and the art, stepped and drawn from.
pub struct World {
    pub runner: FreeRunner,
    look: BuildLook,
    tribes: Vec<TribeDef>,
    /// Each piece's mesh, `None` where it resolves to nothing that decodes.
    meshes: BTreeMap<i32, Option<std::sync::Arc<Mesh>>>,
    unit_models: BTreeMap<i32, (String, String)>,
    root: PathBuf,
    install: Install,
    terrain: (u32, TerrainMesh),
}

impl World {
    /// Open the install and the pair's captures, and stand the game up at
    /// frame zero.
    pub fn open(install_root: &Path, logs: &Path) -> Result<World, String> {
        let install = Install::new(install_root);
        if !install.looks_valid() {
            return Err(format!("no install at {}", install_root.display()));
        }
        let loaded = rondata::load::load(&install).map_err(|e| e.to_string())?;
        let (long, start) = (logs.join(LONG_DUMP), logs.join(START_DUMP));
        let trace = rondata::trace::Trace::read(&logs.join(LONG_TRACE))
            .map_err(|e| format!("{LONG_TRACE}: {e}"))?;
        let runner = FreeRunner::from_captures(
            &loaded,
            &long,
            &[start.as_path()],
            trace.as_ref(),
            sim::Tuning::RON,
        )
        .map_err(|e| format!("{}: {e}", long.display()))?;
        let rules = install.rules().map_err(|e| e.to_string())?;
        let tribes = install.tribe_defs(&rules).map_err(|e| e.to_string())?;
        let graphs: Vec<String> = install
            .units()
            .map_err(|e| e.to_string())?
            .records
            .iter()
            .map(|r| r.text("GRAPH").unwrap_or_default().trim().to_string())
            .collect();
        let mut unit_models = BTreeMap::new();
        for e in unit_art(&install, &graphs) {
            for p in &e.pieces {
                unit_models
                    .entry(*p)
                    .or_insert_with(|| (e.model.clone(), e.scale.clone()));
            }
        }
        Ok(World {
            look: BuildLook::new(&install).with_types(&loaded.build_types),
            tribes,
            meshes: BTreeMap::new(),
            unit_models,
            root: install.root().to_path_buf(),
            install,
            runner,
            terrain: (u32::MAX, TerrainMesh::default()),
        })
    }

    /// Step the game to `frame` (the dump's `FRAME frame`).
    pub fn run_to(&mut self, frame: i64) {
        while self.runner.frame() < frame {
            self.runner.step();
        }
    }

    /// The heightfield's triangles, rebuilt when the door's terrain version
    /// moves.
    pub fn terrain(&mut self) -> &TerrainMesh {
        let t = self.runner.terrain();
        if t.version != self.terrain.0 || self.terrain.1.positions.is_empty() {
            let field = Heightfield {
                width: t.width,
                height: t.height,
                corners: &t.corners,
            };
            let mountains: Vec<(i32, i32)> =
                t.mountains.iter().flatten().map(|p| (p.x, p.y)).collect();
            self.terrain = (t.version, TerrainMesh::build(&field, &t.land, &mountains));
        }
        &self.terrain.1
    }

    /// This frame's figures and buildings as the observer sees them: the
    /// placed input and the draw list.
    pub fn draw_list(
        &self,
        camera: &render::Camera,
        screen: &render::Screen,
        size: (i32, i32),
    ) -> (Vec<Placed>, Vec<PlacedBuilding>, Vec<Drawn>) {
        let scene = self.runner.scene(None);
        let figures = placed(&scene);
        let buildings = placed_buildings(&scene, &self.look, &self.tribes);
        let list = render::draw::draw_list(
            &figures,
            &buildings,
            self.look.scaffold_timing(),
            camera,
            screen,
            Some(size),
            None,
        );
        (figures, buildings, list)
    }

    fn mesh(&mut self, piece: i32, building: bool) -> Option<std::sync::Arc<Mesh>> {
        if let Some(m) = self.meshes.get(&piece) {
            return m.clone();
        }
        let (model, scale) = if building {
            let set = std::iter::once(piece).collect();
            let art = build_art(&self.install, &set);
            let z = art.get(&piece)?.as_ref().ok()?.zbuffer.clone();
            (z.unwrap_or_default(), 1.0)
        } else {
            let (m, s) = self.unit_models.get(&piece)?.clone();
            (m, s.parse::<f32>().unwrap_or(1.0) * GUY_SCALE)
        };
        let decoded = resolve_art(&self.root, &model)
            .and_then(|p| std::fs::read(p).ok())
            .and_then(|b| bh3::decode(&b).ok())
            .map(|m| {
                std::sync::Arc::new(Mesh {
                    rest: rest_pose(&m),
                    faces: m.faces,
                    scale,
                })
            });
        self.meshes.insert(piece, decoded.clone());
        decoded
    }

    /// Every drawn figure's and building's mesh, placed in world space: one
    /// entry a draw-list entry the client draws, its owner, whether it is a
    /// building, and its triangles.
    pub fn objects(
        &mut self,
        figures: &[Placed],
        buildings: &[PlacedBuilding],
        list: &[Drawn],
    ) -> Vec<Drawing> {
        let at_figure: BTreeMap<(u8, i16, u16), &Placed> = figures
            .iter()
            .map(|p| ((p.who, p.o, p.figure), p))
            .collect();
        let at_building: BTreeMap<(u8, i16), &PlacedBuilding> =
            buildings.iter().map(|b| ((b.who, b.o), b)).collect();
        let mut out = Vec::new();
        for d in list {
            let (point, facing, building) = match d.pose {
                Pose::Figure { figure, facing, .. } => {
                    let Some(p) = at_figure.get(&(d.who, d.o, figure)) else {
                        continue;
                    };
                    ([p.x, p.y, p.z], facing, false)
                }
                Pose::Building { .. } => {
                    let Some(b) = at_building.get(&(d.who, d.o)) else {
                        continue;
                    };
                    let (x, y, z) = b.anchor_point();
                    ([x, y, z], 0, true)
                }
                // The wheat field is a mesh `FarmsOut::draw` builds, not a
                // piece; the client does not build it yet (item 4048).
                Pose::Field { .. } => continue,
                // The foundation is a `<CONSTRUCTION>` sprite and the
                // scaffold a `<SCAFFOLD>` mesh by element, neither a
                // building piece; the client draws neither yet (item 4061).
                Pose::Foundation { .. } | Pose::Scaffold { .. } => continue,
            };
            let Some(mesh) = self.mesh(d.piece, building) else {
                continue;
            };
            let m = placement(point, facing, mesh.scale);
            let tris = mesh
                .faces
                .iter()
                .map(|f| f.map(|i| apply(&m, mesh.rest[i as usize])))
                .collect();
            out.push(Drawing {
                who: d.who,
                o: d.o,
                piece: d.piece,
                building,
                tris,
            });
        }
        out
    }

    /// Every drawn figure's and building's mesh, placed, as a triangle soup
    /// shaded flat by an owner's debug tint.
    pub fn figures(
        &mut self,
        figures: &[Placed],
        buildings: &[PlacedBuilding],
        list: &[Drawn],
    ) -> Soup {
        let mut soup = Soup::default();
        for d in self.objects(figures, buildings, list) {
            let tint = tint(d.who, d.building);
            for tri in d.tris {
                let lit = shade(tri);
                for p in tri {
                    soup.positions.push(p);
                    soup.colours.push(tint.map(|v| v * lit));
                }
            }
        }
        soup
    }
}

/// One drawn object, placed: a draw-list entry's owner, object and piece,
/// and its mesh's triangles in world space.
pub struct Drawing {
    pub who: u8,
    pub o: i16,
    pub piece: i32,
    pub building: bool,
    pub tris: Vec<[[f32; 3]; 3]>,
}

/// The debug tint an owner's meshes take: untextured, not a team colour
/// (item 4038's).
pub fn tint(who: u8, building: bool) -> [f32; 3] {
    let base = match who {
        0 => [0.62, 0.66, 0.85],
        1 => [0.85, 0.58, 0.52],
        _ => [0.80, 0.78, 0.66],
    };
    if building {
        base.map(|v| v * 0.9)
    } else {
        base
    }
}

/// A flat shade from the triangle's normal against a light from above and
/// the north-west, either face lit.
fn shade(t: [[f32; 3]; 3]) -> f32 {
    let u = [t[1][0] - t[0][0], t[1][1] - t[0][1], t[1][2] - t[0][2]];
    let v = [t[2][0] - t[0][0], t[2][1] - t[0][1], t[2][2] - t[0][2]];
    let n = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if l == 0.0 {
        return 0.6;
    }
    let light = [-0.4f32, -0.4, 0.82];
    let ll = (light[0] * light[0] + light[1] * light[1] + light[2] * light[2]).sqrt();
    0.45 + 0.55 * ((n[0] * light[0] + n[1] * light[1] + n[2] * light[2]) / (l * ll)).abs()
}
