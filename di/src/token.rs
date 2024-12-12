use core::{
    fmt,
    mem::MaybeUninit,
    ops, ptr,
    sync::atomic::{self, AtomicUsize},
};

pub struct SharedToken<'a, Token: Release + Send + Default> {
    token: Token,
    count: &'a AtomicUsize,
}

pub trait Release {
    fn release(&self);
}

impl<'a, Token: Release + Send + Default> SharedToken<'a, Token> {
    pub fn new(token: Token) -> Self {
        static COUNT: AtomicUsize = AtomicUsize::new(0);
        const { assert!(size_of::<Token>() == 0) }
        Self {
            token,
            count: &COUNT,
        }
    }

    pub fn acquire(&self) -> Token {
        self.increment_uses();

        let mut raw_token: MaybeUninit<Token> = MaybeUninit::uninit();
        unsafe {
            // SAFETY: We're duplicating ZSTs with release semantics.
            ptr::copy_nonoverlapping(&self.token, raw_token.as_mut_ptr(), 1);
            raw_token.assume_init()
        }
    }

    pub fn release(&self, _token: Token) {
        if self.decrement_uses() == 0 {
            panic!("released too many tokens")
        }
    }

    fn increment_uses(&self) {
        let old_count = self.count.fetch_add(1, atomic::Ordering::Relaxed);
        if old_count >= MAX_TOKENS {
            panic!("max tokens reached");
        }
    }

    fn decrement_uses(&self) -> usize {
        self.count.fetch_sub(1, atomic::Ordering::Relaxed)
    }
}

const MAX_TOKENS: usize = isize::MAX as usize;

impl<'a, Token: Release + Send + Default> Clone for SharedToken<'a, Token> {
    fn clone(&self) -> Self {
        Self {
            token: self.acquire(),
            count: self.count,
        }
    }
}

impl<Token: Release + Send + Default> Drop for SharedToken<'_, Token> {
    fn drop(&mut self) {
        if self.decrement_uses() == 0 {
            self.token.release();
        }
    }
}

impl<Token: Release + Send + Default> fmt::Debug for SharedToken<'_, Token>
where
    Token: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Token::fmt(self, f)
    }
}

impl<Token: Release + Send + Default> fmt::Display for SharedToken<'_, Token>
where
    Token: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Token::fmt(self, f)
    }
}

impl<Token: Release + Send + Default> ops::Deref for SharedToken<'_, Token> {
    type Target = Token;

    fn deref(&self) -> &Self::Target {
        &self.token
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct TestToken;

    #[test]
    fn test_token() {
        let mut was_released = false;

        impl Release<Self> for TestToken {
            fn release(token: &Self) {
                was_released = true;
            }
        }

        let shared_token = SharedToken::new(Default::default());
    }
}
