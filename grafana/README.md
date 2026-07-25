# Grafana dashboards

`dashboard.json` is the scmscx.com dashboard (uid `adldj75`). Import it with
*Dashboards -> New -> Import -> Upload JSON file*; the uid and title are baked in,
so importing updates the existing dashboard rather than creating a second copy.

It expects the Prometheus datasource uid `aek3m12l7sao0c` (chungus, see
`compose.yaml` for the scrape endpoints). If your datasource uid differs, Grafana
prompts for a replacement on import.

`dashboard_backup.json` is the previous hand-built version (7 panels), kept as a
fallback until the current one has proven itself. Delete it once that is settled.

To edit: change it in the Grafana UI, then *Dashboard settings -> JSON Model* (or
*Export -> Export for sharing externally* off) and paste the result back here so
the repo copy stays authoritative.
