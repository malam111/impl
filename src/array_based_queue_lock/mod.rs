enum Qstat {
    Go,
    Wait,
}

struct Qlock {
    queue: Vec<Qstat>,
    last_q: usize,
}

impl Qlock {

    fn new(num: usize) -> Self {
        Qlock {
            queue: Vec::with_capacity(num),
            last_q: 0,
        }
    }

    fn queue() -> Self {

    }
}

#[cfg(test)]
fn test {

    use super::*;
    use std::sync::{Arc};

    const THREAD_NUM: usize = 4;

    #[test]
    fn main_test() {
        let queue = Arc::new(Qlock::new(THREAD_NUM));

        assert!(1);
    }
}
