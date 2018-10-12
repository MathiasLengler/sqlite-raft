// TODO: remove when all test are rewritten
#![allow(unused_imports)]
#![allow(dead_code)]

use protobuf;
use raft::{Error as RaftError, StorageError};
use raft::eraftpb::{ConfState, Entry, Snapshot};
use raft::storage::{MemStorage, Storage};
use sqlite_raft_storage::SqliteStorage;
use sqlite_raft_storage::storage_traits::{StorageMut, StorageTestable};
use std::path::PathBuf;
use utils::storage::test_storage_impls;
use utils::temp_db::with_test_db_path;

// TODO: rewrite rest of tests

// TODO extract these duplicated utility functions for tests
fn new_entry(index: u64, term: u64) -> Entry {
    let mut e = Entry::new();
    e.set_term(term);
    e.set_index(index);
    e
}

fn size_of<T: protobuf::Message>(m: &T) -> u32 {
    m.compute_size()
}

fn new_snapshot(index: u64, term: u64, nodes: Vec<u64>, data: Vec<u8>) -> Snapshot {
    let mut s = Snapshot::new();
    s.mut_metadata().set_index(index);
    s.mut_metadata().set_term(term);
    s.mut_metadata().mut_conf_state().set_nodes(nodes);
    s.set_data(data);
    s
}

#[test]
fn test_storage_term() {
    let ents = vec![new_entry(3, 3), new_entry(4, 4), new_entry(5, 5)];
    let mut tests = vec![
        (2, Err(RaftError::Store(StorageError::Compacted))),
        (3, Ok(3)),
        (4, Ok(4)),
        (5, Ok(5)),
        (6, Err(RaftError::Store(StorageError::Unavailable))),
    ];

    for (i, (idx, wterm)) in tests.drain(..).enumerate() {
        test_storage_impls(|storage: &mut dyn StorageTestable| {
            storage.set_entries(&ents);

            let t = storage.term(idx);
            if t != wterm {
                panic!("#{}: expect res {:?}, got {:?}. Storage:\n{:#?}", i, wterm, t, storage);
            }
        });
    }
}


#[test]
fn test_storage_entries() {
    let ents = vec![
        new_entry(3, 3),
        new_entry(4, 4),
        new_entry(5, 5),
        new_entry(6, 6),
    ];
    let max_u64 = u64::max_value();
    let mut tests = vec![
        (
            2,
            6,
            max_u64,
            Err(RaftError::Store(StorageError::Compacted)),
        ),
        (
            3,
            4,
            max_u64,
            Err(RaftError::Store(StorageError::Compacted)),
        ),
        (4, 5, max_u64, Ok(vec![new_entry(4, 4)])),
        (4, 6, max_u64, Ok(vec![new_entry(4, 4), new_entry(5, 5)])),
        (
            4,
            7,
            max_u64,
            Ok(vec![new_entry(4, 4), new_entry(5, 5), new_entry(6, 6)]),
        ),
        // even if maxsize is zero, the first entry should be returned
        (4, 7, 0, Ok(vec![new_entry(4, 4)])),
        // limit to 2
        (
            4,
            7,
            (size_of(&ents[1]) + size_of(&ents[2])) as u64,
            Ok(vec![new_entry(4, 4), new_entry(5, 5)]),
        ),
        (
            4,
            7,
            (size_of(&ents[1]) + size_of(&ents[2]) + size_of(&ents[3]) / 2) as u64,
            Ok(vec![new_entry(4, 4), new_entry(5, 5)]),
        ),
        (
            4,
            7,
            (size_of(&ents[1]) + size_of(&ents[2]) + size_of(&ents[3]) - 1) as u64,
            Ok(vec![new_entry(4, 4), new_entry(5, 5)]),
        ),
        // all
        (
            4,
            7,
            (size_of(&ents[1]) + size_of(&ents[2]) + size_of(&ents[3])) as u64,
            Ok(vec![new_entry(4, 4), new_entry(5, 5), new_entry(6, 6)]),
        ),
    ];
    for (i, (lo, hi, maxsize, wentries)) in tests.drain(..).enumerate() {
        test_storage_impls(|storage: &mut dyn StorageTestable| {
            storage.set_entries(&ents);

            let e = storage.entries(lo, hi, maxsize);
            if e != wentries {
                panic!("#{}: expect entries {:?}, got {:?}. Storage:\n{:#?}", i, wentries, e, storage);
            }
        });
    }
}


#[test]
fn test_storage_last_index() {
    let ents = vec![new_entry(3, 3), new_entry(4, 4), new_entry(5, 5)];

    test_storage_impls(|storage: &mut dyn StorageTestable| {
        storage.set_entries(&ents);

        let wresult = Ok(5);
        let result = storage.last_index();
        if result != wresult {
            panic!("want {:?}, got {:?}. Storage:\n{:#?}", wresult, result, storage);
        }

        storage.append(&[new_entry(6, 5)]).expect("append failed");

        let wresult = Ok(6);
        let result = storage.last_index();
        if result != wresult {
            panic!("want {:?}, got {:?}. Storage:\n{:#?}", wresult, result, storage);
        }
    });
}

// TODO: unignore when compact is implemented
#[test]
#[ignore]
fn test_storage_first_index() {
    let ents = vec![new_entry(3, 3), new_entry(4, 4), new_entry(5, 5)];

    test_storage_impls(|storage: &mut dyn StorageTestable| {
        storage.set_entries(&ents);

        let wresult = Ok(4);
        let result = storage.first_index();
        if result != wresult {
            panic!("want {:?}, got {:?}. Storage:\n{:#?}", wresult, result, storage);
        }

        storage.compact(4).expect("compact failed");
        let wresult = Ok(5);
        let result = storage.first_index();
        if result != wresult {
            panic!("want {:?}, got {:?}. Storage:\n{:#?}", wresult, result, storage);
        }
    });
}

