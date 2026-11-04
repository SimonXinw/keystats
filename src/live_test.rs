use crate::live::{classify, FADE_MS};

#[test]
fn held_and_sticky_stay_lit_then_release_fades() {
    assert_eq!(classify(true, false, None), 1.0);
    assert_eq!(classify(false, true, None), 1.0);
    assert_eq!(classify(true, false, Some(FADE_MS * 4)), 1.0);
    assert_eq!(classify(false, false, Some(0)), 1.0);
    let half = classify(false, false, Some(FADE_MS / 2));
    assert!(half > 0.2 && half < 0.3);
    assert_eq!(classify(false, false, Some(FADE_MS)), 0.0);
    assert_eq!(classify(false, false, None), 0.0);
}
