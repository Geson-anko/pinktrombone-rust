use crate::consts::MAX_TRANSIENTS;

pub struct Transient {
    pub position: usize,
    pub time_alive: f64,
    pub lifetime: f64,
    pub strength: f64,
    pub exponent: f64,
    pub is_free: bool,
    pub id: usize,
}

impl Transient {
    pub fn new(id: usize) -> Self {
        Transient {
            position: 0,
            time_alive: 0.0,
            lifetime: 0.0,
            strength: 0.0,
            exponent: 0.0,
            is_free: true,
            id,
        }
    }
}
pub struct TransientPool {
    pool: Vec<Transient>,
    free_ids: Vec<usize>,
}

impl TransientPool {
    pub fn new() -> Self {
        let mut pool = Vec::with_capacity(MAX_TRANSIENTS);
        let mut free_ids = Vec::with_capacity(MAX_TRANSIENTS);
        for i in (0..MAX_TRANSIENTS).rev() {
            pool.push(Transient::new(i));
            free_ids.push(i);
        }
        TransientPool { pool, free_ids }
    }

    pub fn append(&mut self, position: usize) {
        if let Some(free_id) = self.free_ids.pop() {
            let t = &mut self.pool[free_id as usize];
            t.is_free = false;
            t.time_alive = 0.0;

            t.lifetime = 0.2;
            t.strength = 0.3;
            t.exponent = 200.0;
            t.position = position;
        }
    }

    pub fn remove(&mut self, id: usize) {
        if !self.pool[id as usize].is_free {
            self.pool[id as usize].is_free = true;
            self.free_ids.push(id);
        }
    }

    pub fn size(&self) -> usize {
        MAX_TRANSIENTS - self.free_ids.len()
    }

    pub fn get_valid_transients(&mut self) -> Vec<&mut Transient> {
        self.pool.iter_mut().filter(|t| !t.is_free).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transient_new() {
        let t = Transient::new(1);
        assert_eq!(t.id, 1);
        assert_eq!(t.position, 0);
        assert_eq!(t.time_alive, 0.0);
        assert!(t.is_free);
    }

    #[test]
    fn test_transient_pool_new() {
        let pool = TransientPool::new();
        assert_eq!(pool.size(), 0);
        assert_eq!(pool.free_ids.len(), MAX_TRANSIENTS);
    }

    #[test]
    fn test_transient_pool_append() {
        let mut pool = TransientPool::new();
        pool.append(5);
        assert_eq!(pool.size(), 1);
        let valid = pool.get_valid_transients();
        assert_eq!(valid.len(), 1);
        assert_eq!(valid[0].position, 5);
    }

    #[test]
    fn test_transient_pool_remove() {
        let mut pool = TransientPool::new();
        pool.append(5);
        assert_eq!(pool.size(), 1);
        pool.remove(0);
        assert_eq!(pool.size(), 0);
    }

    #[test]
    fn test_transient_pool_max_capacity() {
        let mut pool = TransientPool::new();
        for i in 0..MAX_TRANSIENTS + 1 {
            // Try to append one more than max
            pool.append(i);
        }
        assert_eq!(pool.size(), MAX_TRANSIENTS);
    }
}
