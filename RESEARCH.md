# Valuation Research Playbook

When the user says "refresh valuations [for X]":

1. Read the dataset (`GET http://127.0.0.1:4000/api/data`, or read `data.json`).
   The server must be running to write back via the API.
2. For each target item, run multiple `research-mcp` searches using `name`,
   `details`, and `condition` (resale/sold listings, marketplaces, price guides).
   Fetch the strongest sources and record each source's figure + URL.
3. Synthesize a snapshot: `low`, `high`, `best`, a `confidence`
   (`low`/`medium`/`high`), and a one-line `comment`.
4. POST it (date = today, ISO `YYYY-MM-DD`):
   `POST /api/items/{id}/valuations` with the Valuation JSON.
   Same-day snapshots are upserted (replaced), so re-running is safe.

Rules:
- Never invent figures — every number traces to a fetched source.
- Currencies are per-item; do not convert. Match the item's currency.
- History is append-forward; never edit or delete past snapshots.
