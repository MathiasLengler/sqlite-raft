SELECT "index", term, entry_type, data, context, sync_log
FROM Entries
WHERE "index" = :index
  AND core_id = :core_id
ORDER BY "index";
