// rust-embed 가 컴파일 시점에 폴더를 요구하므로, 프론트 빌드 전에도 컴파일되도록 빈 폴더를 만든다.
fn main() {
    let dist = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../web/dist");
    if !dist.exists() {
        std::fs::create_dir_all(&dist).expect("create web/dist");
    }
    println!("cargo:rerun-if-changed=../../web/dist");
}
