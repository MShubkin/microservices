use std::sync::{Mutex, MutexGuard, OnceLock};

#[derive(Debug, Default)]
struct FreeIds {
    free: Vec<u16>,
    next: u16,
}

impl FreeIds {
    fn new() -> FreeIds {
        FreeIds::default()
    }

    fn next(&mut self) -> FreeId {
        if let Some(id) = self.free.pop() {
            FreeId { id, is_new: false }
        } else {
            let id = self.next;
            self.next += 1;
            FreeId { id, is_new: true }
        }
    }

    fn release(&mut self, id: &FreeId) {
        self.free.push(id.id);
    }
}

static FREE_IDS: OnceLock<Mutex<FreeIds>> = OnceLock::new();

fn free_ids() -> MutexGuard<'static, FreeIds> {
    FREE_IDS
        .get_or_init(|| Mutex::new(FreeIds::new()))
        .lock()
        .expect("no poison")
}

pub struct FreeId {
    id: u16,
    is_new: bool,
}

impl FreeId {
    pub fn new() -> FreeId {
        free_ids().next()
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn is_new(&self) -> bool {
        self.is_new
    }
}

impl Default for FreeId {
    fn default() -> Self {
        FreeId::new()
    }
}

impl Drop for FreeId {
    fn drop(&mut self) {
        free_ids().release(self);
    }
}
