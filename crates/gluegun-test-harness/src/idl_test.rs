use anyhow::Context;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};
use crate::BLESS;

struct IdlTest {
    rs_path: PathBuf,
    idl_path: PathBuf,
}

pub fn idl_tests() -> anyhow::Result<()> {
    let idl_tests = assemble_idl_tests()?;
    run_idl_tests(idl_tests)?;
    Ok(())
}

fn assemble_idl_tests() -> anyhow::Result<Vec<IdlTest>> {
    let mut tests = vec![];
    for entry in std::fs::read_dir("idl-tests").with_context(|| "failed to read `idl-tests` directory")? {
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
    for test in tests {
        match run_idl_test(&test) {
            Ok(()) => {}
            Err(err) => {
                let err_path = test.idl_path.with_extension("err");
                std::fs::write(&err_path, format!("{:?}", err))
                    .with_context(|| format!("failed to write `{}`", err_path.display()))?;

                eprintln!(
                    "Test failure: test `{rs}` failed, see `{err}`",
                    rs = test.rs_path.display(),
                    err = err_path.display()
                );

                test_failures.push(test);
            }
        }
    }

    if test_failures.is_empty() {
        return Ok(());
    }

    anyhow::bail!("{test_failures} out of {tests_len} tests failed", test_failures = test_failures.len())
}

fn run_idl_test(test: &IdlTest) -> anyhow::Result<()> {
    let crate_name = test.rs_path.file_stem().ok_or_else(|| {
        anyhow::anyhow!(
            "no file name for `.rs` file in `{}`",
            test.rs_path.display()
        )
    })?;
    let crate_name = crate_name.to_str().ok_or_else(|| {
        anyhow::anyhow!(
            "non-utf8 file name for `.rs` file in `{}`",
            test.rs_path.display()
        )
    })?;
    let parsed_idl = gluegun_idl::Parser::new()
        .parse_crate_named(crate_name, &test.rs_path)
        .with_context(|| format!("failed to load `{}`", test.rs_path.display()))?;
    let idl_json = serde_json::to_string_pretty(&parsed_idl)
        .with_context(|| format!("failed to serialize json from `{}`", test.rs_path.display()))?;
    let reference_json = std::fs::read_to_string(&test.idl_path).unwrap_or_default();

    if idl_json != reference_json {
        if *BLESS {
            eprintln!("test `{}` blessed because BLESS=1", test.rs_path.display());
        } else {
            let diff = similar::udiff::unified_diff(
                similar::Algorithm::Myers,
                &reference_json,
                &idl_json,
                2,
                Some((&test.idl_path.display().to_string(), "new")),
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
