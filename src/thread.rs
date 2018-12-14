use std::ops::Drop;
use supercow::{Phantomcow, Supercow};

use crate::ffi;
use crate::utils::{ToStr, ScopedSupercow, ScopedPhantomcow};
use crate::Messages;
use crate::MessageOwner;
use crate::Tags;
use crate::TagsOwner;

pub trait ThreadOwner: Send + Sync {}

#[derive(Debug)]
pub(crate) struct ThreadPtr {
    pub ptr: *mut ffi::notmuch_thread_t,
}

#[derive(Debug)]
pub struct Thread<'o, O>
where
    O: ThreadOwner,
{
    pub(crate) handle: ThreadPtr,
    pub(crate) marker: ScopedPhantomcow<'o, O>,
}

impl<'o, O> MessageOwner for Thread<'o, O> where O: ThreadOwner + 'o {}
impl<'o, O> TagsOwner for Thread<'o, O> where O: ThreadOwner + 'o {}

impl<'o, O> Thread<'o, O>
where
    O: ThreadOwner + 'o,
{
    pub fn from_ptr<P>(ptr: *mut ffi::notmuch_thread_t, owner: P) -> Thread<'o, O>
    where
        P: Into<ScopedPhantomcow<'o, O>>,
    {
        Thread {
            handle: ThreadPtr { ptr },
            marker: owner.into(),
        }
    }

    pub fn id(self: &Self) -> String {
        let tid = unsafe { ffi::notmuch_thread_get_thread_id(self.handle.ptr) };
        tid.to_str().unwrap().to_string()
    }

    pub fn total_messages(self: &Self) -> i32 {
        unsafe { ffi::notmuch_thread_get_total_messages(self.handle.ptr) }
    }

    #[cfg(feature = "0.26")]
    pub fn total_files(self: &Self) -> i32 {
        unsafe { ffi::notmuch_thread_get_total_files(self.handle.ptr) }
    }

    pub fn toplevel_messages(self: &Self) -> Messages<'_, Self> {
        <Self as ThreadExt<'o, O>>::toplevel_messages(self)
    }

    /// Get a `Messages` iterator for all messages in 'thread' in
    /// oldest-first order.
    pub fn messages(self: &Self) -> Messages<'_, Self> {
        <Self as ThreadExt<'o, O>>::messages(self)
    }

    pub fn tags(&self) -> Tags<'_, Self> {
        <Self as ThreadExt<'o, O>>::tags(self)
    }

    pub fn subject(self: &Self) -> String {
        let sub = unsafe { ffi::notmuch_thread_get_subject(self.handle.ptr) };

        sub.to_str().unwrap().to_string()
    }

    pub fn authors(self: &Self) -> Vec<String> {
        let athrs = unsafe { ffi::notmuch_thread_get_authors(self.handle.ptr) };

        athrs
            .to_str()
            .unwrap()
            .split(',')
            .map(|s| s.to_string())
            .collect()
    }

    /// Get the date of the oldest message in 'thread' as a time_t value.
    pub fn oldest_date(self: &Self) -> i64 {
        unsafe { ffi::notmuch_thread_get_oldest_date(self.handle.ptr) as i64 }
    }

    /// Get the date of the newest message in 'thread' as a time_t value.
    pub fn newest_date(self: &Self) -> i64 {
        unsafe { ffi::notmuch_thread_get_newest_date(self.handle.ptr) as i64 }
    }
}

pub trait ThreadExt<'o, O>
where
    O: ThreadOwner + 'o,
{
    fn tags<'s, S>(thread: S) -> Tags<'s, Thread<'o, O>>
    where
        S: Into<ScopedSupercow<'s, Thread<'o, O>>>,
    {
        let threadref = thread.into();
        Tags::from_ptr(
            unsafe { ffi::notmuch_thread_get_tags(threadref.handle.ptr) },
            ScopedSupercow::phantom(threadref),
        )
    }

    fn toplevel_messages<'s, S>(thread: S) -> Messages<'s, Thread<'o, O>>
    where
        S: Into<ScopedSupercow<'s, Thread<'o, O>>>,
    {
        let threadref = thread.into();
        Messages::from_ptr(
            unsafe { ffi::notmuch_thread_get_toplevel_messages(threadref.handle.ptr) },
            ScopedSupercow::phantom(threadref),
        )
    }

    /// Get a `Messages` iterator for all messages in 'thread' in
    /// oldest-first order.
    fn messages<'s, S>(thread: S) -> Messages<'s, Thread<'o, O>>
    where
        S: Into<ScopedSupercow<'s, Thread<'o, O>>>,
    {
        let threadref = thread.into();
        Messages::from_ptr(
            unsafe { ffi::notmuch_thread_get_messages(threadref.handle.ptr) },
            ScopedSupercow::phantom(threadref),
        )
    }
}

impl<'o, O> ThreadExt<'o, O> for Thread<'o, O> where O: ThreadOwner + 'o {}

unsafe impl<'o, O> Send for Thread<'o, O> where O: ThreadOwner + 'o {}
unsafe impl<'o, O> Sync for Thread<'o, O> where O: ThreadOwner + 'o {}
