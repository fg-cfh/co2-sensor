#![no_std]
#![no_main]

pub mod singleton;
pub mod token;

pub trait Initialized {
    fn init(&self);

    fn is_initialized(&self) -> bool;

    fn ensure_is_initialized(&self) {
        if !self.is_initialized() {
            self.init();
        }
    }
}

pub trait WithDependency<Dependency> {
    fn with_dependency<Result, F: FnOnce(Dependency) -> Result>(f: F) -> Result;
}
