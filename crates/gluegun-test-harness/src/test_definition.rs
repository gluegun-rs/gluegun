use std::sync::Arc;

use anyhow::Context;
use camino::Utf8PathBuf;

pub struct Test {
    test_crate: Arc<String>,
    plugins: Arc<Vec<String>>,
    actions: Vec<TestAction>,
}

#[derive(Debug)]
pub enum TestAction {
    /// Invoke cargo with the given `$OPTIONS`
    Cargo { options: Vec<String> },

    /// Invoke cargo-gluegun with the given `$OPTIONS`
    CargoGluegun { options: Vec<String> },

    /// Find the given text and replace it
    Replace {
        path: Utf8PathBuf,
        find: String,
        replace: String,
    },
}

impl Test {
    pub fn new(test_crate: String, plugins: Vec<String>) -> Self {
        Self {
            test_crate: Arc::new(test_crate),
            plugins: Arc::new(plugins),
            actions: vec![],
        }
    }

    /// Create a builder to execute cargo (options to be added to builder)
    pub fn cargo_builder(self, command: impl ToString) -> CommandBuilder {
        CommandBuilder {
            test: self,
            make_action: |options| TestAction::Cargo { options },
            options: vec![command.to_string()],
        }
    }

    /// Create a builder to execute cargo-gluegun (options to be added to builder)
    pub fn cargo_glue_gun_builder(self) -> CommandBuilder {
        CommandBuilder {
            test: self,
            make_action: |options| TestAction::CargoGluegun { options },
            options: vec![],
        }
    }

    /// Invoke glue gun with the default args for the given crate + each plugin
    pub fn cargo_glue_gun(self) -> Self {
        let test_crate = self.test_crate.clone();
        let plugins = self.plugins.clone();
        self.cargo_glue_gun_builder()
            .option("--package")
            .option(&test_crate)
            .options(&plugins[..])
            .finish()
    }

    /// Create a builder to execute cargo with the given options
    pub fn cargo_build_plugin_crates(mut self) -> Self {
        let test_crate = self.test_crate.clone();
        let plugins = self.plugins.clone();
        for plugin in &plugins[..] {
            self = self
                .cargo_builder("build")
                .option("--package")
                .option(format!("{}-{}", test_crate, plugin))
                .finish()
        }
        self
    }

    pub fn replace(
        mut self,
        path: impl Into<Utf8PathBuf>,
        from: impl ToString,
        to: impl ToString,
    ) -> Self {
        self.actions.push(TestAction::Replace { path: path.into(), find: from.to_string(), replace: to.to_string() });
        self
    }

    /// Execute the test from the given directory
    pub fn execute(self, directory: Utf8PathBuf) -> anyhow::Result<()> {
        execute_test(self, directory)
    }
}

pub struct CommandBuilder {
    test: Test,
    make_action: fn(Vec<String>) -> TestAction,
    options: Vec<String>,
}

impl CommandBuilder {
    pub fn option(mut self, option: impl ToString) -> Self {
        self.options.push(option.to_string());
        self
    }

    pub fn options(mut self, option: impl IntoIterator<Item = impl ToString>) -> Self {
        self.options
            .extend(option.into_iter().map(|i| i.to_string()));
        self
    }

    pub fn finish(mut self) -> Test {
        self.test.actions.push((self.make_action)(self.options));
        self.test
    }
}

fn execute_test(test: Test, directory: Utf8PathBuf) -> anyhow::Result<()> {
    let _dir_guard = DirectoryGuard::new()?;

    std::env::set_current_dir(&directory)
        .with_context(|| format!("changing to directory `{directory}`"))?;

    for action in &test.actions {
        execute_action(action).with_context(|| format!("executing action: {action:?}"))?;
    }

    Ok(())
}

struct DirectoryGuard {
    start_dir: Utf8PathBuf,
}

impl DirectoryGuard {
    fn new() -> anyhow::Result<Self> {
        let start_dir = Utf8PathBuf::try_from(std::env::current_dir()?)?;
        Ok(Self { start_dir })
    }
}

impl Drop for DirectoryGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(self.start_dir.clone());
    }
}

fn execute_action(action: &TestAction) -> anyhow::Result<()> {
    match action {
        TestAction::Cargo { options } => cargo_action(options),
        TestAction::Replace {
            path,
            find,
            replace,
        } => replace_action(path, find, replace),
        TestAction::CargoGluegun { options } => cargo_gluegun::cli_main_from(options),
    }
}

fn cargo_action(options: &[String]) -> anyhow::Result<()> {
    let mut command = std::process::Command::new("cargo");
    command.args(options);
    let status = command.status()?;
    if !status.success() {
        anyhow::bail!("cargo command failed");
    }
    Ok(())
}

fn replace_action(path: &Utf8PathBuf, find: &str, replace: &str) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(path)?;

    if !content.contains(find) {
        anyhow::bail!("find string not found in file");
    }

    let content = content.replace(find, replace);
    std::fs::write(path, content)?;
    Ok(())
}
