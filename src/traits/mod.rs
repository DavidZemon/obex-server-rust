use crate::response_status::ResponseStatus;

use mockall::automock;

#[cfg_attr(test, automock)]
pub trait Runner {
    fn run(&self, cmd: Vec<&str>) -> Result<Output, ResponseStatus>;
}

pub struct Output {
    pub status: Box<dyn WithStatus>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[cfg_attr(test, automock)]
pub trait WithStatus {
    fn success(&self) -> bool;
}