//#[test]
//fn test_storage_compact() {
//    let ents = vec![new_entry(3, 3), new_entry(4, 4), new_entry(5, 5)];
//    let mut tests = vec![
//        (2, Err(RaftError::Store(StorageError::Compacted)), 3, 3, 3),
//        (3, Err(RaftError::Store(StorageError::Compacted)), 3, 3, 3),
//        (4, Ok(()), 4, 4, 2),
//        (5, Ok(()), 5, 5, 1),
//    ];
//    for (i, (idx, wresult, windex, wterm, wlen)) in tests.drain(..).enumerate() {
//        let storage = MemStorage::new();
//        storage.wl().entries = ents.clone();
//
//        let result = storage.wl().compact(idx);
//        if result != wresult {
//            panic!("#{}: want {:?}, got {:?}", i, wresult, result);
//        }
//        let index = storage.wl().entries[0].get_index();
//        if index != windex {
//            panic!("#{}: want {}, index {}", i, windex, index);
//        }
//        let term = storage.wl().entries[0].get_term();
//        if term != wterm {
//            panic!("#{}: want {}, term {}", i, wterm, term);
//        }
//        let len = storage.wl().entries.len();
//        if len != wlen {
//            panic!("#{}: want {}, term {}", i, wlen, len);
//        }
//    }
//}
//
//#[test]
//fn test_storage_create_snapshot() {
//    let ents = vec![new_entry(3, 3), new_entry(4, 4), new_entry(5, 5)];
//    let nodes = vec![1, 2, 3];
//    let mut cs = ConfState::new();
//    cs.set_nodes(nodes.clone());
//    let data = b"data".to_vec();
//
//    let mut tests = vec![
//        (4, Ok(new_snapshot(4, 4, nodes.clone(), data.clone()))),
//        (5, Ok(new_snapshot(5, 5, nodes.clone(), data.clone()))),
//    ];
//    for (i, (idx, wresult)) in tests.drain(..).enumerate() {
//        let storage = MemStorage::new();
//        storage.wl().entries = ents.clone();
//
//        storage
//            .wl()
//            .create_snapshot(idx, Some(cs.clone()), data.clone())
//            .expect("create snapshot failed");
//        let result = storage.snapshot();
//        if result != wresult {
//            panic!("#{}: want {:?}, got {:?}", i, wresult, result);
//        }
//    }
//}

#[test]
fn test_storage_append() {
    let ents = vec![new_entry(3, 3), new_entry(4, 4), new_entry(5, 5)];

    let mut tests = vec![
        (
            vec![new_entry(3, 3), new_entry(4, 4), new_entry(5, 5)],
            Ok(()),
            vec![new_entry(3, 3), new_entry(4, 4), new_entry(5, 5)],
        ),
        (
            vec![new_entry(3, 3), new_entry(4, 6), new_entry(5, 6)],
            Ok(()),
            vec![new_entry(3, 3), new_entry(4, 6), new_entry(5, 6)],
        ),
        (
            vec![
                new_entry(3, 3),
                new_entry(4, 4),
                new_entry(5, 5),
                new_entry(6, 5),
            ],
            Ok(()),
            vec![
                new_entry(3, 3),
                new_entry(4, 4),
                new_entry(5, 5),
                new_entry(6, 5),
            ],
        ),
        // truncate incoming entries, truncate the existing entries and append
        (
            vec![new_entry(2, 3), new_entry(3, 3), new_entry(4, 5)],
            Ok(()),
            vec![new_entry(3, 3), new_entry(4, 5)],
        ),
        // truncate the existing entries and append
        (
            vec![new_entry(4, 5)],
            Ok(()),
            vec![new_entry(3, 3), new_entry(4, 5)],
        ),
        // direct append
        (
            vec![new_entry(6, 6)],
            Ok(()),
            vec![
                new_entry(3, 3),
                new_entry(4, 4),
                new_entry(5, 5),
                new_entry(6, 6),
            ],
        ),
    ];
    for (i, (entries, wresult, wentries)) in tests.drain(..).enumerate() {
        test_storage_impls(|storage: &mut dyn StorageTestable| {
            storage.set_entries(&ents);

            let result = storage.append(&entries);
            if result != wresult {
                panic!("#{}: want {:?}, got {:?}. Storage:\n{:#?}", i, wresult, result, storage);
            }

            let e = storage.clone_entries();
            if e != wentries {
                panic!("#{}: want {:?}, entries {:?}. Storage:\n{:#?}", i, wentries, e, storage);
            }
        });
    }
}

//#[test]
//fn test_storage_apply_snapshot() {
//    let nodes = vec![1, 2, 3];
//    let data = b"data".to_vec();
//
//    let snapshots = vec![
//        new_snapshot(4, 4, nodes.clone(), data.clone()),
//        new_snapshot(3, 3, nodes.clone(), data.clone()),
//    ];
//
//    let storage = MemStorage::new();
//
//    // Apply snapshot successfully
//    let i = 0;
//    let wresult = Ok(());
//    let r = storage.wl().apply_snapshot(snapshots[i].clone());
//    if r != wresult {
//        panic!("#{}: want {:?}, got {:?}", i, wresult, r);
//    }
//
//    // Apply snapshot fails due to StorageError::SnapshotOutOfDate
//    let i = 1;
//    let wresult = Err(RaftError::Store(StorageError::SnapshotOutOfDate));
//    let r = storage.wl().apply_snapshot(snapshots[i].clone());
//    if r != wresult {
//        panic!("#{}: want {:?}, got {:?}", i, wresult, r);
//    }
//}
