use std::ops::{Deref, DerefMut};
use std::sync::{Mutex, MutexGuard};

type Listener<T> = Box<dyn Fn(&T) + Send + Sync>;

pub struct WatchLock<'a, T> {
    inner: MutexGuard<'a, T>
}

impl<'a, T> Deref for WatchLock<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.inner.deref()
    }
}

pub struct WatchLockSilent<'a, T> {
    inner: MutexGuard<'a, T>
}

impl<'a, T> Deref for WatchLockSilent<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.inner.deref()
    }
}

impl<'a, T> DerefMut for WatchLockSilent<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.inner.deref_mut()
    }
}

pub struct WatchLockMut<'a, T> {
    parent: &'a Watch<T>,
    inner: MutexGuard<'a, T>
}

impl<'a, T> Deref for WatchLockMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.inner.deref()
    }
}

impl<'a, T> DerefMut for WatchLockMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.inner.deref_mut()
    }
}

impl<'a, T> Drop for WatchLockMut<'a, T> {
    fn drop(&mut self) {
        for listener in &self.parent.listeners {
            listener(self.inner.deref())
        }
    }
}

pub struct Watch<T> {
    listeners: Vec<Listener<T>>,
    content: Mutex<T>
}

impl<T> Watch<T> {
    pub fn new(content: T) -> Self {
        Self {
            listeners: vec![],
            content: Mutex::new(content)
        }
    }

    pub fn get(&self) -> WatchLock<T> {
        WatchLock {
            inner: self.content.lock().unwrap()
        }
    }

    pub fn get_mut(&self) -> WatchLockMut<T> {
        WatchLockMut {
            parent: self,
            inner: self.content.lock().unwrap()
        }
    }
    
    pub fn get_silent(&self) -> WatchLockSilent<T> {
        WatchLockSilent {
            inner: self.content.lock().unwrap()
        }
    }

    pub fn listen<F: Fn(&T) + Send + Sync + 'static>(mut self, listener: F) -> Self {
        self.listeners.push(Box::new(listener));

        self
    }
}
