#[cfg(test)]
mod tests {
    use crate::keys;
    use crate::stats::Stats;
    use std::sync::Arc;
    use std::time::Duration;

    fn own_name() -> String {
        std::env::current_exe()
            .ok()
            .and_then(|path| path.file_name().map(|name| name.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "keystats.exe".into())
    }

    #[test]
    fn counts_each_key_and_drops_stale_session() {
        let stats = Stats::new();
        stats.start();
        let session = stats.session();
        stats.ingest(session, Arc::from("League of Legends.exe"), 0x51);
        stats.ingest(session, Arc::from("League of Legends.exe"), 0x51);
        stats.ingest(session, Arc::from("League of Legends.exe"), keys::MOUSE_RIGHT);
        stats.ingest(999, Arc::from("other.exe"), 0x41);
        let snap = stats.snapshot();
        assert!(snap.running);
        assert_eq!(snap.keyboard, 2);
        assert_eq!(snap.mouse, 1);
        assert_eq!(snap.recent, 3);
        assert_eq!(snap.rows.len(), 2);
        assert_eq!(snap.rows[0].key, 0x51);
        stats.pause();
        assert_eq!(stats.session(), 0);
        stats.ingest(session, Arc::from("League of Legends.exe"), 0x57);
        assert_eq!(stats.snapshot().keyboard, 3);
        stats.reset();
        assert!(stats.snapshot().rows.is_empty());
        stats.ingest(session, Arc::from("League of Legends.exe"), 0x57);
        assert!(stats.snapshot().rows.is_empty());
    }

    #[test]
    fn own_window_stays_out_until_asked_and_restore_keeps_counts() {
        let stats = Stats::new();
        stats.start();
        let session = stats.session();
        stats.ingest(session, Arc::from("Code.exe"), 0x41);
        stats.ingest(session, Arc::from(own_name()), 0x42);
        let hidden = stats.snapshot();
        assert_eq!(hidden.keyboard, 1);
        assert_eq!(hidden.recent, 1);
        stats.set_count_self(true);
        assert_eq!(stats.snapshot().keyboard, 2);
        assert_eq!(stats.snapshot().recent, 2);
        let saved = stats.dump();
        let again = Stats::new();
        again.restore(Duration::from_secs(90), 4, saved.rows);
        again.set_count_self(true);
        let snap = again.snapshot();
        assert!(!snap.running);
        assert_eq!(snap.elapsed, Duration::from_secs(90));
        assert_eq!(snap.dropped, 4);
        assert_eq!(snap.keyboard, 2);
        assert_eq!(snap.recent, 0);
    }
}
