SELECT "index"
FROM Entries
WHERE core_id = :core_id
ORDER BY "index" ASC
LIMIT 1;
