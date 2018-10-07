use raft::eraftpb::Entry;

pub fn entries_trim_front(entries: &[Entry], current_first_idx: u64) -> &[Entry] {
    if entries.is_empty() {
        return entries;
    }

    let append_first_idx = entries[0].index;

    if append_first_idx < current_first_idx {
        let first_tail_idx = (current_first_idx - append_first_idx) as usize;
        &entries[first_tail_idx..]
    } else {
        &entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO extract these duplicated utility functions for tests
    fn new_entry(index: u64, term: u64) -> Entry {
        let mut e = Entry::new();
        e.set_term(term);
        e.set_index(index);
        e
    }

    #[test]
    fn test_entries_trim_front() {
        let tests = vec![
            ((vec![], 0), vec![]),
            (
                (vec![new_entry(3, 3), new_entry(4, 4), new_entry(5, 5)], 2),
                vec![new_entry(3, 3), new_entry(4, 4), new_entry(5, 5)]
            ),
            (
                (vec![new_entry(3, 3), new_entry(4, 4), new_entry(5, 5)], 3),
                vec![new_entry(3, 3), new_entry(4, 4), new_entry(5, 5)]
            ),
            (
                (vec![new_entry(3, 3), new_entry(4, 4), new_entry(5, 5)], 4),
                vec![new_entry(4, 4), new_entry(5, 5)]
            ),
            (
                (vec![new_entry(3, 3), new_entry(4, 4), new_entry(5, 5)], 5),
                vec![new_entry(5, 5)]
            ),
            (
                (vec![new_entry(3, 3), new_entry(4, 4), new_entry(5, 5)], 6),
                vec![]
            ),
        ];

        for ((entries, current_first_idx), expected_truncated_entries) in tests {
            let truncated_entries = entries_trim_front(&entries, current_first_idx);

            assert_eq!(truncated_entries, expected_truncated_entries.as_slice(),
                       "current_first_idx = {}", current_first_idx);
        }
    }
}