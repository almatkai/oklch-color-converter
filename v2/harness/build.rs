use std::{env, fs, path::PathBuf};
fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .to_path_buf();
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let base = root.join("fixtures/host/src-tauri/src/native_core");
    let common = base.join("common.rs");
    let sdk = base.join("extensions/v2_sdk.rs");
    let native = base.join("extensions/native.rs");
    let adapter = root.join("adapter/oklch.rs");
    // Production runtime fixture is modified only by the reviewed registration patch.
    let runtime = base.join("extensions/v2_runtime.rs");
    let original = fs::read_to_string(&runtime).unwrap();
    let registered=original.replace("\"perplexity\" => Some(value)","\"perplexity\" | \"oklch-color-converter\" => Some(value)").replace("_ => &[],","\"oklch-color-converter\" => super::v2_oklch::commands(),\n        _ => &[],").replace("_ => Err(format!(\"No native adapter for {slug}\")),","\"oklch-color-converter\" => super::v2_oklch::handle(ctx, command, operation, payload, state).await,\n        _ => Err(format!(\"No native adapter for {slug}\")),");
    assert!(registered.contains("super::v2_oklch::handle"));
    fs::write(out.join("runtime.rs"), registered).unwrap();
    let stubs=["v2_amphetamine","v2_audio_device","v2_cleanshotx","v2_color_picker","v2_gif_search","v2_google_calendar","v2_google_search","v2_perplexity","v2_port_manager","v2_speedtest","v2_timers"].map(|m|format!("mod {m}{{pub fn commands()->&'static[&'static str]{{&[]}}pub async fn handle(_: &crate::CoreContext,_:&str,_:&str,_:&serde_json::Value,_:&mut serde_json::Value)->crate::CoreResult{{Err(\"unrelated adapter stub\".into())}}}}\n")).join("");
    let generated = format!(
        r#"
    pub mod native_core {{
        #[path={common:?}] pub mod common;
        pub mod extensions {{
            #[allow(dead_code)] // Unused portions of unchanged SDK fixture.
            #[path={sdk:?}] mod v2_sdk;
            #[path={native:?}] mod native;
            #[path={runtime:?}] mod v2_runtime;
            #[path={adapter:?}] mod v2_oklch;
            {stubs}
            mod registry {{
                use crate::{{CoreContext,CoreResult}};use serde_json::{{Value,json}};
                pub fn installed_command(ctx:&CoreContext,id:&str,command:&str)->Option<Value>{{
                    // Installation policy stub: confined to a disposable test dir.
                    if ctx.paths.data_dir.join("installed").is_file() && super::v2_runtime::supports(id,command){{Some(json!({{}}))}}else{{None}}
                }}
                pub fn preferences(_:&Value,_:Option<&str>)->CoreResult{{Ok(json!({{}}))}}
            }}
            #[derive(Clone,Default)] pub struct Sessions(native::NativeSessions);
            impl Sessions {{
                pub async fn handle(&self,ctx:&crate::CoreContext,op:&str,payload:serde_json::Value)->serde_json::Value{{
                    self.0.handle(ctx,&std::sync::Arc::new(std::sync::Mutex::new(())),op,&[payload]).await.unwrap().unwrap()
                }}
                pub async fn clear(&self){{self.0.clear().await;}}
                pub async fn clear_slug(&self){{self.0.clear_slug("oklch-color-converter").await;}}
            }}
            pub fn dispatch_supports(id:&str,command:&str)->bool{{let _=v2_runtime::canonical_id("oklch-color-converter");v2_runtime::supports(id,command)}}
            pub async fn direct(ctx:&crate::CoreContext,op:&str,payload:&serde_json::Value,state:&mut serde_json::Value)->crate::CoreResult{{v2_oklch::handle(ctx,"convert-color",op,payload,state).await}}
        }}
    }}
    "#,
        common = common,
        sdk = sdk,
        native = native,
        runtime = out.join("runtime.rs"),
        adapter = adapter,
        stubs = stubs
    );
    fs::write(out.join("bindings.rs"), generated).unwrap();
    for p in [common, sdk, native, runtime, adapter] {
        println!("cargo:rerun-if-changed={}", p.display());
    }
}
