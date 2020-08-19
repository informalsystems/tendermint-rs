use crate::helpers::*;
use crate::tester::TestResult::{Failure, ParseError, ReadError, Success};
use serde::de::DeserializeOwned;
use std::panic::UnwindSafe;
use std::path::Path;
use std::{fs, path::PathBuf};
use std::{
    panic,
    sync::{Arc, Mutex},
};
use tempfile::TempDir;

#[derive(Debug, Clone)]
pub struct TestEnv {
    /// Directory where the test is being executed
    current_dir: String,
    logs: Vec<String>,
}

impl TestEnv {
    pub fn new(current_dir: &str) -> Option<Self> {
        fs::create_dir_all(current_dir).ok().and_then(|_| {
            Some(TestEnv {
                current_dir: current_dir.to_string(),
                logs: vec![],
            })
        })
    }

    pub fn cleanup(&self) -> Option<()> {
        fs::remove_dir_all(&self.current_dir)
            .ok()
            .and(fs::create_dir_all(&self.current_dir).ok())
    }

    pub fn push(&self, child: &str) -> Option<Self> {
        let mut path = PathBuf::from(&self.current_dir);
        path.push(child);
        path.to_str().and_then(|path| TestEnv::new(path))
    }

    pub fn current_dir(&self) -> &str {
        &self.current_dir
    }

    pub fn add_log(&mut self, log: &str) {
        self.logs.push(log.to_string());
    }

    /// Read a file from a path relative to the environment current dir into a string
    pub fn read_file(&self, rel_path: &str) -> Option<String> {
        self.full_path(rel_path)
            .and_then(|full_path| fs::read_to_string(&full_path).ok())
    }

    /// Parse a file from a path relative to the environment current dir as the given type
    pub fn parse_file_as<T: DeserializeOwned>(&self, rel_path: &str) -> Option<T> {
        self.read_file(rel_path)
            .and_then(|input| serde_json::from_str(&input).ok())
    }

    /// Copy a file from the path outside environment into the environment current dir
    /// Returns the relative path of the file, or None if copying was not successful
    pub fn copy_file_from(&self, path: &str) -> Option<String> {
        let path = Path::new(path);
        if !path.is_file() {
            return None;
        }
        path.file_name().and_then(|name| {
            name.to_str().and_then(|name| {
                self.full_path(name).and_then(|dest| {
                    fs::copy(path, dest)
                        .ok()
                        .and_then(|_| Some(name.to_string()))
                })
            })
        })
    }

    /// Copy a file from the path relative to the other environment into the environment current dir
    /// Returns the relative path of the file, or None if copying was not successful
    pub fn copy_file_from_env(&self, other: &TestEnv, path: &str) -> Option<String> {
        other
            .full_path(path)
            .and_then(|full_path| self.copy_file_from(&full_path))
    }

    /// Convert a relative path to the full path from the test root
    /// Return None if the full path can't be formed
    pub fn full_path(&self, rel_path: &str) -> Option<String> {
        let full_path = PathBuf::from(&self.current_dir).join(rel_path);
        match full_path.to_str() {
            None => None,
            Some(full_path) => Some(full_path.to_string()),
        }
    }

    /// Convert a full path to the path relative to the test root
    /// Returns None if the full path doesn't contain test root as prefix
    pub fn rel_path(&self, full_path: &str) -> Option<String> {
        match PathBuf::from(full_path).strip_prefix(&self.current_dir) {
            Err(_) => None,
            Ok(rel_path) => match rel_path.to_str() {
                None => None,
                Some(rel_path) => Some(rel_path.to_string()),
            },
        }
    }

    /// Convert a relative path to the full path from the test root, canonicalized
    /// Returns None the full path can't be formed
    pub fn full_canonical_path(&self, rel_path: &str) -> Option<String> {
        let full_path = PathBuf::from(&self.current_dir).join(rel_path);
        full_path
            .canonicalize()
            .ok()
            .and_then(|p| p.to_str().map(|x| x.to_string()))
    }
}

