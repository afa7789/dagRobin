# Learnings from Claude-Coordinator (Dennison Bertram)

## Source
Twitter/X thread by @DennisonBertram, Mar 27, 2026
https://github.com/dennisonbertram/claude-coordinator

---

## Key Concepts to Consider for dagRobin

### 1. Structured Task Contracts
- JSON-based contracts with: title, type, scope, allowed_files, forbidden_files, dependencies, test_requirements, output_contract
- Prevents scope creep, enforces boundaries
- Workers may not proceed without a contract

### 2. Structured Worker Output
Required sections:
- Scope Completed
- Files Changed
- Tests Run
- New Invariants or Assumptions
- Risks or Blockers
- Recommended Next Step

### 3. Learning System (Two-Stage Pipeline)
- **Stage 1: Inbox** — After each task, append learnings to `learning-inbox.jsonl` (JSON Lines)
- **Stage 2: Promotion** — At milestones, review inbox, deduplicate, promote to durable docs
  - Conventions → `docs/context/repo-practices.md`
  - Issues → `docs/context/known-issues.md`
  - Clear promoted entries

### 4. Intent Validation (Separate from Code Review)
- Code review: "is code correct?"
- Intent validation: "did we build what user wanted?"
- Gap analysis: scope, interpretation, assumptions, UX, completeness
- Returns: SATISFIED | NEEDS-WORK | NEEDS-DISCUSSION

### 5. Session Continuity
- Write `context-packet.md` at session end with:
  - Current milestone and progress
  - Key decisions made
  - Open blockers
  - State needed to resume
- Read at startup before anything else

### 6. Review Triggers (Conditional)
Spawn reviewer when:
- Security-sensitive code changes
- Concurrency or shared state changes
- User-visible changes
- Event surfaces / API contracts
- Insufficient test coverage
- Task flagged high-risk

### 7. 7-Phase State Machine
`intake → plan → delegate → integrate → review → promote-learnings → close`

### 8. Minimal Coordinator Tools
Stable coordinator: Agent + Read + Glob + Grep (no direct writes)
Experimental: Agent only — pure delegation to subagents

### 9. Command Intent Capture
At intake, write `docs/context/command-intent.md`:
- User's exact words
- Coordinator's interpreted intent
- Success criteria
- User's mental model
- Assumptions
- Explicitly out of scope
User confirms before work begins.

---

## What's Already Similar in dagRobin
- Explicit roles (Architect/Builder/Reviewer) map to Coordinator/Worker/Reviewer
- `MULTI_AGENT_PLAN.md` serves same purpose as `docs/plans/active-plan.md`

---

## Highest-Impact Additions to Consider
1. **Learning system** — Two-stage pipeline (inbox → promotion)
2. **Intent validation** — Separate from code review
3. **Structured task contracts** — JSON with scope/files/dependencies
4. **Session continuity** — context-packet.md pattern

---

## Source: claude-coordinator Directory Structure
```
.coord/                    # Machine operational state
├── task-ledger.json       # All tasks with status
├── learning-inbox.jsonl   # Candidate learnings
├── context-packet.md     # Session continuity
├── tasks/TASK-XXX.json   # Per-task artifacts
├── reviews/REVIEW-XXX.json
└── milestones/M-XXX.json

docs/                      # Human-readable memory
├── context/
│   ├── current-intent.md
│   ├── repo-practices.md
│   ├── known-issues.md
│   └── command-intent.md
└── plans/
    ├── active-plan.md
    └── execution-brief.md
```

## Agent Models
- opus: coordinator, reviewer
- sonnet: worker, planner, briefer
- haiku: scribe (state writer only)
