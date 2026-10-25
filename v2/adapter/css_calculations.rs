//! Production sibling module for native_core::extensions, compiled by validation/host.
use super::v2_sdk;
use crate::native_core::common::{CoreContext, CoreResult};
use css_calculations_native_v2::{calculate, validate, DEFAULTS, FIELDS, MAX_FIELD_BYTES, TITLES};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);
fn nonce() -> Result<u64, String> {
    NEXT.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
        .map_err(|_| "CSS action capability space exhausted; restart the application".into())
}
pub fn commands() -> &'static [&'static str] {
    &["clamp_px", "clamp_rem"]
}
fn binding(state: &Value, name: &str) -> String {
    format!("css:{}:{name}", state["nonce"].as_u64().unwrap_or(0))
}
fn ack(value: &Value) -> bool {
    value.is_object()
        && value["ok"] == true
        && value.get("error").is_none_or(Value::is_null)
        && value.get("success").is_none_or(|v| v == true)
}
fn form(state: &Value, controlled: Option<&str>, initial: bool) -> Value {
    let rem = state["command"] == "clamp_rem";
    let mut children: Vec<_> = FIELDS.iter().enumerate().map(|(i,key)| {
        let placeholder = if i == 4 { "Pixels Per Rem".to_owned() }
            else { format!("{} Value In {}", if i % 2 == 0 { "Minimal" } else { "Maximal" }, if rem { "Rems" } else { "Pixels" }) };
        let mut props = json!({"id":key,"title":TITLES[i],"placeholder":placeholder,"actionId":binding(state,key)});
        if initial || controlled == Some(*key) { props["value"] = state["drafts"][key].clone(); }
        if let Some(error) = state["errors"].get(key).and_then(Value::as_str) { props["error"] = json!(error); }
        json!({"type":"Form.TextField","props":props,"children":[]})
    }).collect();
    if let Some(message) = state["message"].as_str() {
        children.push(json!({"type":"Form.Description","props":{"text":message},"children":[]}));
    }
    if let Some(output) = state["effect"]["css"].as_str() {
        children.push(json!({"type":"Form.Description","props":{"title":"Clamp","text":output},"children":[]}));
    }
    let mut actions =
        vec![json!({"id":binding(state,"submit"),"title":"Copy Clamp","kind":"submit-form"})];
    if state["effect"]["status"] == "uncertain" || state["effect"]["status"] == "pending" {
        if let Some(retry) = state["effect"]["retry"].as_str() {
            actions.push(json!({"id":retry,"title":"Retry Copy (Previous Clamp)","kind":"action"}));
        }
    }
    v2_sdk::view(
        if rem {
            "Calculate Clamp REM"
        } else {
            "Calculate Clamp PX"
        },
        json!({"type":"Form","props":{},"children":children}),
        actions,
    )
}
fn text(value: &Value) -> Result<&str, String> {
    let text = value.as_str().ok_or("CSS field value must be a string")?;
    if text.len() > MAX_FIELD_BYTES {
        return Err("CSS field exceeds the 65536-byte resource limit".into());
    }
    Ok(text)
}
async fn copy(ctx: &CoreContext, state: &mut Value, css: &str) -> CoreResult {
    // Commit intent before awaiting. Cancellation cannot silently authorize an automatic repeat.
    state["effect"] =
        json!({"css":css,"status":"pending","retry":format!("css-retry:{}",nonce()?)});
    let result = v2_sdk::copy(ctx, css).await;
    if result.as_ref().is_ok_and(ack) {
        state["effect"]["status"] = json!("acknowledged");
        state["message"] = json!("Clamp Copied 🎊");
    } else {
        state["effect"]["status"] = json!("uncertain");
        // Router errors can contain private paths/control bytes. Never echo untrusted errors.
        state["message"] = json!("Copy was not acknowledged. It may have completed. Use Retry Copy to repeat deliberately.");
    }
    Ok(form(state, None, false))
}
pub async fn handle(
    ctx: &CoreContext,
    command: &str,
    operation: &str,
    payload: &Value,
    state: &mut Value,
) -> CoreResult {
    if !commands().contains(&command) {
        return Err("Unknown CSS Calculations command".into());
    }
    match operation {
        "extension:run-command" => {
            let mut drafts = json!({});
            for (i, key) in FIELDS.iter().enumerate() {
                drafts[key] = json!(DEFAULTS[i]);
            }
            // Source declares no launch arguments or preferences. Do not infer values from them.
            *state = json!({"nonce":nonce()?,"command":command,"drafts":drafts,"errors":{}});
            return Ok(form(state, None, true));
        }
        "extension:dispose-session" => {
            *state = Value::Null;
            return Ok(v2_sdk::no_view("CSS Calculations session disposed"));
        }
        _ => (),
    }
    if state["nonce"].as_u64().is_none() || state["command"] != command {
        return Err("CSS Calculations session is no longer active".into());
    }
    match operation {
        "extension:refresh-session" | "extension:search-text-changed" | "extension:load-more" => {
            Ok(v2_sdk::unchanged())
        }
        "extension:invoke-action" => {
            let id = payload["actionId"]
                .as_str()
                .ok_or("Missing CSS action ID")?;
            if let Some(key) = FIELDS.iter().find(|key| id == binding(state, key)) {
                let values = payload["formValues"]
                    .as_object()
                    .ok_or("Missing CSS change values")?;
                if values.len() != 1 || !values.contains_key("value") {
                    return Err("CSS changes require formValues.value only".into());
                }
                let value = text(&values["value"])?;
                let error = validate(key, &json!(value), true);
                state["drafts"][key] = json!(value);
                state["errors"][key] = json!(error);
                return Ok(form(state, Some(key), false));
            }
            if id == binding(state, "submit") {
                let values = payload["formValues"]
                    .as_object()
                    .ok_or("Missing CSS form values")?;
                if values.len() != 5 || FIELDS.iter().any(|k| !values.contains_key(*k)) {
                    return Err("CSS submission requires all five fields".into());
                }
                // Validate transport fully before touching drafts/bindings.
                for key in FIELDS {
                    text(&values[key])?;
                }
                state["drafts"] = json!(values);
                let mut invalid = false;
                for key in FIELDS {
                    if let Some(error) = validate(key, &values[key], false) {
                        invalid = true;
                        // Preserve own hooks.ts setErrorByKey mismatch for viewport/base keys.
                        if key == "min_font_size" || key == "max_font_size" {
                            state["errors"][key] = json!(error);
                        }
                    }
                }
                if invalid {
                    return Ok(form(state, None, false));
                }
                let css = calculate(
                    &json!(values),
                    if command == "clamp_px" { "px" } else { "rem" },
                );
                if (state["effect"]["status"] == "uncertain"
                    || state["effect"]["status"] == "pending")
                    && state["effect"]["css"] == css
                {
                    state["message"] =
                        json!("Previous copy is uncertain. Use Retry Copy to repeat deliberately.");
                    return Ok(form(state, None, false));
                }
                return copy(ctx, state, &css).await;
            }
            if state["effect"]["retry"].as_str() == Some(id)
                && matches!(
                    state["effect"]["status"].as_str(),
                    Some("pending" | "uncertain")
                )
            {
                let css = state["effect"]["css"]
                    .as_str()
                    .ok_or("Missing bound CSS output")?
                    .to_owned();
                return copy(ctx, state, &css).await;
            }
            Err("Unknown, stale, or cross-session CSS action".into())
        }
        _ => Err("Unsupported CSS Calculations operation".into()),
    }
}
