fn main() {
    println!("cargo:rerun-if-env-changed=WOOTING_RGB_SDK_PATH");
    println!("cargo:rerun-if-changed=external/wooting-rgb-sdk/src/wooting-rgb-sdk.h");
}
