#[cfg(test)]
mod tests {
    use crate::board;
    use crate::keys::NUMPAD_ENTER;
    use std::collections::HashSet;

    #[test]
    fn full_keyboard_keys_are_unique_and_aligned() {
        let spots = board::keyboard();
        let ids: HashSet<_> = spots.iter().map(|s| s.key).collect();
        assert_eq!(ids.len(), spots.len());
        assert!(spots.len() >= 100 && spots.len() <= 108);
        let at = |key| spots.iter().find(|s| s.key == key).unwrap().x;
        assert_eq!(at(0x2C), at(0x2D));
        assert!(at(0xA0) < at(0xA1));
        assert_eq!(at(0x51), 1.5);
        assert_eq!(at(0x64), 19.0);
        assert_eq!(at(0x26), 16.5);
        assert_eq!(at(0x6B), 22.0);
        assert!(ids.contains(&0x20));
        assert!(ids.contains(&NUMPAD_ENTER));
    }
}
