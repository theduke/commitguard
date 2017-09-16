/*!
# commitguard

**commitguard** is a [Rust](https://rust-lang.org) library that makes it very easy to create
shell scripts that can be used as **git pre-commit** hooks.

The primary use case is generating pre-commit hooks for Rust projects automatically in a **build.rs** build script,
but it may be useful for other projects too.

## Why

## Info

## License

This project is under the MIT license.
See the LICENSE file for more details.

## Versioning

This project follows semantic versioning.

## README

The README.md file is auto-generated from src/lib.rs in the build.rs script.

All changes should be done there.

*/

/// ErrorMode specifies the action to take when a check fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErrorMode {
    /// Only warn about the failure, but continue other checks and allow the commit.
    Warn,
    /// Abort immediately, skipping all remaining checks.
    AbortImmediate,
    /// Abort, but continue running other checks.
    Abort,
}

/// A single check to be performed.
pub struct Check {
    /// Name of the command to run.
    command: String,
    /// Name of the command to check for.
    /// This can be different from the actual command + args.
    check_for_command: Option<String>,
    /// If true, the command needs to be present to pass the check.
    /// If false, the command is ignored.
    command_required: bool,
    /// Instructions for installing the required command for running the check.
    install_instructions: Option<String>,
    /// Arguments to pass to the command.
    args: Vec<String>,
    /// The error mode to use.
    error_mode: ErrorMode,
    /// Optional name for the check, used in output.
    name: Option<String>,
    /// Ignore the output if the check succeeds.
    silent: bool,
}

impl Check {
    /// Construct a new check.
    ///
    /// The returned value can be used as a builder by specifying additional options with the
    /// respective functions.
    pub fn new<S: Into<String>>(command: S) -> Self {
        Check {
            command: command.into(),
            check_for_command: None,
            command_required: false,
            install_instructions: None,
            args: vec![],
            error_mode: ErrorMode::AbortImmediate,
            name: None,
            silent: true,
        }
    }

    pub fn check_for_command<S: Into<String>>(mut self, command: S) -> Self {
        self.check_for_command = Some(command.into());
        self
    }

    pub fn args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Require this command to be present.
    ///
    /// If true, the check fails if the command can not be found.
    /// If false, the check is ignored if the command can not be found.
    pub fn require_command(mut self) -> Self {
        self.command_required = true;
        self
    }

    /// Warn if the check fails instead of aborting the hook.
    ///
    /// Also see abort() and abort_immediate().
    pub fn warn(mut self) -> Self {
        self.error_mode = ErrorMode::Warn;
        self
    }

    /// Abort the hook if this check fails, but run also run other checks before aborting.
    ///
    /// If you want to abort immediately without running other checks, use abort_immedate().
    /// To only warn and continue the hook, use warn().
    pub fn abort(mut self) -> Self {
        self.error_mode = ErrorMode::Abort;
        self
    }

    /// Abort the hook immediately if the check fails, without running other checks before aborting.
    ///
    /// Also see warn() and abort_immediate().
    pub fn abort_immediate(mut self) -> Self {
        self.error_mode = ErrorMode::AbortImmediate;
        self
    }

    /// Provide install instructions if the command is not present on the system.
    pub fn install_instructions<S: Into<String>>(mut self, instructions: S) -> Self {
        self.install_instructions = Some(instructions.into());
        self
    }

    /// Specify a custom name for the command.
    ///
    /// The name will be used in the output.
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Specify if the check should be silent.
    ///
    /// If true, the check output will be ignored if the check succeeds, and only printed once
    /// it failed.
    ///
    /// Defaults to false.
    pub fn silent(mut self, silent: bool) -> Self {
        self.silent = silent;
        self
    }
}

/// Builder for a new pre-commit hook.
///
/// You can add multiple checks to the guard.
pub struct CommitGuard {
    /// The checks to run.
    checks: Vec<Check>,
}

impl CommitGuard {
    pub fn new() -> Self {
        return CommitGuard { checks: vec![] };
    }

    /// Add a new check to the guard.
    ///
    /// Construct checks with Check::new() or use some of the built-in ones.
    pub fn check(mut self, check: Check) -> Self {
        self.checks.push(check);
        self
    }

    /// Build the commit guard as an SH shell script.
    pub fn build_sh(&self) -> String {
        let mut script = String::new();

        script.push_str("#! /bin/sh\n\n");
        script.push_str(
            "# This script was auto-generated with commitguard \
             (https://github.com/theduke/commitguard\n\n",
        );

        script.push_str("echo 'Running commit guards...\n'\n\n");
        script.push_str("FAILED_CHECKS=''\n\n");

        for check in &self.checks {
            let name = check.name.as_ref().unwrap_or(&check.command);

            let check_command = check
                .check_for_command
                .clone()
                .unwrap_or(check.command.clone());

            let install_instructions = check
                .install_instructions
                .as_ref()
                .map(|content| {
                    format!(
                        "    echo 'Install it like this: {}'\n",
                        content.replace('\n', "\\n").replace('\'', "\\'")
                    )
                })
                .unwrap_or("".to_string());

            let handle_missing = if check.command_required {
                // Command is required, so abort.
                format!(
                    "echo 'ERROR: missing required command {}'\n{}    exit 1",
                    check_command,
                    install_instructions
                )
            } else {
                // Missing command is allowed, so print warning and continue.
                format!(
                    "echo 'WARNING: missing command {}'\n{}",
                    check_command,
                    install_instructions
                )
            };

            let cmd = format!("{} {}", check.command, check.args.join(" "));

            let mut execute = if check.silent {
                format!("OUTPUT=$({} 2>&1)\n", cmd)
            } else {
                format!("{}\n", cmd)
            };

            let print_silent = if check.silent {
                "echo \"\n$OUTPUT\n\n\"\n".to_string()
            } else {
                "".to_string()
            };

            execute.push_str("    OK=$?\n");

            let handle_err = match check.error_mode {
                ErrorMode::Warn => format!(
                    "{}    echo 'WARNING: check {} failed!'\n",
                    print_silent,
                    name
                ),
                ErrorMode::AbortImmediate => format!(
                    "{}    echo 'ERROR: check {} failed!'\n    exit 1\n",
                    print_silent,
                    name
                ),
                ErrorMode::Abort => format!(
                    "{}    echo 'ERROR: check {cmd} failed!'\n    FAILED_CHECKS='$FAILED_CHECKS {cmd}'\n",
                    print_silent,
                    cmd = name
                ),
            };


            script.push_str(&format!("# Check if {} command exists\n", check_command));
            script.push_str(&format!(
                "\
echo 'Running check \"{name}\"\n'
command -v {cmd} >/dev/null 2>&1
if [[ $? -gt 0 ]]; then
    # Command not found.
    {handle_missing}
else
    # Command found, so execute it.
    {execute}
    if [ $OK -gt 0 ]; then
        {handle_err}
    else
        echo 'Check \"{name}\" OK'
    fi
fi
echo ''

",
                name = name,
                cmd = check_command,
                handle_missing = handle_missing,
                execute = execute,
                handle_err = handle_err
            ));
        }

        script.push_str(
            "\
if [ ! -z \"$FAILED_CHECKS\" ]; then
    echo \"Failed checks: $FAILED_CHECKS\"
    exit 1
fi

exit 0
",
        );

        script
    }
}
