# Collector capacity measurements

These committed experiments compare the prior callback collector with capacity
reservation. `pilot-*` uses a per-call method lookup above ten rows; `eager-*`
caches the reservation method once per module and still reserves above ten rows.
The final cutoff is being raised to 64 rows because the extra JNI upcall can
outweigh growth savings on short captures. Final measurements follow in this PR.

All experiments retain raw samples and complete process ranges, including slow
processes. `aa-*` compares the identical baseline artifacts. Source revisions and
artifact hashes are recorded in each JSON file and `source-identity.json`.
