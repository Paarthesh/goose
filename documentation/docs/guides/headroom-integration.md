---
sidebar_position: 20
title: Compressing Context with Headroom
sidebar_label: Headroom Integration
---

# Compressing Context with Headroom

[Headroom](https://github.com/headroomlabs-ai/headroom) is an open-source
context compression layer for AI agents. It sits between goose and the model
provider, compressing tool outputs, logs, RAG results, files, and
conversation history before they're sent to the LLM — same answers, fewer
tokens. Compression runs locally; no prompt or file content is sent anywhere
to be compressed.

Wrapping goose with Headroom is optional and off by default. It's most
useful for long-running sessions with large tool outputs (log dumps, search
results, big file reads), where it can meaningfully cut token usage and cost
without changing what the model sees in substance.

## Install

```bash
uv tool install --python 3.13 "headroom-ai[all]"  # standalone CLI
# or
pip install "headroom-ai[all]"                    # ships the `headroom` CLI
```

## Run goose wrapped

```bash
just headroom-wrap
# equivalent to:
headroom wrap goose
```

This starts a local compression proxy and launches goose configured to
route through it. Undo with:

```bash
headroom unwrap goose
```

`headroom wrap` also registers [Serena](https://github.com/oraios/serena) for
semantic code navigation at user scope, so it stays available across
projects until you `headroom unwrap`. Skip that with `headroom wrap goose --code-memory none`.

## Verify it's working

```bash
headroom doctor      # confirms routing works
headroom dashboard    # live savings while a wrapped session is running
```

## Security considerations

- Compression happens entirely on your machine — Headroom does not proxy
  your prompts or files to a third-party service.
- The proxy sits between goose and your configured model provider on
  `localhost` only by default; it does not change which provider or model
  goose talks to, only what's sent to it.
- Headroom's CCR (Cache-Compress-Retrieve) keeps original, uncompressed
  content cached locally so the model can retrieve it on demand — treat that
  local cache with the same care as any other local secrets/logs storage.
- Review [Headroom's security policy](https://github.com/headroomlabs-ai/headroom/blob/main/SECURITY.md)
  before wrapping sessions that handle sensitive data.
