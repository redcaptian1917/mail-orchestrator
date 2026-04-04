# Architecture

## Overview

Mail Orchestrator is a standalone Rust daemon that watches Maildir directories via inotify, routes notifications to subscribers, processes send-as commands, schedules recurring email delivery, and maintains a SQLite audit trail. Domain-agnostic -- one binary manages any Postfix+Dovecot server.

## System Diagram

```
+----------------------------------------------------------+
|                   mail-orchestrator                        |
|                                                           |
|  Maildir (inotify)              Cron Scheduler            |
|       |                              |                    |
|       v                              v                    |
|  +---------+    +--------+     +-----------+              |
|  | watcher |    | parser |     | scheduler |              |
|  | (notify)|    | (mail- |     | (cron     |              |
|  |         |--->| parser)|     |  crate)   |              |
|  +---------+    +--------+     +-----------+              |
|       |              |               |                    |
|       v              v               v                    |
|  +-----------+  +---------+    +----------+               |
|  | notifier  |  | router  |    | sender   |               |
|  | - email   |  | - auth  |    | (lettre) |               |
|  | - ticket  |  | - send- |    |          |               |
|  | - alert   |  |   as    |    +----------+               |
|  +-----------+  +---------+         |                     |
|       |              |              v                     |
|       |              |    +------------------+            |
|       |              |    | Templates        |            |
|       |              |    | (MiniJinja)      |            |
|       v              v    +------------------+            |
|  +--------------------------------------------+          |
|  |        db (rusqlite / SQLite)               |          |
|  |  audit log: every action with timestamp     |          |
|  +--------------------------------------------+          |
|                                                           |
|  +--------------------------------------------+          |
|  |        config (single TOML file)            |          |
|  |  domains, mailboxes, routes, schedules      |          |
|  +--------------------------------------------+          |
+----------------------------------------------------------+
```

## Data Flow

1. **Watch:** The watcher module uses Linux inotify (via the `notify` crate) to detect new files in configured Maildir `new/` directories. Zero polling, instant detection.
2. **Parse:** On detection, the parser module reads the raw email using `mail-parser` (RFC 5322). Extracts From, To, Subject, body, and custom headers.
3. **Route:** If the message arrived at the router mailbox, the router module checks the sender against the authorized list, parses the command (send-as identity, forward target), and dispatches via the sender.
4. **Notify:** If the message matches a notification route, the notifier module dispatches to all subscribers -- email alerts, ticket creation, Shield integration.
5. **Schedule:** The scheduler module evaluates cron expressions each minute. When a schedule fires, it renders a MiniJinja template and sends via SMTP through the sender module.
6. **Audit:** Every step (receive, parse, route, notify, send, schedule) is logged to SQLite with a UUID, timestamp, and message metadata.

## Key Design Decisions

- **Single TOML config.** All domains, mailboxes, routes, schedules in one file. Version-controllable, auditable, no web UI needed.
- **inotify over polling.** Instant detection, zero idle CPU. Linux-specific by design.
- **Domain-agnostic.** No hardcoded domains. Change TOML, restart, manage a different server.
- **Audit everything.** SQLite trail for every message, notification, and scheduled delivery.
- **lettre for SMTP.** Battle-tested Rust client with TLS. No custom SMTP implementation.

## Threat Model

**Defends against:** unauthorized send-as (allowlist), audit gaps (pre-execution SQLite logging), config injection (typed serde structs), template injection (MiniJinja auto-escape).

**Out of scope:** SMTP relay security (Postfix), TLS certs (Let's Encrypt), message body encryption (planned), inotify DoS (kernel-bounded).

## Future Directions

- Sequoia-PGP: automatic encryption for sensitive routes
- Shield integration: two-way alert escalation
- Prometheus metrics: delivery latency and queue depth
- Webhook notification targets alongside email
