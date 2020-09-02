use crate::helpers::*;
use crate::tester::TestResult::{Failure, ParseError, ReadError, Success};
use serde::de::DeserializeOwned;
use std::panic::UnwindSafe;
use std::{fs, path::PathBuf};
use std::{
    panic,
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone)]
pub struct TestEnv {
    root_dir: String,
    logs: Vec<String>,
}

impl TestEnv {
    pub fn add_log(&mut self, log: &str) {
        self.logs.push(log.to_string());
    }

    /// Read a file from a path relative to the environment root into a string
    pub fn read_file(&mut self, path: &str) -> Option<String> {
        match self.full_path(path) {
            None => None,
            Some(full_path) => match fs::read_to_string(&full_path) {
                Ok(file) => Some(file),
                Err(_) => None,
            },
        }
    }

    /// Convert a relative path to the full path from the test root
    /// Return None if the full path can't be formed
    pub fn full_path(&self, rel_path: &str) -> Option<String> {
        let full_path = PathBuf::from(&self.root_dir).join(rel_path);
        match full_path.to_str() {
            None => None,
            Some(full_path) => Some(full_path.to_string()),
        }
    }

    /// Convert a full path to the path relative to the test root
    /// Return None if the full path doesn't contain test root as prefix
    pub fn rel_path(&self, full_path: &str) -> Option<String> {
        match PathBuf::from(full_path).strip_prefix(&self.root_dir) {
            Err(_) => None,
            Ok(rel_path) => match rel_path.to_str() {
                None => None,
                Some(rel_path) => Some(rel_path.to_string()),
            },
        }
    }
}

type TestFn = Box<dyn Fn(&str) -> TestResult>;

pub struct Tester {
    root_dir: String,
    tests: Vec<(String, TestFn)>,
    results: std::collections::BTreeMap<String, Vec<(String, TestResult)>>,
}

#[derive(Debug, Clone)]
pub enum TestResult {
    ReadError,
    ParseError,
    Success(TestEnv),
    Failure { message: String, location: String },
}

impl TestResult {
    pub fn is_success(&self) -> bool {
        matches!(self, TestResult::Success(_))
    }
    pub fn is_failure(&self) -> bool {
        matches!(self,
        TestResult::Failure {
            message: _,
            location: _,}
        )
    }
    pub fn is_readerror(&self) -> bool {
        matches!(self, TestResult::ReadError)
    }
    pub fn is_parseerror(&self) -> bool {
        matches!(self, TestResult::ParseError)
    }
}

impl Tester {
    pub fn new(root_dir: &str) -> Tester {
        Tester {
            root_dir: root_dir.to_string(),
            tests: vec![],
            results: Default::default(),
        }
    }

    pub fn env(&self) -> TestEnv {
        TestEnv {
            root_dir: self.root_dir.clone(),
            logs: vec![],
        }
    }

