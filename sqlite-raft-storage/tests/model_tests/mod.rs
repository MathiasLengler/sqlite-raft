use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn model_example() {
    model! {
        Model => let m = AtomicUsize::new(0),
        Implementation => let mut i: usize = 0,
        Add(usize)(v in 0usize..4) => {
            let expected = m.fetch_add(v, Ordering::SeqCst) + v;
            i += v;
            assert_eq!(expected, i);
        },
        Set(usize)(v in 0usize..4) => {
            m.store(v, Ordering::SeqCst);
            i = v;
        },
        Eq(usize)(v in 0usize..4) => {
            let expected = m.load(Ordering::SeqCst) == v;
            let actual = i == v;
            assert_eq!(expected, actual);
        },
        Cas((usize, usize))((old, new) in (0usize..4, 0usize..4)) => {
            let expected =
                m.compare_and_swap(old, new, Ordering::SeqCst);
            let actual = if i == old {
                i = new;
                old
            } else {
                i
            };
            assert_eq!(expected, actual);
        }
    }
}

#[test]
fn mode_storage() {
    // TODO:

    model! {
        Model => let m = AtomicUsize::new(0),
        Implementation => let mut i: usize = 0,
        Add(usize)(v in 0usize..4) => {
            let expected = m.fetch_add(v, Ordering::SeqCst) + v;
            i += v;
            assert_eq!(expected, i);
        },
        Set(usize)(v in 0usize..4) => {
            m.store(v, Ordering::SeqCst);
            i = v;
        },
        Eq(usize)(v in 0usize..4) => {
            let expected = m.load(Ordering::SeqCst) == v;
            let actual = i == v;
            assert_eq!(expected, actual);
        },
        Cas((usize, usize))((old, new) in (0usize..4, 0usize..4)) => {
            let expected =
                m.compare_and_swap(old, new, Ordering::SeqCst);
            let actual = if i == old {
                i = new;
                old
            } else {
                i
            };
            assert_eq!(expected, actual);
        }
    }
}
