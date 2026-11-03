use std::time::Instant;

/// 最近 60 秒的次数。只在计数线程里更新，每个格子是一秒。
pub struct Pace {
    origin: Instant,
    all: [u32; 60],
    own: [u32; 60],
    cursor: u64,
}

impl Pace {
    pub fn new() -> Self {
        Self {
            origin: Instant::now(),
            all: [0; 60],
            own: [0; 60],
            cursor: 0,
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn hit(&mut self, own: bool) {
        let sec = self.origin.elapsed().as_secs();
        self.roll(sec);
        let slot = &mut self.all[(sec % 60) as usize];
        *slot = slot.saturating_add(1);
        if own {
            let slot = &mut self.own[(sec % 60) as usize];
            *slot = slot.saturating_add(1);
        }
    }

    pub fn recent(&mut self, count_own: bool) -> u64 {
        self.roll(self.origin.elapsed().as_secs());
        let all = sum(&self.all);
        if count_own {
            all
        } else {
            all.saturating_sub(sum(&self.own))
        }
    }

    fn roll(&mut self, sec: u64) {
        let jump = sec.saturating_sub(self.cursor);
        if jump == 0 {
            return;
        }
        if jump >= 60 {
            self.all = [0; 60];
            self.own = [0; 60];
        } else {
            for step in 1..=jump {
                let index = ((self.cursor + step) % 60) as usize;
                self.all[index] = 0;
                self.own[index] = 0;
            }
        }
        self.cursor = sec;
    }
}

fn sum(buckets: &[u32; 60]) -> u64 {
    buckets.iter().map(|n| *n as u64).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn recent_window_drops_own_keys_and_expires() {
        let mut pace = Pace::new();
        pace.hit(false);
        pace.hit(true);
        pace.hit(true);
        assert_eq!(pace.recent(false), 1);
        assert_eq!(pace.recent(true), 3);
        pace.origin = Instant::now() - Duration::from_secs(120);
        assert_eq!(pace.recent(true), 0);
    }
}
