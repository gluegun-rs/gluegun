use std::process::Command;

use accessors_rs::Accessors;

/// Options for configuring and registering helper utilities that ought to be available.
/// These are extra commands, like `cargo-component` for WASM, that need to be installed
/// for a given bit of crate creation code to work.
#[derive(Accessors)]
pub struct HelperCommand {
    #[accessors(get)]
    name: String,

    install_option: InstallOption,
}

impl HelperCommand {
    pub(crate) fn new(name: String) -> Self {
        Self {
            name,
            install_option: InstallOption::Fail,
        }
    }

    fn assert_install_option_not_configured(&self) {
        if let InstallOption::Fail = self.install_option {
            return;
        }

        panic!(
            "install option was already configured to be {:?}",
            self.install_option
        )
    }

    /// Install the helper command if necessary.
    pub(crate) fn install_if_needed(&self) -> anyhow::Result<()> {
        if which::which(&self.name).is_ok() {
            // Command is already present on the PATH
            return Ok(());
        }

        match self.install_option {
            InstallOption::Fail => {
                anyhow::bail!(
                    "helper command `{}` is not installed and no install option was configured",
                    self.name
                );
            }
            InstallOption::FailWithMessage(ref message) => {
                anyhow::bail!(
                    "helper command `{}` is not installed: {}",
                    self.name,
                    message
                );
            }
            InstallOption::CargoInstall { ref crate_name } => {
                let status = Command::new("cargo")
                    .arg("install")
                    .arg(crate_name)
                    .status()
                    .map_err(|e| {
                        anyhow::anyhow!("failed to install helper command `{}`: {}", self.name, e)
                    })?;

                if !status.success() {
                    anyhow::bail!("`cargo install {crate_name}` failed with code {status:?}");
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
enum InstallOption {
    Fail,

    /// Report an error if not installed: the default
    FailWithMessage(String),

    /// Execute `cargo install {crate_name}`
    CargoInstall {
        crate_name: String,
    },
}

pub struct HelperCommandGuard<'g> {
    utility: &'g mut HelperCommand,
}

impl<'g> HelperCommandGuard<'g> {
    pub(crate) fn new(utility: &'g mut HelperCommand) -> Self {
        Self { utility }
    }

    /// Configure the helper utility to fail with the given message.
    /// The default behavior is to fail with a generic message.
    ///
    /// # Panics
    ///
    /// If the behavior when not installed has already been configured.
    pub fn or_fail(self, message: String) -> Self {
        self.utility.assert_install_option_not_configured();
        self.utility.install_option = InstallOption::FailWithMessage(message);
        self
    }

    /// Configure the helper utility to fail with the given message.
    /// The default behavior is to fail with a generic message.
    ///
    /// # Panics
    ///
    /// If the behavior when not installed has already been configured.
    pub fn or_run_cargo_install(self, crate_name: &str) -> Self {
        self.utility.assert_install_option_not_configured();
        self.utility.install_option = InstallOption::CargoInstall { crate_name: crate_name.to_string() };
        self
    }
}
