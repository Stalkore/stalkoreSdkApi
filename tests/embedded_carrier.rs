#![cfg(all(windows, target_arch = "x86"))]

use stalkore_sdk_api::SdkApi;
use std::{
    os::windows::ffi::OsStrExt,
    path::Path,
    time::{Duration, Instant},
};
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, LoadLibraryW};

fn wide(path: &Path) -> Vec<u16> {
    path.as_os_str().encode_wide().chain([0]).collect()
}

#[test]
fn discovers_an_embedded_provider_without_sdk_dll() {
    let Some(source) = std::env::var_os("STALKORE_EMBEDDED_FIXTURE_DLL") else {
        return;
    };
    let directory = tempfile::tempdir().unwrap();
    let carrier = directory.path().join("D3DX9_43.dll");
    std::fs::copy(source, &carrier).unwrap();
    let plugin = std::env::var_os("STALKORE_EMBEDDED_HEADLESS_DLL");
    if let Some(plugin) = plugin {
        std::fs::create_dir(directory.path().join("Plugins")).unwrap();
        std::fs::copy(plugin, directory.path().join("Plugins/headless.dll")).unwrap();
        // The integration test shares a desktop with running clients. Give its
        // shortcut a separate binding so the fixture cannot steal theirs.
        std::fs::write(
            directory.path().join("Plugins/headless.ini"),
            "[headless]\nhotkey = Ctrl+Alt+Q\n",
        )
        .unwrap();
        std::fs::write(
            directory.path().join("plugins.json"),
            r#"{"plugins":["Plugins/headless.dll"]}"#,
        )
        .unwrap();
    } else {
        std::fs::write(directory.path().join("plugins.json"), r#"{"plugins":[]}"#).unwrap();
    }
    let module = unsafe { LoadLibraryW(wide(&carrier).as_ptr()) };
    assert!(!module.is_null(), "embedded DLL did not load");
    assert!(unsafe { GetModuleHandleW(wide(Path::new("sdk.dll")).as_ptr()) }.is_null());

    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Ok(client) = SdkApi::connect().and_then(|api| api.client()) {
            if client.probe().is_ok() {
                break;
            }
        }
        assert!(
            Instant::now() < deadline,
            "embedded SDK startup did not publish its service tables"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    loop {
        let log = std::fs::read_to_string(directory.path().join("sdk.log")).unwrap_or_default();
        if log.contains("All startup callbacks accepted.") {
            break;
        }
        let plugin_log = std::fs::read_to_string(directory.path().join("Plugins/headless.log"))
            .unwrap_or_default();
        assert!(
            !log.contains("Startup failed:"),
            "embedded plugin startup failed: {log}\nplugin: {plugin_log}"
        );
        assert!(
            Instant::now() < deadline,
            "embedded plugin startup did not finish: {log}"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}