#[derive(Debug, Clone)]
pub enum TestResult {
    ReadError,
    ParseError,
    Success(TestEnv),
    Failure { message: String, location: String },
}

/// A function that takes as input the test file path and its content,
/// and returns the result of running the test on it
type TestFn = Box<dyn Fn(&str, &str) -> TestResult>;

pub struct Test {
    /// test name
    pub name: String,
    /// test function
    pub test: TestFn,
}

pub struct Tester {
    name: String,
    root_dir: String,
    tests: Vec<Test>,
    results: std::collections::BTreeMap<String, Vec<(String, TestResult)>>,
}

impl TestResult {
    pub fn is_success(&self) -> bool {
        match self {
            TestResult::Success(_) => true,
            _ => false,
        }
    }
    pub fn is_failure(&self) -> bool {
        match self {
            TestResult::Failure {
                message: _,
                location: _,
            } => true,
            _ => false,
        }
    }
    pub fn is_readerror(&self) -> bool {
        match self {
            TestResult::ReadError => true,
            _ => false,
        }
    }
    pub fn is_parseerror(&self) -> bool {
        match self {
            TestResult::ParseError => true,
            _ => false,
        }
    }
}

impl Tester {
    pub fn new(name: &str, root_dir: &str) -> Tester {
        Tester {
            name: name.to_string(),
            root_dir: root_dir.to_string(),
            tests: vec![],
            results: Default::default(),
        }
    }

    pub fn env(&self) -> Option<TestEnv> {
        TestEnv::new(&self.root_dir)
    }

    pub fn output_env(&self) -> Option<TestEnv> {
        fs::create_dir_all(self.root_dir.clone() + "/_" + &self.name)
            .ok()
            .and(TestEnv::new(&(self.root_dir.clone() + "/_" + &self.name)))
    }

    fn capture_test<F>(test: F) -> TestResult
    where
        F: FnOnce() -> TestEnv + UnwindSafe,
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
        let result = panic::catch_unwind(|| test());
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
        let test_env = self.env().unwrap();
        let test_fn = move |_path: &str, input: &str| match parse_as::<T>(&input) {
            Ok(test_case) => Tester::capture_test(|| {
                test(test_case);
                test_env.clone()
            }),
            Err(_) => ParseError,
        };
        self.tests.push(Test {
            name: name.to_string(),
            test: Box::new(test_fn),
        });
    }

    pub fn add_test_with_env<T>(&mut self, name: &str, test: fn(T, &TestEnv, &TestEnv, &TestEnv))
    where
        T: 'static + DeserializeOwned + UnwindSafe,
    {
        let test_env = self.env().unwrap();
        let output_env = self.output_env().unwrap();
        let test_fn = move |path: &str, input: &str| match parse_as::<T>(&input) {
            Ok(test_case) => Tester::capture_test(|| {
                let dir = TempDir::new().unwrap();
                let env = TestEnv::new(dir.path().to_str().unwrap()).unwrap();
                let output_dir = output_env.full_path(path).unwrap();
                let output_env = TestEnv::new(&output_dir).unwrap();
                test(test_case, &env, &test_env, &output_env);
                env
            }),
            Err(_) => ParseError,
        };
        self.tests.push(Test {
            name: name.to_string(),
            test: Box::new(test_fn),
        });
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
        match self.env().unwrap().read_file(path) {
            None => self.read_error(path),
            Some(input) => {
                let mut results = Vec::new();
                for Test { name, test } in &self.tests {
                    match test(path, &input) {
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
                            // ignore path components starting with '_'
                            if let Some(last) = entry.path().iter().rev().next() {
                                if let Some(last) = last.to_str() {
                                    if last.starts_with("_") {
                                        continue;
                                    }
                                }
                            }
                            if let Ok(kind) = entry.file_type() {
                                let path = format!("{}", entry.path().display());
                                let rel_path = self.env().unwrap().rel_path(&path).unwrap();
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
