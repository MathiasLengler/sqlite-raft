SELECT term, vote, "commit"
FROM HardStates
JOIN Cores USING (core_id)
WHERE core_id = :core_id;