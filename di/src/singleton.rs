use super::Initialized;
use core::cell::RefCell;
use core::panic;
use critical_section::Mutex;

pub trait SingletonHolder: Initialized + Default + Sync {
    type Content: 'static;

    fn with<Result, F: FnOnce(Self::Content) -> (Self::Content, Result)>(&self, f: F) -> Result;

    fn with_ref<Result, F>(&self, f: F) -> Result
    where
        F: FnOnce(&Self::Content) -> Result,
    {
        self.with(|state| {
            let result = f(&state);
            (state, result)
        })
    }

    fn with_ref_mut<Result, F>(&self, f: F) -> Result
    where
        F: FnOnce(&mut Self::Content) -> Result,
    {
        self.with(|mut state| {
            let result = f(&mut state);
            (state, result)
        })
    }
}

pub struct SingletonHolderImpl<Content>
where
    Content: 'static + Send,
{
    state: Mutex<RefCell<Option<Content>>>,
}

impl<Content> SingletonHolderImpl<Content>
where
    Content: Send,
{
    pub const fn new() -> Self {
        SingletonHolderImpl {
            state: Mutex::new(RefCell::new(None)),
        }
    }

    pub fn reset(&mut self) {
        critical_section::with(|cs| {
            let mut content = self.state.borrow_ref_mut(cs);
            *content = None;
        });
    }
}

impl<Content> Default for SingletonHolderImpl<Content>
where
    Content: Send,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Content> Initialized for SingletonHolderImpl<Content>
where
    Content: Default + Send,
{
    fn init(&self) {
        critical_section::with(|cs| {
            let mut prev = self.state.borrow_ref_mut(cs);
            if prev.is_none() {
                prev.replace(Default::default());
            } else {
                panic!("already initialized");
            }
        });
    }

    fn is_initialized(&self) -> bool {
        critical_section::with(|cs| self.state.borrow_ref(cs).is_some())
    }
}

impl<Content> SingletonHolder for SingletonHolderImpl<Content>
where
    Content: Default + Send,
{
    type Content = Content;

    fn with<Result, F: FnOnce(Content) -> (Content, Result)>(&self, f: F) -> Result {
        critical_section::with(|cs| {
            let mut opt = self.state.borrow_ref_mut(cs);
            let prev = opt.take().expect("singleton not initialized");
            let (after, result) = f(prev);
            opt.replace(after);
            result
        })
    }
}

pub trait Singleton: Sized + Sync {
    type Content: 'static + Default + Send;

    fn with_state_holder<Result, F>(f: F) -> Result
    where
        F: FnOnce(&SingletonHolderImpl<Self::Content>) -> Result;

    fn with<Result, F>(f: F) -> Result
    where
        F: FnOnce(Self::Content) -> (Self::Content, Result),
    {
        Self::with_state_holder(|state_holder| state_holder.with(f))
    }

    fn with_ref<Result, F>(f: F) -> Result
    where
        F: FnOnce(&Self::Content) -> Result,
    {
        Self::with_state_holder(|state_holder| state_holder.with_ref(f))
    }

    fn with_ref_mut<Result, F>(f: F) -> Result
    where
        F: FnOnce(&mut Self::Content) -> Result,
    {
        Self::with_state_holder(|state_holder| state_holder.with_ref_mut(f))
    }
}

impl<S, Content> Initialized for S
where
    Content: Default + Send,
    S: Singleton<Content = Content>,
{
    fn init(&self) {
        Self::with_state_holder(|state_holder| {
            state_holder.ensure_is_initialized();
        });
    }

    fn is_initialized(&self) -> bool {
        Self::with_state_holder(|state_holder| state_holder.is_initialized())
    }
}
