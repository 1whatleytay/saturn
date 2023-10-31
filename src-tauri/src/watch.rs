use std::ops::{Deref, DerefMut};
use std::sync::{Mutex, MutexGuard};
use tauri::ClipboardManager;

// (New, Old)
type Listener<T> = Box<dyn Fn(&T, &T) + Send + Sync>;

pub struct WatchLock<'a, T> {
    inner: MutexGuard<'a, WatchInner<T>>
}

impl<'a, T> Deref for WatchLock<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner.content
    }
}

pub struct WatchLockSilent<'a, T> {
    inner: MutexGuard<'a, WatchInner<T>>
}

impl<'a, T> Deref for WatchLockSilent<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner.content
    }
}

impl<'a, T> DerefMut for WatchLockSilent<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.inner.content
    }
}

pub struct WatchLockMut<'a, T> {
    inner: MutexGuard<'a, WatchInner<T>>
}

impl<'a, T> Deref for WatchLockMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner.content
    }
}

impl<'a, T> DerefMut for WatchLockMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.inner.content
    }
}

impl<'a, T> Drop for WatchLockMut<'a, T> {
    fn drop(&mut self) {
        for listener in &self.inner.listeners {
            listener(&self.inner.content, &self.inner.last)
        }
    }
}

pub struct WatchInner<T> {
    listeners: Vec<Listener<T>>,
    last: T,
    content: T
}

pub struct Watch<T: Clone> {
    inner: Mutex<WatchInner<T>>
}

impl<T: Clone> Watch<T> {
    pub fn new(content: T) -> Self {
        Self {
            inner: Mutex::new(WatchInner {
                listeners: vec![],
                last: content.clone(),
                content
            })
        }
    }

    pub fn get(&self) -> WatchLock<T> {
        WatchLock {
            inner: self.inner.lock().unwrap()
        }
    }

    pub fn get_mut(&self) -> WatchLockMut<T> {
        let mut inner = self.inner.lock().unwrap();
        inner.last = inner.content.clone();

        WatchLockMut { inner }
    }

    pub fn get_silent(&self) -> WatchLockSilent<T> {
        WatchLockSilent {
            inner: self.inner.lock().unwrap()
        }
    }

    pub fn listen<F: Fn(&T, &T) + Send + Sync + 'static>(&self, listener: F) {
        self.inner.lock().unwrap().listeners.push(Box::new(listener));
    }
}
