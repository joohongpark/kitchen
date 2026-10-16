// sqlx::migrate! 는 컴파일 시점에 마이그레이션을 내장하므로, 파일이 추가·변경되면 다시 빌드한다.
fn main() {
    println!("cargo:rerun-if-changed=../../migrations");
}
