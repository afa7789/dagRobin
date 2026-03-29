# dagRobin Improvements from Claude-Coordinator

Date: 2026-03-27
Source: @DennisonBertram claude-coordinator plugin

---

## High-Impact Improvements

### 1. Structured Task Contracts
Add optional contract fields to tasks:
```bash
--contract-file <path>   # JSON file with allowed_files, forbidden_files, test_requirements
--contract-output <path>  # Expected output format
```
**Why**: Prevents workers from straying outside their scope.

---

### 2. Learning Inbox → Promotion Pipeline
Add two new commands:
```bash
dagRobin learn <task-id> "<insight>" --category practice|issue|pattern
dagRobin promote-learnings   # Review inbox, promote to docs/
```
**Why**: Captures institutional knowledge automatically, not just in your head.

**Two-stage pattern**:
- Stage 1: Inbox — append learnings to `learnings-inbox.jsonl` after each task
- Stage 2: Promote — review inbox, deduplicate, move to durable docs
  - Conventions → `docs/repo-practices.md`
  - Issues → `docs/known-issues.md`

---

### 3. Session Context Packets
Add:
```bash
dagRobin session save --message "<what was done>" --open-issues "<blockers>"
dagRobin session resume   # Print context for new agent
```
**Why**: Agents die mid-task; no way to resume intelligently today.

**Should capture**:
- Current task and progress
- Key decisions made this session
- Open blockers or pending tasks
- State needed to resume correctly

---

### 4. Review Triggers on Tasks
Add a `--risk` flag to tasks:
```bash
dagRobin add feature-x --risk security|concurrency|user-facing|high
```
**Why**: Flagging risks triggers appropriate review paths.

**Review triggers** (when to spawn reviewer):
- Security-sensitive code changes (auth, crypto, permissions)
- Concurrency or shared state changes
- User-visible changes (UI, API responses)
- Event surfaces / API contracts
- Insufficient test coverage
- Task flagged high-risk

---

### 5. Structured Worker Output
Not a code change, but a **convention**: claim should record expected output format in metadata, and `dagRobin update` should accept `--output-file <path>`.

**Required sections**:
- Scope Completed
- Files Changed
- Tests Run
- New Invariants or Assumptions
- Risks or Blockers
- Recommended Next Step

---

## Lower Priority

| Improvement | Effort |
|------------|--------|
| Intent validation phase | High (needs separate agent) |
| Minimal coordinator mode (Agent + Read only) | Medium |
| 7-phase state machine | Medium |

---

## What dagRobin Already Does Well
- Simple & Fast (Sled embedded DB)
- Conflict Prevention (claim mechanism)
- DAG-native (proper dependency tracking)
- Multi-format output (JSON, YAML, table, Mermaid, DOT)
- Import/Export

---

## Gap Analysis

| Feature | dagRobin | claude-coordinator |
|---------|----------|-------------------|
| Structured task contracts | ❌ | ✅ |
| Worker output contracts | ❌ | ✅ |
| Learning inbox → promotion | ❌ | ✅ |
| Intent validation | ❌ | ✅ |
| Session continuity (context-packet) | ❌ | ✅ |
| Conditional review triggers | ❌ | ✅ |
| 7-phase state machine | ❌ | ✅ |
| Minimal coordinator tools | N/A | ✅ |
