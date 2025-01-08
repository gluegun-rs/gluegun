use lingo_star_idl;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

fn main() -> anyhow::Result<()> {
    let idl_tests = assemble_idl_tests()?;
    run_idl_tests(&idl_tests)?;
    Ok(())
}

struct IdlTest {
    build_type: IdlTestType,
    rs_path: PathBuf,
    idl_path: PathBuf,
}

enum IdlTestType {
    SingleFile,
}

fn assemble_idl_tests() -> anyhow::Result<Vec<IdlTest>> {
    let mut tests = vec![];
    for entry in std::fs::read_dir("idl")? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
        } else if is_eq(&path, Path::extension, "rs") {
            let idl_path = path.with_extension("idl");
            tests.push(IdlTest {
                build_type: IdlTestType::SingleFile,
                rs_path: path,
                idl_path,
            })
        }
    }
    Ok(tests)
}

fn run_idl_tests(tests: &[IdlTest]) -> anyhow::Result<()> {
    progress_bar::init_progress_bar(tests.len());
    for test in tests {
        match run_idl_test(test) {
            Ok(()) => {}
            Err(err) => {
                progress_bar::print_progress_bar_info(
                    "Test failure",
                    &format!("test `{}` failed", test.rs_path.display()),
                    progress_bar::Color::Red,
                    progress_bar::Style::Bold,
                );
                progress_bar::finalize_progress_bar();
                return Err(err);
            }
        }
        progress_bar::inc_progress_bar();
    }
    progress_bar::finalize_progress_bar();
    Ok(())
}

fn run_idl_test(test: &IdlTest) -> anyhow::Result<()> {
    let parsed_idl = lingo_star_idl::parse_path(&test.rs_path)?;
    let generated_rs = lingo_star_idl::generate_rs(&parsed_idl);
    if generated_rs != rs {
        panic!(
            "Generated code does not match for {}",
            test.rs_path.display()
        );
    }
    Ok(())
}

fn is_eq(p: &Path, op: impl Fn(&Path) -> Option<&OsStr>, arg: &str) -> bool {
    match op(p) {
        Some(s) => s == arg,
        None => false,
    }
}
