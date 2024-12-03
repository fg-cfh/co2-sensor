#![no_std]
#![no_main]

use core::panic;
use core::sync::atomic::{AtomicBool, Ordering};
use critical_section::Mutex;

pub trait Initialized {
    fn init(self);
}

pub trait RefProvider<T> {
    fn get(&self) -> &T
    where
        T: Sized;
}

pub trait MutRefProvider<T> {
    fn get_mut(&mut self) -> &mut T;
}

pub trait SingletonHolder<T>: Sync
where
    T: 'static,
{
    fn init_once<I, F: FnOnce(I) -> T>(&mut self, dependencies: I, f: F);

    fn with_state<R, F: FnOnce(T) -> (T, R)>(&mut self, f: F) -> R;

    fn is_initialized(&mut self) -> bool;
}

impl<T> SingletonHolder<T> for Mutex<Option<T>>
where
    T: 'static + Send,
{
    fn init_once<I, F: FnOnce(I) -> T>(&mut self, dependencies: I, f: F) {
        let prev = self.get_mut();
        if prev.is_none() {
            prev.replace(f(dependencies));
        } else {
            panic!("already initialized");
        }
    }

    fn with_state<R, F: FnOnce(T) -> (T, R)>(&mut self, f: F) -> R {
        let opt = self.get_mut();
        let prev = opt.take().expect("singleton not initialized");
        let (after, result) = f(prev);
        opt.replace(after);
        result
    }

    fn is_initialized(&mut self) -> bool {
        self.get_mut().is_some()
    }
}

impl SingletonHolder<bool> for AtomicBool {
    fn init_once<I, F: FnOnce(I) -> bool>(&mut self, dependencies: I, f: F) {
        self.compare_exchange(false, f(dependencies), Ordering::Acquire, Ordering::Relaxed)
            .expect("already initialized");
    }

    fn with_state<R, F: FnOnce(bool) -> (bool, R)>(&mut self, f: F) -> R {
        let val = self.get_mut();
        let (next_val, result) = f(*val);
        *val = next_val;
        result
    }

    fn is_initialized(&mut self) -> bool {
        self.load(Ordering::Acquire)
    }
}

pub struct SingletonHolderImpl<T>
where
    T: 'static,
{
    state: Mutex<Option<T>>,
}

impl<T> SingletonHolderImpl<T>
where
    T: 'static + Send,
{
    fn new() -> SingletonHolderImpl<T> {
        SingletonHolderImpl {
            state: Mutex::new(None),
        }
    }
}

impl<T> Default for SingletonHolderImpl<T>
where
    T: 'static + Send,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SingletonHolder<T> for SingletonHolderImpl<T>
where
    T: 'static + Send,
{
    fn init_once<I, F: FnOnce(I) -> T>(&mut self, input: I, f: F) {
        self.state.init_once(input, f);
    }

    fn with_state<DS, F: FnOnce(T) -> (T, DS)>(&mut self, f: F) -> DS {
        self.state.with_state(f)
    }

    fn is_initialized(&mut self) -> bool {
        self.state.is_initialized()
    }
}

pub trait Singleton<T, I>: Sized
where
    T: 'static + Send,
    I: 'static,
{
    fn with_dependencies<F>(f: F)
    where
        F: FnOnce(I);

    fn with_state_holder<F, R>(f: F) -> R
    where
        F: FnOnce(&'static mut SingletonHolderImpl<T>) -> R;

    fn from(input: I) -> T;

    fn is_initialized() -> bool {
        Self::with_state_holder(|state: &'static mut SingletonHolderImpl<T>| state.is_initialized())
    }

    fn ensure_is_initialized() {
        if !Self::is_initialized() {
            Self::init_once();
        }
    }

    fn init_once() {
        Self::with_dependencies(|dependencies: I| {
            Self::with_state_holder(|state: &'static mut SingletonHolderImpl<T>| {
                state.init_once(dependencies, Self::from);
            });
        });
    }

    fn with_state<R, F>(f: F) -> R
    where
        F: FnOnce(T) -> (T, R),
    {
        Self::with_state_holder(|state: &'static mut SingletonHolderImpl<T>| state.with_state(f))
    }
}
