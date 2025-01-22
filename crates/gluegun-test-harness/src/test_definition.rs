use std::{process::Command, sync::Arc};

use anyhow::Context;
use camino::Utf8PathBuf;
use cp_r::CopyOptions;
use temp_dir::TempDir;

pub struct Test {
    test_crate: Arc<String>,
    source_directory: Utf8PathBuf,
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
    pub fn new(
        test_crate: impl ToString,
        plugins: impl IntoIterator<Item: ToString>,
        source_directory: impl Into<Utf8PathBuf>,
    ) -> Self {
        Self {
            test_crate: Arc::new(test_crate.to_string()),
            source_directory: source_directory.into(),
            plugins: Arc::new(plugins.into_iter().map(|t| t.to_string()).collect()),
            actions: vec![],
        }
    }

    /// Create a builder to execute cargo (options to be added to builder)
    pub fn cargo_builder(self, command: impl ToString) -> CommandBuilder {
        CommandBuilder {
            test: self,
            make_action: |options| TestAction::Cargo { options },
            options: vec!["--verbose".to_string(), command.to_string()],
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
    pub fn execute(self) -> anyhow::Result<()> {
        TestExecutor::new(self)?.execute()?;
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
    temp_dir: Utf8PathBuf,
    temp_dir_cleanup: Option<TempDir>,
}

impl TestExecutor {
    fn new(test: Test) -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;
        Ok(Self {
            test,
            temp_dir: Utf8PathBuf::try_from(temp_dir.path().to_path_buf()).unwrap(),
            temp_dir_cleanup: Some(temp_dir),
        })
    }

    fn leak(&mut self) {
        self.temp_dir_cleanup.take().map(|t| t.leak());
    }

    fn execute(mut self) -> anyhow::Result<()> {
        eprintln!(
            "# executing test {src} in {temp_dir}",
            src = self.test.source_directory,
            temp_dir = self.temp_dir,
        );

        self.leak();

        // initialize temporary directory with contents of `directory`
        CopyOptions::new().copy_tree(&self.test.source_directory, &self.temp_dir)?;

        // test test actions
        for action in &self.test.actions {
            self.execute_action(action)
                .with_context(|| format!("executing action: {action:?}"))?;
        }

        Ok(())
    }

    fn execute_action(&self, action: &TestAction) -> anyhow::Result<()> {
        eprintln!("## execute action {action:?}");
        match action {
            TestAction::Cargo { options } => self.cargo_action(options),

            TestAction::Replace {
                path,
                find,
                replace,
            } => self.replace_action(path, find, replace),

            TestAction::CargoGluegun { options } => cargo_gluegun::Builder::new(
                &self.temp_dir,
                Some("cargo-gluegun")
                    .into_iter()
                    .chain(options.iter().map(|o| &o[..])),
            )?
            .plugin_command(|_workspace_metadata, _package_metadata, plugin| {
                let manifest_path = std::env::var("CARGO_MANIFEST_PATH")
                    .with_context(|| format!("fetching `CARGO_MANIFEST_PATH` variable"))?;
                let mut c = Command::new("cargo");
                c
                    .arg("run")
                    .arg("--manifest-path")
                    .arg(manifest_path)
                    .arg("-p")
                    .arg(format!("gluegun-{plugin}"))
                    .arg("--");
                Ok(c)
            })
            .execute(),
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
        let file_path = self.temp_dir.join(path);

        let content = std::fs::read_to_string(&file_path)?;

        if !content.contains(find) {
            anyhow::bail!("`{file_path}` does not contain `{find}`");
        }

        let content = content.replace(find, replace);
        std::fs::write(path, content)?;
        Ok(())
    }
}
