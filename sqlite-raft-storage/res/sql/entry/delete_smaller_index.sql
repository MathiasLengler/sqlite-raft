DELETE
FROM Entries
WHERE core_id = :core_id
AND "index" < :index;