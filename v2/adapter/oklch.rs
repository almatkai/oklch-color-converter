//! Compiled-in beta adapter. This file is included by extensions/mod.rs.
//! See integration/README.md. Original command: strongeron.
use super::v2_sdk as sdk;
use crate::native_core::common::{CoreContext, CoreResult};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    OnceLock,
};
use tezbar_oklch_color_converter::{convert, ColorData};

pub fn commands() -> &'static [&'static str] {
    &["convert-color"]
}
fn token() -> String {
    static NONCE: OnceLock<String> = OnceLock::new();
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let n = NEXT
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
        .expect("Native action token space exhausted");
    format!(
        "ocv:{}:{n}",
        NONCE.get_or_init(|| uuid::Uuid::new_v4().to_string())
    )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Session {
    version: u8,
    instance: String,
    input: String,
    rows: Vec<ColorData>,
    actions: Vec<CopyAction>,
    notice: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CopyAction {
    id: String,
    row: usize,
    hex: bool,
    stage: Stage,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum Stage {
    Ready,
    Intent,
    Uncertain,
    Acknowledged,
}
impl Session {
    fn create(input: &str) -> Result<Self, String> {
        let rows = convert(input)?;
        let actions = (0..rows.len())
            .flat_map(|row| {
                [false, true].map(move |hex| CopyAction {
                    id: token(),
                    row,
                    hex,
                    stage: Stage::Ready,
                })
            })
            .collect();
        Ok(Self {
            version: 1,
            instance: token(),
            input: input.into(),
            rows,
            actions,
            notice: String::new(),
        })
    }
    fn read(state: &Value) -> Result<Self, String> {
        let s: Self = serde_json::from_value(state.clone())
            .map_err(|_| "Color converter session is invalid; reopen the command.")?;
        if s.version != 1
            || s.instance.is_empty()
            || s.rows != convert(&s.input)?
            || s.actions.len() != s.rows.len() * 2
        {
            return Err("Color converter session is invalid; reopen the command.".into());
        }
        let mut ids = std::collections::HashSet::new();
        for (i, a) in s.actions.iter().enumerate() {
            if a.row != i / 2
                || a.hex != (i % 2 == 1)
                || !a.id.starts_with("ocv:")
                || !ids.insert(&a.id)
            {
                return Err("Color converter session is invalid; reopen the command.".into());
            }
        }
        Ok(s)
    }
    fn save(&self, state: &mut Value) -> Result<(), String> {
        *state =
            serde_json::to_value(self).map_err(|_| "Could not prepare color converter session.")?;
        Ok(())
    }
    fn view(&self) -> Value {
        let mut descriptors = Vec::new();
        let mut children = Vec::new();
        for (i, row) in self.rows.iter().enumerate() {
            let ids: Vec<_> = self
                .actions
                .iter()
                .filter(|a| a.row == i)
                .map(|a| a.id.clone())
                .collect();
            children.push(json!({"type":"List.Item","props":{"id":format!("color:{}",row.format),"title":row.value,"subtitle":row.format,"icon":{"source":"Icon.CircleFilled","tintColor":row.hex_value},"accessories":[{"text":row.tag_text}],"actionIds":ids}}));
        }
        for a in &self.actions {
            let base = if a.hex {
                "Copy as Hex"
            } else {
                "Copy Color Value"
            };
            let retry = matches!(a.stage, Stage::Intent | Stage::Uncertain);
            descriptors.push(json!({"id":a.id,"title":if retry{format!("Retry {base}")}else{base.into()},"kind":"action"}));
        }
        let mut v = sdk::view(
            "Color Converter",
            json!({"type":"List","props":{"navigationTitle":"Color Converter","searchBarPlaceholder":"Enter color...","__hasServerSearch":true,"filtering":false,"isLoading":false},"children":[{"type":"List.Section","children":children}]}),
            descriptors,
        );
        if !self.notice.is_empty() {
            v["message"] = json!(self.notice);
        }
        v
    }
}
fn acknowledged(reply: &Value) -> bool {
    reply.is_object()
        && reply.get("ok") == Some(&Value::Bool(true))
        && reply.get("error").is_none_or(Value::is_null)
        && reply.get("success").is_none_or(|v| v == &Value::Bool(true))
}

pub async fn handle(
    ctx: &CoreContext,
    command: &str,
    operation: &str,
    payload: &Value,
    state: &mut Value,
) -> CoreResult {
    if !commands().contains(&command) {
        return Err("Unknown color converter command.".into());
    }
    match operation {
        "extension:run-command" => {
            // V1 declares neither arguments nor preferences and starts with empty search.
            let s = Session::create("")?;
            s.save(state)?;
            Ok(s.view())
        }
        "extension:dispose-session" => {
            *state = Value::Null;
            Ok(sdk::unchanged())
        }
        "extension:search-text-changed" => {
            let _ = Session::read(state)?;
            let input = payload
                .get("searchText")
                .and_then(Value::as_str)
                .ok_or("Color search text must be a string.")?;
            let s = Session::create(input)?;
            s.save(state)?;
            Ok(s.view())
        }
        "extension:refresh-session" => {
            // NativeSessions may discard a refresh clone. Rendering never mints IDs
            // or mutates state, so every returned capability is already committed.
            let s = Session::read(state)?;
            Ok(s.view())
        }
        "extension:load-more" => {
            let _ = Session::read(state)?;
            Ok(sdk::unchanged())
        }
        "extension:invoke-action" => {
            let mut s = Session::read(state)?;
            let id = payload
                .get("actionId")
                .and_then(Value::as_str)
                .ok_or("Missing color action.")?;
            if id.len() > 128 {
                return Err("Color action is stale or invalid; use the current results.".into());
            }
            let index = s
                .actions
                .iter()
                .position(|a| a.id == id)
                .ok_or("Color action is stale or invalid; use the current results.")?;
            // Ignore caller-supplied content/formValues. Only committed canonical rows
            // determine clipboard text; stale/cross-session/forged IDs have no effect.
            let action = &s.actions[index];
            let row = &s.rows[action.row];
            let text = if action.hex {
                row.hex_value.clone()
            } else {
                row.value.clone()
            };
            s.actions[index].id = token();
            s.actions[index].stage = Stage::Intent;
            s.notice =
                "Copy result is uncertain. Use the explicit Retry action to try again.".into();
            s.save(state)?; // checkpoint before the external effect (also survives cancellation)
            let result = sdk::copy(ctx, &text).await;
            if result.as_ref().is_ok_and(acknowledged) {
                s.actions[index].stage = Stage::Acknowledged;
                s.notice = "Copied to clipboard".into();
            } else {
                s.actions[index].stage = Stage::Uncertain;
                s.notice="Failed to confirm copy. It may have reached the clipboard; use Retry only if needed.".into();
            }
            s.save(state)?;
            Ok(s.view())
        }
        _ => Err("Unsupported color converter operation.".into()),
    }
}
