use std::sync::atomic::{AtomicUsize, Ordering};
use std::path::PathBuf;
use raft::storage::MemStorage;
use sqlite_raft_storage::SqliteStorage;
use sqlite_raft_storage::storage_traits::StorageTestable;


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
fn model_storage() {
    use utils::temp_db::with_test_db_path;


    // TODO: implement model description
    with_test_db_path(|test_db_path: PathBuf| {
        let mut mem_storage = MemStorage::new();
        test(&mut mem_storage);

        let mut sqlite_storage = SqliteStorage::open(test_db_path).unwrap();
        test(&mut sqlite_storage);
    });


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
