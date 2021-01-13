use std::{cell::RefCell, rc::Rc, io::Read};

// Buffer made up of smaller contiguous chunks. We use those to discard data more easily.

// We slighly break rules for buffers we can discard from. Trying to access data that was already
// discarded causes a panic.
pub trait DiscontiguousBuf {
    fn get_chunk(&self, start: usize) -> &[u8];
    fn get_mut_chunk(&mut self, start: usize) -> &mut [u8];
    fn len(&self) -> usize;
}

pub trait DiscontiguousBufExt : DiscontiguousBuf {
    // Compare with another buf from position start. Position end is the first position at which
    // the two streams differ (or end of one of the streams).
    //
    // TODO: this compiles to byte-by-byte comparison loop. Pretty good, but maybe we can do better
    // by comparing words or even SIMD. Profile.
    fn common_prefix_from(&self, other: &impl DiscontiguousBuf, start: usize) -> usize {
        let mut end = start;
        loop {
            let my_chunk = self.get_chunk(end);
            let other_chunk = other.get_chunk(end);
            if my_chunk.is_empty() || other_chunk.is_empty() {
                return end;
            }
            let eq_len = my_chunk.iter().zip(other_chunk).take_while(|(a, b)| a == b).count();
            end += eq_len;
            if eq_len < std::cmp::min(my_chunk.len(), other_chunk.len()) {
                return end;
            }
        }
    }
}
/* For merging incoming replays we want a data structure that we can append bytes to and discard
 * bytes from front whenever we want. We accept responsibility to never read data we already
 * discarded.
 */
pub trait BufWithDiscard {
    /* Discard all data up to 'until'.
     * Can discard beyond data end, in that case new writes will only increase the end value until
     * it reaches discard point.
     * */
    fn discard(&mut self, until: usize);
}

pub trait BufWithDiscardExt: BufWithDiscard {
    fn discard_all(&mut self) {
        self.discard(usize::MAX);
    }
}

/* Read that doesn't need to keep a reference to self. Needed when we use a mutably shared RefCell
 * in a task, so that we don't borrow across an await.
 */
pub trait ReadAt {
    fn read_at(&self, start: usize, buf: &mut [u8]) -> std::io::Result<usize>;
}

impl<T: DiscontiguousBuf> ReadAt for T {
    fn read_at(&self, start: usize, buf: &mut [u8]) -> std::io::Result<usize> {
        if start >= self.len() {
            return Ok(0);
        }
        self.get_chunk(start).read(buf)
    }
}

impl<T: ReadAt> ReadAt for Rc<RefCell<T>> {
    fn read_at(&self, start: usize, buf: &mut [u8]) -> std::io::Result<usize> {
        self.borrow().read_at(start, buf)
    }
}

pub struct ReadAtCursor<'a, T: ReadAt + ?Sized> {
    bwd: &'a T,
    start: usize,
}

impl<'a, T: ReadAt + ?Sized> Read for ReadAtCursor<'a, T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let res = self.bwd.read_at(self.start, buf)?;
        self.start += res;
        Ok(res)
    }
}

pub trait ReadAtExt: ReadAt {
    fn reader_from<'a>(&'a self, start: usize) -> ReadAtCursor<'a, Self> {
        ReadAtCursor {bwd: self, start}
    }
    fn reader<'a>(&'a self) -> ReadAtCursor<'a, Self> {
        self.reader_from(0)
    }
}