#![feature(try_trait)]
#![feature(iterator_try_fold)]
extern crate byteorder;

pub mod error;
pub mod status;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
