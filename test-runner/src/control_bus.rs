//! Local(gtest) test results collector for onchain tests
use std::fmt;
use std::sync::{Arc, RwLock};

use codec::Decode;
use colored::Colorize;
use gtest::WasmProgram;

use gear_test_runtime::{ProgressSignal, TestInfo, TestUpdate};

#[derive(Debug, Default)]
pub struct ControlBus {
    running_state: Arc<RwLock<State>>,
}

#[derive(Debug, Default)]
pub struct State {
    started: u32,
    failed: u32,
    succeded: u32,
    unfinished: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Report {
    pub total_started: u32,
    pub total_failed: u32,
    pub total_succeded: u32,
    pub unfinished: Vec<String>,
}

impl State {
    pub fn submit_fail(&mut self, test_info: TestInfo) {
        self.failed += 1;
        self.remove(test_info);
    }

    pub fn submit_start(&mut self, test_info: TestInfo) {
        self.started += 1;
        self.append(test_info);
    }

    pub fn submit_success(&mut self, test_info: TestInfo) {
        self.succeded += 1;
        self.remove(test_info);
    }

    fn remove(&mut self, test_info: TestInfo) {
        let pos = self.unfinished.iter().position(|e| *e == test_info.name);
        if let Some(pos) = pos {
            self.unfinished.swap_remove(pos);
        }
    }

    fn append(&mut self, test_info: TestInfo) {
        self.unfinished.push(test_info.name)
    }

    pub fn report(&self) -> Report {
        Report {
            total_started: self.started,
            total_succeded: self.succeded,
            total_failed: self.failed,
            unfinished: self.unfinished.clone(),
        }
    }
}

impl Report {
    pub fn success(&self) -> bool {
        if self.unfinished.len() > 0 {
            false
        } else if self.total_failed != 0 {
            false
        } else if self.total_started != self.total_succeded {
            false
        } else {
            true
        }
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "test result: {}. {} passed; {} failed;",
            match self.success() {
                true => "ok".green(),
                false => "fail".red(),
            },
            self.total_succeded,
            self.total_failed
        )?;

        if !self.unfinished.is_empty() {
            write!(f, "unfinished tests: [")?;
            for unfinished in self.unfinished.iter() {
                write!(f, "{}", unfinished)?;
            }
            writeln!(f, "]")?;
        }

        Ok(())
    }
}

impl ControlBus {
    pub fn running_state(&self) -> Arc<RwLock<State>> {
        self.running_state.clone()
    }
}

impl WasmProgram for ControlBus {
    fn init(&mut self, _payload: Vec<u8>) -> Result<Option<Vec<u8>>, &'static str> {
        // does nothing!
        Ok(None)
    }

    fn handle(&mut self, payload: Vec<u8>) -> Result<Option<Vec<u8>>, &'static str> {
        let progress_signal =
            ProgressSignal::decode(&mut &payload[..]).expect("Failed to decode progress signal!");

        let ProgressSignal { test_info, update } = progress_signal;

        match update {
            TestUpdate::Start => {
                self.running_state.write().unwrap().submit_start(test_info);
            }
            TestUpdate::Success => {
                println!("test {} ... {}", test_info.name, "ok".green());
                self.running_state
                    .write()
                    .unwrap()
                    .submit_success(test_info);
            }
            TestUpdate::Fail(hint) => {
                println!("test {} ... {}", test_info.name, "fail".red());
                println!("\t --- ERROR REPORT @ {}", test_info.name);
                println!("{}", hint);
                println!("\t --- END OF REPORT @ {}", test_info.name);
                self.running_state.write().unwrap().submit_fail(test_info);
            }
        }

        Ok(None)
    }

    fn handle_reply(&mut self, _payload: Vec<u8>) -> Result<(), &'static str> {
        Ok(())
    }

    fn handle_signal(&mut self, _payload: Vec<u8>) -> Result<(), &'static str> {
        Ok(())
    }

    fn state(&mut self) -> Result<Vec<u8>, &'static str> {
        Ok(vec![])
    }
}
