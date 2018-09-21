SELECT term, vote, "commit"
FROM HardStates
WHERE core_id = :core_id;