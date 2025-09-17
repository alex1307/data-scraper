**Motto:** Raptor 2.0: From Scraping to Shaping the Web.

# Sky Net Agent

**Subtitle:** Raptor → Internet Refactoring 2.0

---

## Vision

We are not just scraping data. We are **rewriting the Internet**.
Sky Net Agent is our AI-powered platform that:

- Collects, normalizes, and indexes real-world markets (cars today, more tomorrow).
- Exposes a unified **Agent API** for natural language queries and structured search.
- Provides analytics, summaries, and insights beyond what search engines offer.

> "We won’t just break Google and the Internet… we’ll rewrite it." — the team motto.

---

## Architecture (high-level)

- **Scrapers (Raptor)** → Rust-based, config-driven, resilient.
- **Pipeline** → Kafka → Postgres (listings) → Redis (cache).
- **Agent API** → Fastify/TS service: `/query`, `/healthz`, auth, caching, metrics.
- **AI Layer** → parsing natural language → filters → SQL + scoring + summarization.

---

## Roadmap

1. **MVP:** Unified filters (Autouncle, Mobile.bg, etc.), Postgres sink, Agent API.
2. **v2:** Multi-language parser + smart scoring (price, freshness, features).
3. **v3:** Analytics & visualizations (price distributions, trends, alerts).
4. **v4:** Vector search + embeddings → personalized agent.

---

## Code Name

**Sky Net Agent** is the platform codename. Internally, we keep the battle-tested name **Raptor** for the crawler subsystem. Together:

➡️ *Raptor feeds Sky Net Agent.*

---

## Status

✅ Scrapers unified
✅ Config-driven filters
✅ Postgres + Redis foundation
🚧 Agent API under construction
🚀 Next: AI query & analytics

---

This is not just a side project. It’s **Internet Refactoring 2.0**.</file>
