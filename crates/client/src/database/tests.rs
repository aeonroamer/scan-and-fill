use std::sync::Arc;

use super::*;

fn temp_db(name: &str) -> Db {
    let path = std::env::temp_dir().join(format!(
        "scanfill-test-{}-{name}.redb",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);
    Arc::new(redb::Database::create(path).unwrap())
}

fn block<T>(future: impl Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(future)
}

fn session(key: &str, updated_at: i64) -> BulkSession {
    BulkSession {
        key: key.into(),
        name: format!("Session {key}"),
        created_at: 0,
        updated_at,
        cursor: Some(0),
        bills: vec![StoredBill {
            reference: "04154374153178".into(),
            source: "scanned".into(),
            format: Some("electricity".into()),
            label: Some("Electricity".into()),
            company: Some("MEPCO".into()),
            late_fee: false,
            original_amount: Some(2130),
            amount: Some(2130),
            paid: Some(false),
        }],
    }
}

#[test]
fn fresh_database_gives_default_settings() {
    let db = temp_db("fresh");
    let settings = block(load_settings(&db)).unwrap();
    assert!(settings.contact.is_none());
    assert_eq!(settings.contact(), DEFAULT_CONTACT);
    assert!(block(list_sessions(&db)).unwrap().is_empty());
}

#[test]
fn settings_round_trip() {
    let db = temp_db("settings");
    let settings = Settings {
        contact: Some("03001234567".into()),
        software: Some("easypaisa".into()),
        step_delay_ms: Some(5),
        sound: Some(false),
        trigger: Some("left".into()),
    };
    block(save_settings(&db, settings)).unwrap();

    let loaded = block(load_settings(&db)).unwrap();
    assert_eq!(loaded.contact.as_deref(), Some("03001234567"));
    assert_eq!(loaded.software.as_deref(), Some("easypaisa"));
    assert_eq!(loaded.step_delay_ms, Some(5));
    assert_eq!(loaded.sound, Some(false));
    assert_eq!(loaded.trigger.as_deref(), Some("left"));
}

#[test]
fn sessions_list_newest_first_and_delete() {
    let db = temp_db("sessions");
    block(save_session(&db, session("old", 10))).unwrap();
    block(save_session(&db, session("new", 20))).unwrap();

    let keys: Vec<_> = block(list_sessions(&db))
        .unwrap()
        .into_iter()
        .map(|session| session.key)
        .collect();
    assert_eq!(keys, ["new", "old"]);

    block(delete_session(&db, "old".into())).unwrap();
    let sessions = block(list_sessions(&db)).unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].key, "new");
    assert_eq!(sessions[0].bills[0].company.as_deref(), Some("MEPCO"));
}

#[test]
fn saving_the_same_key_overwrites() {
    let db = temp_db("overwrite");
    block(save_session(&db, session("a", 1))).unwrap();

    let mut renamed = session("a", 2);
    renamed.name = "Renamed".into();
    block(save_session(&db, renamed)).unwrap();

    let sessions = block(list_sessions(&db)).unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].name, "Renamed");
}

#[test]
fn old_session_without_new_fields_still_loads() {
    let old = r#"{"key":"a","name":"Old","created_at":1,"updated_at":2,"cursor":null,
        "bills":[{"reference":"123","source":"manual","format":null,"label":null,
        "late_fee":false,"original_amount":null,"amount":null}]}"#;
    let session: BulkSession = serde_json::from_str(old).unwrap();
    assert!(session.bills[0].company.is_none());
    assert!(session.bills[0].paid.is_none());
}
