---
name: vant-phase
description: >
  Show current phase progress, list completed and remaining tasks, and update
  PROGRESS.md. Use when the user asks about progress, says "where are we",
  "what's next", "status", or at the start of a new conversation session.
allowed-tools: Read, Edit, Glob, Grep
---

# Vant Phase Tracker

## Steps

1. **Read current progress**:
   - Read `/Users/alanguyen/Code/Others/vant/PROGRESS.md`
   - Read the plan file if it exists at `.claude/plans/` for context

2. **Summarize current state**:
   - Which phase are we in?
   - What tasks are completed (checked)?
   - What tasks remain (unchecked)?
   - What's the next action to take?

3. **Update PROGRESS.md** if the user reports completed work:
   - Check off completed tasks
   - Add new tasks discovered during work
   - Move to next phase if all tasks are done

4. **Report to user**:
   - Current phase and progress percentage
   - Next recommended action
   - Any blockers or decisions needed