    fn capture_test<F>(env: TestEnv, test: F) -> TestResult
    where
        F: FnOnce(TestEnv) -> TestEnv + UnwindSafe,
    {
        let test_result = Arc::new(Mutex::new(ParseError));
        let old_hook = panic::take_hook();
        panic::set_hook({
            let result = test_result.clone();
            Box::new(move |info| {
                let mut result = result.lock().unwrap();
                let message = match info.payload().downcast_ref::<&'static str>() {
                    Some(s) => s.to_string(),
                    None => match info.payload().downcast_ref::<String>() {
                        Some(s) => s.clone(),
                        None => "Unknown error".to_string(),
                    },
                };
                let location = match info.location() {
                    Some(l) => l.to_string(),
                    None => "".to_string(),
                };
                *result = Failure { message, location };
            })
        });
        let test_fun = || test(env.clone());
        let result = panic::catch_unwind(test_fun);
        panic::set_hook(old_hook);
        match result {
            Ok(res) => Success(res),
            Err(_) => (*test_result.lock().unwrap()).clone(),
        }
    }

    pub fn add_test<T>(&mut self, name: &str, test: fn(T))
    where
        T: 'static + DeserializeOwned + UnwindSafe,
    {
        let test_env = self.env();
        let test_fn = move |input: &str| match parse_as::<T>(&input) {
            Ok(test_case) => Tester::capture_test(test_env.clone(), |env| {
                test(test_case);
                env
            }),
            Err(_) => ParseError,
        };
        self.tests.push((name.to_string(), Box::new(test_fn)));
    }

    pub fn add_test_with_env<T>(&mut self, name: &str, test: fn(T, &mut TestEnv))
    where
        T: 'static + DeserializeOwned + UnwindSafe,
    {
        let test_env = self.env();
        let test_fn = move |input: &str| match parse_as::<T>(&input) {
            Ok(test_case) => Tester::capture_test(test_env.clone(), |env| {
                let mut env = env;
                test(test_case, &mut env);
                env
            }),
            Err(_) => ParseError,
        };
        self.tests.push((name.to_string(), Box::new(test_fn)));
    }

    fn add_result(&mut self, name: &str, path: &str, result: TestResult) {
        self.results
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push((path.to_string(), result))
    }

    fn read_error(&mut self, path: &str) {
        self.results
            .entry("".to_string())
            .or_insert_with(Vec::new)
            .push((path.to_string(), TestResult::ReadError))
    }

    fn parse_error(&mut self, path: &str) {
        self.results
            .entry("".to_string())
            .or_insert_with(Vec::new)
            .push((path.to_string(), TestResult::ParseError))
    }

    pub fn successful_tests(&self, test: &str) -> Vec<(String, TestEnv)> {
        let mut tests = Vec::new();
        if let Some(results) = self.results.get(test) {
            for (path, res) in results {
                if let Success(env) = res {
                    tests.push((path.clone(), env.clone()))
                }
            }
        }
        tests
    }

    pub fn failed_tests(&self, test: &str) -> Vec<(String, String, String)> {
        let mut tests = Vec::new();
        if let Some(results) = self.results.get(test) {
            for (path, res) in results {
                if let Failure { message, location } = res {
                    tests.push((path.clone(), message.clone(), location.clone()))
                }
            }
        }
        tests
    }

    pub fn unreadable_tests(&self) -> Vec<String> {
        let mut tests = Vec::new();
        if let Some(results) = self.results.get("") {
            for (path, res) in results {
                if let ReadError = res {
                    tests.push(path.clone())
                }
            }
        }
        tests
    }

    pub fn unparseable_tests(&self) -> Vec<String> {
        let mut tests = Vec::new();
        if let Some(results) = self.results.get("") {
            for (path, res) in results {
                if let ParseError = res {
                    tests.push(path.clone())
                }
            }
        }
        tests
    }

    pub fn print_results(&mut self) {
        let tests = self.unreadable_tests();
        if !tests.is_empty() {
            println!("Unreadable tests:  ");
            for path in tests {
                println!("  > {}", path)
            }
            panic!("Some tests could not be read");
        }
        let tests = self.unparseable_tests();
        if !tests.is_empty() {
            println!("Unparseable tests:  ");
            for path in tests {
                println!("  > {}", path)
            }
            panic!("Some tests could not be parsed");
        }

        for name in self.results.keys() {
            println!("Results for '{}'", name);
            let tests = self.successful_tests(name);
            if !tests.is_empty() {
                println!("  Successful tests:  ");
                for (path, _) in tests {
                    println!("    > {}", path)
                }
            }
            let tests = self.failed_tests(name);
            if !tests.is_empty() {
                println!("  Failed tests:  ");
                for (path, message, location) in tests {
                    println!("    > {}, '{}', {}", path, message, location)
                }
                panic!("Some tests failed");
            }
        }
    }

    pub fn run_for_file(&mut self, path: &str) {
        match self.env().read_file(path) {
            None => self.read_error(path),
            Some(input) => {
                let mut results = Vec::new();
                for (name, test) in &self.tests {
                    match test(&input) {
                        TestResult::ParseError => continue,
                        res => results.push((name.to_string(), path, res)),
                    }
                }
                if results.is_empty() {
                    self.parse_error(path);
                } else {
                    for (name, path, res) in results {
                        self.add_result(&name, path, res)
                    }
                }
            }
        }
    }

    pub fn run_foreach_in_dir(&mut self, dir: &str) {
        let full_dir = PathBuf::from(&self.root_dir).join(dir);
        match full_dir.to_str() {
            None => self.read_error(dir),
            Some(full_dir) => match fs::read_dir(full_dir) {
                Err(_) => self.read_error(full_dir),
                Ok(paths) => {
                    for path in paths {
                        if let Ok(entry) = path {
                            if let Ok(kind) = entry.file_type() {
                                let path = format!("{}", entry.path().display());
                                let rel_path = self.env().rel_path(&path).unwrap();
                                if kind.is_file() || kind.is_symlink() {
                                    if !rel_path.ends_with(".json") {
                                        continue;
                                    } else {
                                        self.run_for_file(&rel_path);
                                    }
                                } else if kind.is_dir() {
                                    self.run_foreach_in_dir(&rel_path);
                                }
                            }
                        }
                    }
                }
            },
        }
    }
}
