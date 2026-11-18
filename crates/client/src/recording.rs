//! A recording of the orders accepted by the window's turn pump.
use rondata::commands::Command;
use rondata::lobby::Choices;
use rondata::recgame::{self, Order, RecGame};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Resolve the existing parent, including symlinks, before comparing two new files.
fn destination(path: &Path) -> Result<PathBuf, String> {
    let name = path
        .file_name()
        .ok_or_else(|| format!("missing file name: {}", path.display()))?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let dir = parent
        .canonicalize()
        .map_err(|e| format!("{}: {e}", parent.display()))?;
    Ok(dir.join(name))
}

pub fn check_paths(path: &Path, trace: &Path) -> Result<(), String> {
    if path.try_exists().map_err(|e| e.to_string())? || destination(path)? == destination(trace)? {
        return Err(format!(
            "choose an unused recording path distinct from the trace: {}",
            path.display()
        ));
    }
    Ok(())
}

pub struct Recording {
    path: PathBuf,
    header: RecGame,
    leader: Command,
    orders: Vec<Order>,
    paused: bool,
}

impl Recording {
    pub fn open(
        path: &Path,
        choices: &Choices,
        logs: &Path,
        leader: Command,
    ) -> Result<Self, String> {
        if path.try_exists().map_err(|e| e.to_string())? {
            return Err(format!("{} already exists", path.display()));
        }
        if !choices.seats.first().is_some_and(|seat| seat.human) {
            return Err("the window records the human in slot 0".into());
        }
        let dir =
            std::env::var_os("RON_RECGAME_DIR").map_or_else(|| logs.to_owned(), PathBuf::from);
        let mut donors = Vec::new();
        for entry in std::fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))? {
            let donor = entry.map_err(|e| e.to_string())?.path();
            if donor.extension().is_some_and(|ext| ext == "rcx") {
                donors.push(donor);
            }
        }
        donors.sort();
        let lobby = choices.lobby();
        let (info, players) = lobby.borrowed();
        for donor in donors {
            let Ok(donor) = recgame::read(&donor.to_string_lossy()) else {
                continue;
            };
            if donor.truncated.is_empty()
                && let Ok(header) = recgame::from_lobby(&info, &players, &donor, Vec::new())
            {
                return Ok(Self {
                    path: path.to_owned(),
                    header,
                    leader,
                    orders: Vec::new(),
                    paused: false,
                });
            }
        }
        Err(format!(
            "no finished donor recording with this lobby's version and options in {}",
            dir.display()
        ))
    }

    /// Apply an input event transactionally. A refused package must leave
    /// the last-sent group and the pause state as the live pump saw them.
    pub fn event(
        &mut self,
        input: &mut crate::input::Input,
        size: (i32, i32),
        frame: i32,
        ev: crate::input::Event,
        view: &crate::input::View,
    ) -> Result<Option<Order>, String> {
        let before = input.clone();
        let order = crate::hud::event(input, size, frame, ev, view);
        if let Some(order) = &order
            && let Err(e) = self.accept(order, input.paused)
        {
            *input = before;
            return Err(e);
        }
        Ok(order)
    }

    // Events between ticks share the turn's package. While the game is
    // paused, each subsequent turn can carry another package at that frame.
    fn accept(&mut self, order: &Order, paused_after: bool) -> Result<(), String> {
        let bytes = rondata::commands::encode(&order.2).len();
        let previous = self
            .orders
            .last_mut()
            .filter(|p| !self.paused && p.0 == order.0 && p.1 == order.1);
        let prior = previous
            .as_ref()
            .map_or(0, |p| rondata::commands::encode(&p.2).len());
        if prior + bytes > 0x200 {
            return Err(format!(
                "frame {}: the turn exceeds the 512-byte package limit",
                order.0
            ));
        }
        if let Some(p) = previous {
            p.2.extend_from_slice(&order.2);
        } else {
            self.orders.push(order.clone());
        }
        self.paused = paused_after;
        Ok(())
    }

    pub fn pending(&self, frame: i64) -> bool {
        self.orders.last().is_some_and(|o| i64::from(o.0) == frame)
    }

    fn finish(&mut self, last: i64) -> Result<Vec<u8>, String> {
        let last = i32::try_from(last).map_err(|e| e.to_string())?;
        // A close while paused saves the completed walk. Inputs waiting
        // on the unstepped frame have never been applied by FreeRunner.
        self.orders.retain(|order| order.0 < last);
        self.header.packages = recgame::minted_packages(self.leader.clone(), &self.orders, last)?;
        Ok(recgame::write_gz(&self.header))
    }

    pub fn save(mut self, last: i64) -> Result<(), String> {
        let bytes = self.finish(last)?;
        let mut name = self.path.as_os_str().to_owned();
        name.push(format!(".{}.partial", std::process::id()));
        let temp = PathBuf::from(name);
        let mut file =
            std::fs::File::create_new(&temp).map_err(|e| format!("{}: {e}", temp.display()))?;
        file.write_all(&bytes)
            .and_then(|()| file.sync_all())
            .map_err(|e| format!("writing {}: {e}", temp.display()))?;
        drop(file);
        std::fs::hard_link(&temp, &self.path).map_err(|e| {
            format!(
                "publishing {}: {e}. Completed recording kept at {}",
                self.path.display(),
                temp.display()
            )
        })?;
        std::fs::remove_file(&temp).map_err(|e| {
            format!(
                "saved {}, temporary link {}: {e}",
                self.path.display(),
                temp.display()
            )
        })?;
        println!(
            "recorded {} input packages through frame {last}: {}",
            self.orders.len(),
            self.path.display()
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests;
