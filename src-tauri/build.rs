use std::env;
use std::path::PathBuf;

fn main() {
    #[cfg(target_os = "macos")]
    build_steinberg_host_bridge();

    tauri_build::build()
}

#[cfg(target_os = "macos")]
fn build_steinberg_host_bridge() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("missing manifest dir"));
    let sdk_root = manifest_dir.join("third_party/vst3sdk");

    println!("cargo:rerun-if-changed={}", sdk_root.display());
    println!(
        "cargo:rerun-if-changed={}",
        manifest_dir
            .join("src/playback/plugin_host/macos_host/steinberg_sdk_host_bridge.cpp")
            .display()
    );

    cc::Build::new()
        .cpp(true)
        .std("c++17")
        .flag_if_supported("-mmacosx-version-min=11.0")
        .flag_if_supported("-Wno-deprecated-declarations")
        .flag_if_supported("-Wno-unnecessary-virtual-specifier")
        .include(&sdk_root)
        .file(sdk_root.join("pluginterfaces/base/funknown.cpp"))
        .file(sdk_root.join("pluginterfaces/base/coreiids.cpp"))
        .file(
            manifest_dir
                .join("src/playback/plugin_host/macos_host/steinberg_sdk_host_bridge.cpp"),
        )
        .file(sdk_root.join("public.sdk/source/vst/vstinitiids.cpp"))
        .file(sdk_root.join("public.sdk/source/vst/hosting/hostclasses.cpp"))
        .file(sdk_root.join("public.sdk/source/vst/hosting/pluginterfacesupport.cpp"))
        .file(sdk_root.join("public.sdk/source/vst/utility/stringconvert.cpp"))
        .file(sdk_root.join("public.sdk/source/common/commonstringconvert.cpp"))
        .compile("compose_steinberg_host_bridge");
}
