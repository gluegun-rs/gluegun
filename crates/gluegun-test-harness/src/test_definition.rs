use std::sync::Arc;

use anyhow::Context;
use camino::Utf8PathBuf;
use cp_r::CopyOptions;
use temp_dir::TempDir;

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

    /// Add a step to invoke `cargo build` on the crates generated from the plugin
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
        self.actions.push(TestAction::Replace {
            path: path.into(),
            find: from.to_string(),
            replace: to.to_string(),
        });
        self
    }

    /// Execute the test from the given directory
    pub fn execute(self, directory: Utf8PathBuf) -> anyhow::Result<()> {
        TestExecutor::new(self, directory)?.execute()?;
        Ok(())
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

struct TestExecutor {
    test: Test,
    temp_dir: TempDir,
    source_directory: Utf8PathBuf,
}

impl TestExecutor {
    fn new(test: Test, source_directory: Utf8PathBuf) -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;
        Ok(Self {
            test,
            temp_dir,
            source_directory,
        })
    }

    fn execute(&self) -> anyhow::Result<()> {
        eprintln!(
            "# executing test {src} in {temp_dir}",
            src = self.source_directory,
            temp_dir = self.temp_dir.path().display(),
        );

        // initialize temporary directory with contents of `directory`
        CopyOptions::new().copy_tree(&self.source_directory, &self.temp_dir)?;

        // test test actions
        for action in &self.test.actions {
            self.execute_action(action).with_context(|| format!("executing action: {action:?}"))?;
        }

        Ok(())
    }

    fn execute_action(&self, action: &TestAction) -> anyhow::Result<()> {
        match action {
            TestAction::Cargo { options } => self.cargo_action(options),
        
            TestAction::Replace {
                path,
                find,
                replace,
            } => self.replace_action(path, find, replace),

            TestAction::CargoGluegun { options } => {
                cargo_gluegun::cli_main_from(
                    &self.temp_dir,
                    options,
                )
            }
        }
    }

    fn cargo_action(&self, options: &[String]) -> anyhow::Result<()> {
        let mut command = std::process::Command::new("cargo");
        command.current_dir(&self.temp_dir);
        command.args(options);
        let status = command.status()?;
        if !status.success() {
            anyhow::bail!("cargo command failed");
        }
        Ok(())
    }

    fn replace_action(&self, path: &Utf8PathBuf, find: &str, replace: &str) -> anyhow::Result<()> {
        let file_path = Utf8PathBuf::try_from(self.temp_dir.path().join(path))?;

        let content = std::fs::read_to_string(&file_path)?;

        if !content.contains(find) {
            anyhow::bail!("`{file_path}` does not contain `{find}`");
        }

        let content = content.replace(find, replace);
        std::fs::write(path, content)?;
        Ok(())
    }
}
