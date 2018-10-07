SELECT "index"
FROM Entries
WHERE core_id = :core_id
ORDER BY "index" DESC
LIMIT 1;
