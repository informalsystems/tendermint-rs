use std::{process, io};
use std::io::Read;

/// A thin wrapper around process::Command to facilitate running external commands.
pub struct Command {
    program: Option<String>,
    args: Vec<String>,
    dir: Option<String>
}

impl Command {

    /// Check whether the given program can be executed
    pub fn exists_program(program: &str) -> bool {
        Command::new().program(program).spawn().is_ok()
    }

    /// Constructs a new Command for the given program with arguments.
    pub fn new() -> Command {
        Command {
            program: None,
            args: vec![],
            dir: None
        }
    }

    /// Sets the program to run
    pub fn program(&mut self, program: &str) -> &mut Self {
        self.program = Some(program.to_owned());
        self
    }

    /// Adds a new program argument
    pub fn arg(&mut self, arg: &str) -> &mut Self {
        self.args.push(arg.to_owned());
        self
    }

    /// Adds a new program argument, concatenated from several parts
    pub fn arg_from_parts(&mut self, parts: Vec<&str>) -> &mut Self {
        let mut arg: String = String::new();
        for part in parts {
            arg = arg + part;
        }
        self.args.push(arg);
        self
    }

    /// Sets the working directory for the child process
    pub fn current_dir(&mut self, dir: &str) -> &mut Self {
        self.dir = Some(dir.to_owned());
        self
    }

    /// Executes the command as a child process, and extracts its status, stdout, stderr.
    pub fn spawn(&mut self) -> io::Result<CommandRun> {
        match &self.program {
            None => Err(io::Error::new(io::ErrorKind::InvalidInput, "")),
            Some(program) => {
                let mut command = process::Command::new(program);
                command.args(&self.args)
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
                    stderr
                })
            }
        }
    }
}

/// The result of a command execution if managed to run the child process
pub struct CommandRun {
    pub status: process::ExitStatus,
    pub stdout: String,
    pub stderr: String
}