# Mail Orchestrator

Running your own email server is a deliberate choice to keep communications under your control -- no third-party provider reading your mail, no terms of service that change without notice, no account suspensions based on opaque policies. But once you have Postfix and Dovecot running, you still need to route notifications, process command emails, schedule digests, and audit delivery. This daemon handles all of that without requiring another SaaS dependency.

## The Problem

Self-hosted email operators managing multiple identities and mailboxes need automation that commercial email providers bundle in but self-hosters must build themselves: notification routing when mail arrives at a service address, scheduled email delivery (digests, reports), command processing via email (send-as-identity), and audit trails for every message. Existing tools are either tied to specific providers, require complex frameworks, or lack the routing logic needed for multi-identity, multi-domain mail servers.

## How It Works

Mail Orchestrator is a Rust daemon that watches Maildir directories for new arrivals using inotify, dispatches notifications to subscribers, processes router commands for multi-identity sending, handles scheduled email delivery via cron expressions, and logs every action to a SQLite audit trail. All configuration is a single TOML file. It is domain-agnostic -- the same binary manages any Postfix + Dovecot server regardless of domain.

```
Maildir (inotify)                 Cron Scheduler
     |                                  |
     v                                  v
  Watcher ----> Parser             Scheduler
     |            |                     |
     v            v                     v
  Notifier    Router               Sender
     |            |                     |
     v            v                     v
  email_notify    send-as           SMTP (lettre)
  create_ticket   forward               |
  shield_alert    auto-reply             v
     |                              Templates (Jinja2)
     v                                   |
  Audit DB (SQLite)  <-------------------+
```

### Modules

| Module | Purpose |
|--------|---------|
| `watcher` | inotify-based Maildir monitoring. Detects new messages in real time. |
| `parser` | RFC 5322 email parsing via mail-parser. Extracts headers, body, sender. |
| `router` | Command processing: authorized senders can send-as any configured identity. |
| `notifier` | Notification dispatch: email alerts, ticket creation, Shield integration. |
| `sender` | SMTP delivery via lettre with TLS. Handles multi-identity From addresses. |
| `scheduler` | Cron-based scheduled delivery: digests, reports, recurring emails. |
| `config` | Single TOML configuration for all domains, mailboxes, routes, and schedules. |
| `db` | SQLite audit trail: every received message, every sent notification, every action. |

**Key design decisions:**

- **Single TOML config.** One file defines all mailboxes, notification routes, authorized senders, scheduled emails, and templates. No database-driven config, no web UI for settings.
- **Domain-agnostic.** The orchestrator does not know or care what domain it serves. Change the TOML, restart the daemon, and it manages a completely different mail server.
- **inotify, not polling.** Maildir watching uses Linux inotify for instant detection of new mail. No polling interval, no missed messages, no wasted CPU.
- **Audit everything.** Every incoming message, every routed command, every sent notification, every scheduled delivery is logged to SQLite with timestamps and message IDs.
- **Jinja2 templates.** Email notifications and scheduled messages use MiniJinja templates for flexible, maintainable email content.

## Current Status

| Component | Status |
|-----------|--------|
| Maildir watcher (inotify) | Working |
| Email parser (RFC 5322) | Working |
| Router (command processing) | Working |
| Notifier (email, ticket, alert) | Working |
| Sender (SMTP via lettre) | Working |
| Scheduler (cron expressions) | Working |
| SQLite audit trail | Working |
| Template rendering | Working |
| Systemd service | Deployed |

## Quick Start

```bash
git clone https://github.com/plausiden/mail-orchestrator.git
cd mail-orchestrator
cargo build --release

# Copy and edit the config
sudo mkdir -p /etc/mail-orchestrator
sudo cp config/orchestrator.toml /etc/mail-orchestrator/orchestrator.toml
# Edit /etc/mail-orchestrator/orchestrator.toml for your domain

# Run
./target/release/mail-orchestrator --config /etc/mail-orchestrator/orchestrator.toml

# Or validate config without starting
./target/release/mail-orchestrator --config /etc/mail-orchestrator/orchestrator.toml --check
```

### Configuration Overview

```toml
[daemon]
db_path = "/var/lib/mail-orchestrator/orchestrator.db"
log_level = "info"

[domain]
name = "example.com"
mail_base = "/var/mail/vhosts/example.com"
smtp_host = "127.0.0.1"
smtp_port = 25

[router]
mailbox = "router"
maildir = "/var/mail/vhosts/example.com/router/Maildir/new/"
authorized_senders = ["admin@example.com"]
allowed_from = ["noreply@example.com", "support@example.com"]

[notify.support]
mailbox = "support@example.com"
maildir = "/var/mail/vhosts/example.com/support/Maildir/new/"
subscribers = ["admin@example.com"]
priority = "normal"
actions = ["email_notify", "create_ticket"]
```

## The PlausiDen Ecosystem

Mail Orchestrator manages email automation for Sacred.Vote's infrastructure -- routing security reports to the right people, processing command emails for multi-identity sending, and maintaining an audit trail of all platform communications. It integrates with PlausiDen Shield for alert escalation. As a standalone, domain-agnostic daemon, it is useful to anyone running a Postfix + Dovecot mail server who needs notification routing, scheduled delivery, and email audit trails without depending on external services.

## License

Apache 2.0
