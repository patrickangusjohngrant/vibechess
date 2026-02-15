fn main() {
    let output = std::process::Command::new("date")
        .arg("+%Y-%m-%d %H:%M")
        .output()
        .expect("failed to run date");
    let timestamp = String::from_utf8_lossy(&output.stdout).trim().to_string();
    println!("cargo:rustc-env=BUILD_TIMESTAMP={timestamp}");
}
