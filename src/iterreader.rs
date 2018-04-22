use std::io::{Read, Result};

pub struct IterReader<I: Iterator<Item = u8>>(pub I);

impl<'a, I: Iterator<Item = u8>> Read for IterReader<I> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut count: usize = 0;
        for i in 0..buf.len() {
            match self.0.next() {
                Some(v) => {
                    buf[i] = v;
                    count += 1;
                }
                None => break,
            }
        }
        Ok(count)
    }
}
