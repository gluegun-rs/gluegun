use camino::Utf8PathBuf;

const PLUGINS: &[&str] = &["java", "py"];

fn project_root_directory() -> Utf8PathBuf {
    Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn demo_directory(name: &str) -> Utf8PathBuf {
    project_root_directory().join("demos").join(name)
}

#[test]
fn idl_tests() -> anyhow::Result<()> {
    gluegun_test_harness::idl_tests()
}

#[test]
fn hello_world() -> anyhow::Result<()> {
    gluegun_test_harness::Test::new("hello_world", PLUGINS, demo_directory("hello_world"))
    .cargo_glue_gun()
    .cargo_build_plugin_crates()
    .execute()
}