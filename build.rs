use std::path::Path;

fn main() {
    println!("cargo:rerun-if-env-changed=WOOTING_RGB_SDK_PATH");
    println!("cargo:rerun-if-changed=external/wooting-rgb-sdk/src/wooting-rgb-sdk.h");

    if Path::new("external/wooting-rgb-sdk").exists() {
        println!("cargo:warning=Using Wooting RGB SDK ABI from external/wooting-rgb-sdk");
    }
}
