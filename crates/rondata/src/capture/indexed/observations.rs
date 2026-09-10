//! Bounded reuse of owned replay observations, never capture text or sim state.
use crate::gamelog::{FrameGuys, FrameUnit, Guy, Initial};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::SystemTime;

const BUDGET: usize = 8 * 1024 * 1024;
const MAX_ENTRIES: usize = 32;

#[derive(Clone)]
pub(super) struct Observations {
    seeds: Vec<(i64, u32)>,
    animations: Vec<(i64, i64, i64)>,
    guys: FrameGuys,
}

fn payload_bytes(
    seeds: &Vec<(i64, u32)>,
    animations: &Vec<(i64, i64, i64)>,
    guys: &FrameGuys,
) -> usize {
    seeds.capacity() * std::mem::size_of::<(i64, u32)>()
        + animations.capacity() * std::mem::size_of::<(i64, i64, i64)>()
        + guys.capacity() * std::mem::size_of::<(i64, Vec<FrameUnit>)>()
        + guys
            .iter()
            .map(|(_, units)| {
                units.capacity() * std::mem::size_of::<FrameUnit>()
                    + units
                        .iter()
                        .map(|u| u.guys.capacity() * std::mem::size_of::<Guy>())
                        .sum::<usize>()
            })
            .sum::<usize>()
}

impl Observations {
    fn from_initial(init: &Initial<'_>) -> Self {
        Self {
            seeds: init.frame_seeds.clone(),
            animations: init.anim_lengths.clone(),
            guys: init.frame_guys.clone(),
        }
    }
    fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + payload_bytes(&self.seeds, &self.animations, &self.guys)
    }
    pub(super) fn apply(&self, init: &mut Initial<'_>) {
        init.frame_seeds.clone_from(&self.seeds);
        init.anim_lengths.clone_from(&self.animations);
        init.frame_guys.clone_from(&self.guys);
    }
}

struct Entry {
    path: PathBuf,
    length: u64,
    modified: SystemTime,
    data: Arc<Observations>,
}
impl Entry {
    fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.path.capacity() + self.data.bytes()
    }
}
#[derive(Default)]
struct Cache(Vec<Entry>);
impl Cache {
    fn get(&self, path: &Path, length: u64, modified: SystemTime) -> Option<Arc<Observations>> {
        self.0
            .iter()
            .find(|e| e.path == path && e.length == length && e.modified == modified)
            .map(|e| Arc::clone(&e.data))
    }
    fn insert(&mut self, entry: Entry, budget: usize) {
        if entry.bytes() > budget {
            return;
        }
        self.0.retain(|e| e.path != entry.path);
        self.0.push(entry);
        while self.0.len() > MAX_ENTRIES || self.0.iter().map(Entry::bytes).sum::<usize>() > budget
        {
            self.0.remove(0);
        }
    }
}
static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();

pub(super) fn get(path: &Path, length: u64, modified: SystemTime) -> Option<Arc<Observations>> {
    CACHE
        .get_or_init(Default::default)
        .lock()
        .ok()?
        .get(path, length, modified)
}

pub(super) fn insert(path: &Path, length: u64, modified: SystemTime, init: &Initial<'_>) {
    // Avoid making a second large observation set merely to reject it.
    if payload_bytes(&init.frame_seeds, &init.anim_lengths, &init.frame_guys) > BUDGET {
        return;
    }
    let entry = Entry {
        path: path.to_owned(),
        length,
        modified,
        data: Arc::new(Observations::from_initial(init)),
    };
    if let Ok(mut cache) = CACHE.get_or_init(Default::default).lock() {
        cache.insert(entry, BUDGET);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, UNIX_EPOCH};

    fn entry(path: &str, seed: u32) -> Entry {
        let init = Initial {
            frame_seeds: vec![(4, seed), (4, seed + 1)],
            anim_lengths: vec![(1, 2, 3)],
            frame_guys: vec![(
                4,
                vec![FrameUnit {
                    who: 1,
                    o: 2,
                    guys: vec![Guy {
                        cur_time: Some(3),
                        ..Guy::default()
                    }],
                    ..FrameUnit::default()
                }],
            )],
            ..Initial::default()
        };
        Entry {
            path: path.into(),
            length: 100,
            modified: UNIX_EPOCH,
            data: Arc::new(Observations::from_initial(&init)),
        }
    }

    #[test]
    fn cached_copies_preserve_order_and_do_not_share_mutation() {
        let mut cache = Cache::default();
        cache.insert(entry("one", 7), BUDGET);
        let cached = cache.get(Path::new("one"), 100, UNIX_EPOCH).unwrap();
        let mut first = Initial::default();
        cached.apply(&mut first);
        assert_eq!(first.frame_seeds, [(4, 7), (4, 8)]);
        assert_eq!(first.anim_lengths, [(1, 2, 3)]);
        assert_eq!(first.frame_guys[0].1[0].guys[0].cur_time, Some(3));
        first.frame_seeds[0].1 = 99;
        first.anim_lengths.clear();
        first.frame_guys[0].1[0].guys[0].cur_time = Some(99);
        let mut second = Initial::default();
        cached.apply(&mut second);
        assert_eq!(second.frame_seeds, [(4, 7), (4, 8)]);
        assert_eq!(second.anim_lengths, [(1, 2, 3)]);
        assert_eq!(second.frame_guys[0].1[0].guys[0].cur_time, Some(3));
        assert!(cache.get(Path::new("other"), 100, UNIX_EPOCH).is_none());
        assert!(cache.get(Path::new("one"), 101, UNIX_EPOCH).is_none());
        assert!(
            cache
                .get(Path::new("one"), 100, UNIX_EPOCH + Duration::from_secs(1))
                .is_none()
        );
    }

    #[test]
    fn budget_rejects_oversize_evicts_oldest_and_replaces_paths() {
        let mut cache = Cache::default();
        let budget = entry("one", 7).bytes();
        cache.insert(entry("one", 7), budget - 1);
        assert!(cache.0.is_empty());
        cache.insert(entry("one", 7), budget);
        cache.insert(entry("two", 8), budget);
        assert!(cache.get(Path::new("one"), 100, UNIX_EPOCH).is_none());
        assert!(cache.get(Path::new("two"), 100, UNIX_EPOCH).is_some());
        cache.insert(entry("two", 9), budget);
        assert_eq!(cache.0.len(), 1);
        assert_eq!(cache.0[0].data.seeds[0].1, 9);
        assert!(cache.0.iter().map(Entry::bytes).sum::<usize>() <= budget);
        for i in 0..100 {
            cache.insert(entry(&format!("path-{i}"), 1), BUDGET);
        }
        assert_eq!(cache.0.len(), MAX_ENTRIES);
    }
}
