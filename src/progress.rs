//! is a helper for reporting progress
//! depends on glib, but not gtk.
use std::iter::{ExactSizeIterator, Zip};

#[derive(Clone)]
pub struct Progress<'a> {
    //TODO: do we need SyncSender? what's even the difference.. they both implement Sync! ;)
    tx: glib::Sender<Option<(f64, String)>>,
    parent: Option<(&'a Progress<'a>, &'a str)>,
    start: f64,
    len: f64,
}

pub struct ProgressIter<'a> {
    progress: Progress<'a>,
    remaining: usize,
}

impl<'a> Progress<'a> {
    pub fn new(tx: glib::Sender<Option<(f64, String)>>) -> Self {
        Progress{
            tx,
            parent: None,
            start: 0f64,
            len: 1f64
        }
    }

    pub fn send(&self, f: f64, message: &str) {
        self.tx.send(Some((
            self.start+f*self.len,
            self.message() + message
        ))).unwrap();
    }

    fn message(&self) -> String {
        if let Some((parent, message)) = self.parent{
            let mut report = parent.message();
            report.push_str(message);
            report.push_str(": ");
            report
        }else{
            "".to_owned()
        }
    }

    pub fn over<'b, I: ExactSizeIterator>(&'b self, i: I, message: &'b str) -> Zip<ProgressIter, I> {
        self.divide(i.len(), message).zip(i)
    }

    pub fn divide<'b>(&'b self, count: usize, message: &'b str) -> ProgressIter {
        let mut progress = self.clone();
        progress.len /= count as f64;
        //TODO: include a (4/6 in the message by moving some stuff around)
        progress.parent = Some((&self, message));
        ProgressIter{
            progress,
            remaining: count,
        }
    }

    pub fn finish(self) {
        if let Some(_) = self.parent {
            // we're just a sub progress
            self.send(1f64, "complete");
        }else{
            // we're the master progress
            self.tx.send(None).unwrap();
        }
    }
}

impl<'a> Iterator for ProgressIter<'a> {
    type Item = Progress<'a>;

    fn next(&mut self) -> Option<Progress<'a>> {
        if self.remaining == 0 { return None; }
        let r = self.progress.clone();
        self.progress.start += self.progress.len;
        self.remaining -= 1;
        Some(r)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<'a> ExactSizeIterator for ProgressIter<'a> {}
