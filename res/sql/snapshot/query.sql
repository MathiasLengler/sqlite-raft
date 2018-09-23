SELECT  data, "index", term, core_id
FROM Snapshots
WHERE core_id = :core_id;