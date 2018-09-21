SELECT "index", term, entry_type, data, context, sync_log
FROM Entries
WHERE "index" BETWEEN :low AND :high_inclusive
  AND core_id = :core_id
ORDER BY "index";
