use crate::qcli::command::votes::CrsCommand;
use assert_cmd::assert::OutputAssertExt;
use quibitestkit::prelude::ProcessOutput;

pub struct Crs {
    crs_command: CrsCommand,
}

impl Crs {
    pub fn new(crs_command: CrsCommand) -> Self {
        Self { crs_command }
    }

    pub fn generate(self) -> String {
        self.crs_command
            .generate()
            .build()
            .assert()
            .success()
            .get_output()
            .as_single_line()
    }
}
