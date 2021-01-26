use std::io::{self, Read};
use std::process;

/// A thin wrapper around process::Command to facilitate running external commands.
pub struct Command {
    program: Option<String>,
    args: Vec<String>,
    dir: Option<String>,
}

/// The result of a command execution if the child process managed to execute
pub struct CommandRun {
    pub status: process::ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

impl Command {
    /// Check whether the given program can be executed
    pub fn exists_program(program: &str) -> bool {
        Command::new().program(program).spawn().is_ok()
    }

    /// Construct a new Command
    pub fn new() -> Command {
        Command {
            program: None,
            args: vec![],
            dir: None,
        }
    }

    /// Set the program to run
    pub fn program(&mut self, program: &str) -> &mut Self {
        self.program = Some(program.to_owned());
        self
    }

    /// Add a new program argument
    pub fn arg(&mut self, arg: &str) -> &mut Self {
        self.args.push(arg.to_owned());
        self
    }

    /// Add a new program argument, concatenated from several parts
    pub fn arg_from_parts(&mut self, parts: Vec<&str>) -> &mut Self {
        let arg = parts.join("");
        self.args.push(arg);
        self
    }

    /// Set the working directory for the child process
    pub fn current_dir(&mut self, dir: &str) -> &mut Self {
        self.dir = Some(dir.to_owned());
        self
    }

    /// Execute the command as a child process, and extract its status, stdout, stderr.
    pub fn spawn(&mut self) -> io::Result<CommandRun> {
        match &self.program {
            None => Err(io::Error::new(io::ErrorKind::InvalidInput, "")),
            Some(program) => {
                let mut command = process::Command::new(program);
                command
                    .args(&self.args)
                    .stdout(process::Stdio::piped())
                    .stderr(process::Stdio::piped());
                if let Some(dir) = &self.dir {
                    command.current_dir(dir);
                }
                let mut process = command.spawn()?;
                let status = process.wait()?;
                let mut stdout = String::new();
                process.stdout.unwrap().read_to_string(&mut stdout)?;
                let mut stderr = String::new();
                process.stderr.unwrap().read_to_string(&mut stderr)?;
                Ok(CommandRun {
                    status,
                    stdout,
                    stderr,
                })
            }
        }
    }
}

impl Default for Command {
    fn default() -> Self {
        Self::new()
    }
}
