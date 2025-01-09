use anyhow::Context;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

lazy_static::lazy_static! {
    static ref BLESS: bool = std::env::var("BLESS").is_ok();
}

fn main() -> anyhow::Result<()> {
    let idl_tests = assemble_idl_tests()?;
    run_idl_tests(idl_tests)?;
    Ok(())
}

struct IdlTest {
    rs_path: PathBuf,
    idl_path: PathBuf,
}

fn assemble_idl_tests() -> anyhow::Result<Vec<IdlTest>> {
    let mut tests = vec![];
    for entry in std::fs::read_dir("test/idl").with_context(|| "failed to read `idl` directory")? {
        let entry = entry.with_context(|| "reading directory entry from `idl`")?;
        let path = entry.path();
        if path.is_dir() {
        } else if is_eq(&path, Path::extension, "rs") {
            let idl_path = path.with_extension("idl");
            tests.push(IdlTest {
                rs_path: path,
                idl_path,
            })
        }
    }
    Ok(tests)
}

fn run_idl_tests(tests: Vec<IdlTest>) -> anyhow::Result<()> {
    let mut test_failures = vec![];
    let tests_len = tests.len();
    progress_bar::init_progress_bar(tests.len());
    for test in tests {
        match run_idl_test(&test) {
            Ok(()) => {}
            Err(err) => {
                let err_path = test.idl_path.with_extension("err");
                std::fs::write(&err_path, format!("{:?}", err)).with_context(|| format!("failed to write `{}`", err_path.display()))?;
    
                progress_bar::print_progress_bar_info(
                    "Test failure",
                    &format!("test `{}` failed, see `{}`", test.rs_path.display(), err_path.display()),
                    progress_bar::Color::Red,
                    progress_bar::Style::Bold,
                );

                test_failures.push(test);
            }
        }
        progress_bar::inc_progress_bar();
    }

    if test_failures.is_empty() {
        progress_bar::print_progress_bar_info(
            "Test summary",
            &format!("all {tests_len} tests passed"),
            progress_bar::Color::Green,
            progress_bar::Style::Bold,
        );
    } else {
        progress_bar::print_progress_bar_info(
            "Test summary",
            &format!("{} out of {tests_len} tests failed", test_failures.len()),
            progress_bar::Color::Red,
            progress_bar::Style::Bold,
        );
    }
    progress_bar::finalize_progress_bar();
    Ok(())
}

fn run_idl_test(test: &IdlTest) -> anyhow::Result<()> {
    let crate_name = test.rs_path.file_stem().ok_or_else(|| anyhow::anyhow!("no file name for `.rs` file in `{}`", test.rs_path.display()))?;
    let crate_name = crate_name.to_str().ok_or_else(|| anyhow::anyhow!("non-utf8 file name for `.rs` file in `{}`", test.rs_path.display()))?;
    let parsed_idl = gluegun_idl::Parser::new().parse_crate_named(crate_name, &test.rs_path).with_context(|| format!("failed to load `{}`", test.rs_path.display()))?;
    let idl_json = serde_json::to_string_pretty(&parsed_idl).with_context(|| format!("failed to serialize json from `{}`", test.rs_path.display()))?;
    let reference_json = std::fs::read_to_string(&test.idl_path).unwrap_or_default();

    if idl_json != reference_json {
        if *BLESS {
            progress_bar::print_progress_bar_info(
                "Test blessed",
                &format!("test `{}` blessed because BLESS=1", test.rs_path.display()),
                progress_bar::Color::Yellow,
                progress_bar::Style::Normal,
            );
            std::fs::write(&test.idl_path, idl_json)?;
        } else {
            let diff = similar::udiff::unified_diff(
                similar::Algorithm::Myers,
                &reference_json,
                &idl_json,
                2,
                Some((&test.idl_path.display().to_string(), "new"))
            );

            return Err(anyhow::anyhow!(
                "test `{}` failed\n\n{diff}",
                test.rs_path.display(),
            ));
        }
    }

    Ok(())
}

fn is_eq(p: &Path, op: impl Fn(&Path) -> Option<&OsStr>, arg: &str) -> bool {
    match op(p) {
        Some(s) => s == arg,
        None => false,
    }
}
