pub use ringbuf::{Producer, Consumer, RingBuffer};
use std::thread;

pub fn transmit<T: Copy>(tx: &mut Producer<T>, t: &T) {
    while tx.is_full() {
        thread::sleep(std::time::Duration::from_millis(1));
    }

    let n = tx.push_slice(std::slice::from_ref(t));
    assert_eq!(n, 1, "transmit: failed to push slice");
}

pub fn receive<T: Copy>(rx: &mut Consumer<T>, t: &mut T) {
    while rx.is_empty() {
        thread::sleep(std::time::Duration::from_millis(1));
    }

    *t = rx.pop().unwrap();
}

pub fn drain<T: Copy>(rx: &mut Consumer<T>, t: &mut T) {
    while rx.is_empty() {
        thread::sleep(std::time::Duration::from_millis(1));
    }

    while rx.len() > 1 {
        rx.pop().unwrap();
    }

    *t = rx.pop().unwrap();
}