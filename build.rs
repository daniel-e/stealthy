use std::process::Command;
use std::env;
use std::path::Path;
use std::io::Write;
use std::fs::File;


fn try_gcc(lib: &str, msg: &str) {

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("main.c");
    let mut f = File::create(&dest_path).unwrap();

    f.write_all(b"
        int main() {
        }
    ").unwrap();

    let s = Command::new("gcc")
        .args(&[dest_path.into_os_string().to_str().unwrap(), lib, "-o"])
        .arg(&format!("{}/main.o", out_dir))
        .status()
        .unwrap();

    assert!(s.success(), "\n\n".to_string() + msg);
}


fn main() {

    try_gcc("-lpcap", "pcap not found. On Ubuntu try 'sudo apt-get install libpcap-dev' before continuing.");
    try_gcc("-lncursesw", "ncursesw not found. On Ubuntu try 'sudo apt-get install libncursesw5-dev' before continuing.");
    try_gcc("-lcrypto", "crypto not found. On Ubuntu try 'sudo apt-get install libssl-dev' before continuing.");

    let out_dir = env::var("OUT_DIR").unwrap();

    Command::new("gcc").args(&["icmp/net.c", "-c", "-fPIC", "-o"])
                       .arg(&format!("{}/net.o", out_dir)).status().unwrap();
    Command::new("ar").args(&["crus", "libicmp.a", "net.o"])
                      .current_dir(&Path::new(&out_dir)).status().unwrap();

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=icmp");
}


