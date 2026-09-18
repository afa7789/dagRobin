# Examples

Runnable task files. Each one imports cleanly into a fresh database and is built
to demonstrate one pattern.

| File | Pattern |
|---|---|
| [`01-web-app.yaml`](./01-web-app.yaml) | A dependency chain — a feature built in order, front to back |
| [`02-parallel-round.yaml`](./02-parallel-round.yaml) | Maximum parallelism — five independent tasks converging on one integration task |
| [`03-bugfix-round.yaml`](./03-bugfix-round.yaml) | Triage — no dependencies, priority does the ordering |

## Try one in 60 seconds

```bash
# Scratch database, so you do not touch a real project
export DAGROBIN_DB=/tmp/dagrobin-demo
rm -rf "$DAGROBIN_DB"

dagRobin import examples/01-web-app.yaml
dagRobin graph
dagRobin ready                    # setup-db — the only thing not blocked
dagRobin status                   # Round: 0/5 (0%)
```

Walk the chain:

```bash
dagRobin claim setup-db -a demo
dagRobin get setup-db             # read metadata.long-description — the spec
dagRobin update setup-db --status done

dagRobin ready                    # auth-middleware unlocked
dagRobin blocked                  # api-users, api-sessions, api-tests, and why
dagRobin status                   # Round: 1/5 (20%)
```

## Try the parallel round

```bash
export DAGROBIN_DB=/tmp/dagrobin-parallel
rm -rf "$DAGROBIN_DB"

dagRobin import examples/02-parallel-round.yaml
dagRobin ready                            # five tasks, all claimable at once
dagRobin conflicts --ready-only           # "No file conflicts detected."
```

That last command is the one that matters before dispatching agents in parallel.
Prove the point by breaking it:

```bash
dagRobin add rogue "Also edits the header" --files src/components/Header.tsx
dagRobin conflicts --ready-only
# Conflict: src/components/Header.tsx
#   - rogue: "Also edits the header" (Pending)
#   - ui-header: "Site header component" (Pending)
```

Two tasks, one file. Either give `rogue` a dependency on `ui-header` or hand both
to the same agent.

## Rounds: archive between batches

```bash
export DAGROBIN_DB=/tmp/dagrobin-rounds
rm -rf "$DAGROBIN_DB"

dagRobin import examples/01-web-app.yaml
for id in setup-db auth-middleware api-users api-sessions api-tests; do
  dagRobin update "$id" --status done
done
dagRobin status                   # Round: 5/5 (100%)

dagRobin archive                  # park the finished round
dagRobin import examples/03-bugfix-round.yaml
dagRobin status                   # Round: 0/5 (0%) — All-time: 5/10 (50%)
```

The round counter tracks the batch you are actually running. The all-time line
keeps the history. Nothing was deleted — `dagRobin archive --undo --all` brings
it all back.

## Writing your own

Only `id` and `title` are required. Everything else earns its place:

```yaml
- id: kebab-case-id            # group related ones: auth-*, api-*, ui-*
  title: One line, human readable
  priority: 1                  # 1 = do first, 5 = whenever
  deps: [other-task-id]        # must be Done before this becomes ready
  files: [src/thing.rs]        # what this task WRITES — powers `conflicts`
  tags: [backend]
  metadata:
    long-description: |        # the spec — see below
      ...
```

**`metadata.long-description` is not optional in practice.** A builder subagent
has no access to the conversation that produced the task; that field is its only
source of truth. A task without one gets implemented by guessing.

A good one states: what to build, where, the exact shapes (types, status codes,
error payloads), what is explicitly out of scope, and what "done" means.

Keep a round to 15–20 tasks of 30–60 minutes each. See [AGENTS.md](../AGENTS.md)
for the agent-side protocol.
