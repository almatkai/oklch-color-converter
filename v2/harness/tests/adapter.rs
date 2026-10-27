use oklch_beta_contract_harness::{direct, dispatch_supports, CoreContext, CorePaths, Sessions};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
fn context() -> (tempfile::TempDir, CoreContext) {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    std::fs::write(p.join("installed"), b"policy fixture").unwrap();
    let ctx = CoreContext::new(
        CorePaths {
            data_dir: p.into(),
            config_dir: p.join("config"),
            resource_dir: p.join("resources"),
            temp_dir: p.join("tmp"),
            home_dir: p.into(),
        },
        None,
    );
    (dir, ctx)
}
async fn start(ctx: &CoreContext, text: &str) -> (Value, Value) {
    let mut state = json!({});
    direct(
        ctx,
        "extension:run-command",
        &json!({"argumentValues":{"color":"red"},"preferences":{"ignored":true}}),
        &mut state,
    )
    .await
    .unwrap();
    let view = direct(
        ctx,
        "extension:search-text-changed",
        &json!({"searchText":text}),
        &mut state,
    )
    .await
    .unwrap();
    (state, view)
}
fn id(view: &Value, index: usize) -> String {
    view["actions"][index]["id"].as_str().unwrap().into()
}
type Calls = Arc<Mutex<Vec<(String, Vec<Value>)>>>;
fn route(ctx: &CoreContext, replies: Vec<Result<Value, String>>) -> Calls {
    let calls = Arc::new(Mutex::new(vec![]));
    let q = Arc::new(Mutex::new(std::collections::VecDeque::from(replies)));
    let log = calls.clone();
    ctx.set_router(Arc::new(move |channel, args| {
        log.lock().unwrap().push((channel, args));
        let reply = q
            .lock()
            .unwrap()
            .pop_front()
            .expect("Unexpected host request");
        Box::pin(async move { reply })
    }))
    .unwrap();
    calls
}
#[tokio::test]
async fn rendered_rows_order_tags_swatches_and_canonical_actions() {
    let (_dir, ctx) = context();
    let calls = route(
        &ctx,
        vec![
            Ok(json!({"ok":true})),
            Ok(json!({"ok":true,"error":null,"success":true})),
        ],
    );
    let (mut state, v) = start(&ctx, "oklch(53% .5 118.34 / .5)").await;
    assert_eq!(v["root"]["props"]["__hasServerSearch"], true);
    assert_eq!(v["root"]["props"]["filtering"], false);
    let rows = v["root"]["children"][0]["children"].as_array().unwrap();
    assert_eq!(rows.len(), 8);
    assert_eq!(v["actions"].as_array().unwrap().len(), 16);
    assert_eq!(rows[0]["props"]["subtitle"], "figmaP3");
    assert_eq!(rows[1]["props"]["accessories"][0]["text"], "out of p3");
    assert_eq!(rows[0]["props"]["icon"]["source"], "Icon.CircleFilled");
    assert_eq!(v["actions"][0]["title"], "Copy Color Value");
    assert_eq!(v["actions"][1]["title"], "Copy as Hex");
    assert!(v["actions"][0].get("shortcut").is_none());
    let first = id(&v, 0);
    let copied = direct(
        &ctx,
        "extension:invoke-action",
        &json!({"actionId":first,"content":"forged","formValues":{"value":"forged"}}),
        &mut state,
    )
    .await
    .unwrap();
    assert_eq!(copied["message"], "Copied to clipboard");
    let current = id(&copied, 1);
    direct(
        &ctx,
        "extension:invoke-action",
        &json!({"actionId":current}),
        &mut state,
    )
    .await
    .unwrap();
    assert!(direct(
        &ctx,
        "extension:invoke-action",
        &json!({"actionId":first}),
        &mut state
    )
    .await
    .is_err());
    let calls = calls.lock().unwrap();
    assert_eq!(
        calls[0],
        (
            "clipboard:write".into(),
            vec![rows[0]["props"]["title"].clone()]
        )
    );
    assert_eq!(
        calls[1],
        (
            "clipboard:write".into(),
            vec![rows[0]["props"]["icon"]["tintColor"].clone()]
        )
    );
}
#[tokio::test]
async fn strict_acknowledgements_private_errors_and_deliberate_retry() {
    let rejected = vec![
        Ok(Value::Null),
        Ok(json!(true)),
        Ok(json!(false)),
        Ok(json!({})),
        Ok(json!({"ok":true,"error":"PRIVATE clipboard secret"})),
        Ok(json!({"ok":true,"error":false})),
        Ok(json!({"ok":true,"success":false})),
        Ok(json!({"ok":true,"success":null})),
        Ok(json!({"ok":"true"})),
        Err("PRIVATE transport detail".into()),
    ];
    for reply in rejected {
        let (_dir, ctx) = context();
        let calls = route(&ctx, vec![reply, Ok(json!({"ok":true}))]);
        let (mut state, v) = start(&ctx, "red").await;
        let first = id(&v, 0);
        let failed = direct(
            &ctx,
            "extension:invoke-action",
            &json!({"actionId":first}),
            &mut state,
        )
        .await
        .unwrap();
        assert!(failed["message"]
            .as_str()
            .unwrap()
            .starts_with("Failed to confirm"));
        assert!(!failed.to_string().contains("PRIVATE"));
        assert_eq!(failed["actions"][0]["title"], "Retry Copy Color Value");
        let saved = state.clone();
        let refresh = direct(&ctx, "extension:refresh-session", &json!({}), &mut state)
            .await
            .unwrap();
        assert_eq!(state, saved);
        assert_eq!(refresh["actions"], failed["actions"]);
        assert_eq!(calls.lock().unwrap().len(), 1);
        assert!(direct(
            &ctx,
            "extension:invoke-action",
            &json!({"actionId":first}),
            &mut state
        )
        .await
        .is_err());
        let retry = id(&failed, 0);
        let ack = direct(
            &ctx,
            "extension:invoke-action",
            &json!({"actionId":retry}),
            &mut state,
        )
        .await
        .unwrap();
        assert_eq!(ack["message"], "Copied to clipboard");
        assert_eq!(calls.lock().unwrap().len(), 2);
    }
}
#[tokio::test]
async fn clones_stale_cross_session_and_refresh_capabilities() {
    let (_dir, ctx) = context();
    let calls = route(&ctx, vec![]);
    let (mut committed, initial) = start(&ctx, "red").await;
    let mut discarded = committed.clone();
    let staged = direct(
        &ctx,
        "extension:search-text-changed",
        &json!({"searchText":"blue"}),
        &mut discarded,
    )
    .await
    .unwrap();
    let next = direct(
        &ctx,
        "extension:search-text-changed",
        &json!({"searchText":"blue"}),
        &mut committed,
    )
    .await
    .unwrap();
    let (mut other, other_view) = start(&ctx, "blue").await;
    for stale in [
        id(&initial, 0),
        id(&staged, 0),
        id(&other_view, 0),
        "forged".into(),
    ] {
        assert!(direct(
            &ctx,
            "extension:invoke-action",
            &json!({"actionId":stale}),
            &mut committed
        )
        .await
        .is_err());
    }
    assert_ne!(id(&staged, 0), id(&next, 0));
    let before = committed.clone();
    let mut refresh_clone = committed.clone();
    let refreshed = direct(
        &ctx,
        "extension:refresh-session",
        &json!({}),
        &mut refresh_clone,
    )
    .await
    .unwrap();
    assert_eq!(refresh_clone, before);
    assert_eq!(refreshed["actions"], next["actions"]);
    assert!(calls.lock().unwrap().is_empty());
    direct(&ctx, "extension:dispose-session", &json!({}), &mut other)
        .await
        .unwrap();
    assert!(
        direct(&ctx, "extension:refresh-session", &json!({}), &mut other)
            .await
            .is_err()
    );
}
#[tokio::test]
async fn checkpoint_survives_cancelled_effect_and_preserves_prior_ack() {
    let (_dir, ctx) = context();
    let count = Arc::new(Mutex::new(0));
    let n = count.clone();
    ctx.set_router(Arc::new(move |channel, args| {
        assert_eq!(channel, "clipboard:write");
        assert_eq!(args.len(), 1);
        let mut n = n.lock().unwrap();
        *n += 1;
        let pending = *n == 2;
        Box::pin(async move {
            if pending {
                std::future::pending::<()>().await;
            }
            Ok(json!({"ok":true}))
        })
    }))
    .unwrap();
    let (mut state, v) = start(&ctx, "red").await;
    let ack = direct(
        &ctx,
        "extension:invoke-action",
        &json!({"actionId":id(&v,0)}),
        &mut state,
    )
    .await
    .unwrap();
    let before = state["actions"][0].clone();
    let payload = json!({"actionId":id(&ack,1)});
    {
        let future = direct(&ctx, "extension:invoke-action", &payload, &mut state);
        tokio::select! {_ = future=>panic!("pending effect unexpectedly completed"),_ = tokio::time::sleep(std::time::Duration::from_millis(10))=>{}}
    }
    assert_eq!(state["actions"][0], before);
    assert_eq!(state["actions"][1]["stage"], "Intent");
    let refreshed = direct(&ctx, "extension:refresh-session", &json!({}), &mut state)
        .await
        .unwrap();
    assert_eq!(refreshed["actions"][1]["title"], "Retry Copy as Hex");
    assert_eq!(*count.lock().unwrap(), 2);
    assert!(
        direct(&ctx, "extension:invoke-action", &payload, &mut state)
            .await
            .is_err()
    );
    let retry = json!({"actionId":id(&refreshed,1)});
    direct(&ctx, "extension:invoke-action", &retry, &mut state)
        .await
        .unwrap();
    assert_eq!(*count.lock().unwrap(), 3);
}
#[tokio::test]
async fn actual_beta_native_sessions_and_runtime_aliases() {
    for alias in [
        "com.tezbar.native.oklch-color-converter",
        "raycast.oklch-color-converter",
        "oklch-color-converter",
    ] {
        assert!(dispatch_supports(alias, "convert-color"));
    }
    assert!(!dispatch_supports(
        "raycast.oklch-color-converter",
        "invented"
    ));
    let (_dir, ctx) = context();
    let calls = route(&ctx, vec![Ok(json!({"ok":true}))]);
    let sessions = Sessions::default();
    let launch=sessions.handle(&ctx,"extension:run-command",json!({"extensionId":"com.tezbar.native.oklch-color-converter","commandName":"convert-color"})).await;
    assert_eq!(launch["ok"], true);
    let sid = launch["sessionId"].clone();
    let search = sessions
        .handle(
            &ctx,
            "extension:search-text-changed",
            json!({"sessionId":sid,"searchText":"red"}),
        )
        .await;
    let refresh = sessions
        .handle(&ctx, "extension:refresh-session", json!({"sessionId":sid}))
        .await;
    assert_eq!(search["actions"], refresh["actions"]);
    let alias = sessions
        .handle(
            &ctx,
            "extension:run-command",
            json!({"extensionId":"raycast.oklch-color-converter","commandName":"convert-color"}),
        )
        .await;
    assert_eq!(alias["ok"], true);
    assert_ne!(sid, alias["sessionId"]);
    let foreign = sessions
        .handle(
            &ctx,
            "extension:invoke-action",
            json!({"sessionId":alias["sessionId"],"actionId":id(&search,0)}),
        )
        .await;
    assert_eq!(foreign["ok"], false);
    let ack = sessions
        .handle(
            &ctx,
            "extension:invoke-action",
            json!({"sessionId":sid,"actionId":id(&search,0)}),
        )
        .await;
    assert_eq!(ack["message"], "Copied to clipboard");
    assert_eq!(calls.lock().unwrap().len(), 1);
    assert_eq!(
        sessions
            .handle(&ctx, "extension:dispose-session", json!({"sessionId":sid}))
            .await,
        true
    );
    assert_eq!(
        sessions
            .handle(
                &ctx,
                "extension:invoke-action",
                json!({"sessionId":sid,"actionId":id(&ack,0)})
            )
            .await["ok"],
        false
    );
    sessions.clear_slug().await;
    sessions.clear().await;
}
#[tokio::test]
async fn invalid_input_state_and_no_implicit_reads() {
    let (_dir, ctx) = context();
    let calls = route(&ctx, vec![]);
    let mut state = json!({});
    let launch = direct(
        &ctx,
        "extension:run-command",
        &json!({"argumentValues":{"value":"red"},"preferences":{}}),
        &mut state,
    )
    .await
    .unwrap();
    assert_eq!(launch["actions"], json!([]));
    assert!(direct(
        &ctx,
        "extension:search-text-changed",
        &json!({"searchText":32}),
        &mut state
    )
    .await
    .is_err());
    let before = state.clone();
    assert!(direct(
        &ctx,
        "extension:search-text-changed",
        &json!({"searchText":"x".repeat(65537)}),
        &mut state
    )
    .await
    .is_err());
    assert_eq!(state, before);
    state["rows"] = json!("corrupt");
    assert!(
        direct(&ctx, "extension:refresh-session", &json!({}), &mut state)
            .await
            .is_err()
    );
    assert!(calls.lock().unwrap().is_empty());
}
