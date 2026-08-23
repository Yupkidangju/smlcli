fn main() {
    println!("cargo:rerun-if-changed=.git/HEAD");
    let commit = std::process::Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    let build_epoch = std::env::var("SOURCE_DATE_EPOCH").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=SMLCLI_SHORT_COMMIT={commit}");
    println!("cargo:rustc-env=SMLCLI_BUILD_EPOCH={build_epoch}");
}
