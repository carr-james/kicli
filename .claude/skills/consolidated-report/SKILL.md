---
name: consolidated-report
description: Structure of the session's consolidated report. Orchestrator use, appended per tick, delivered at every /goal stop.
---

# Consolidated report

Maintained AS YOU GO — append per tick, not at the end. At any /goal stop
(phase complete or BLOCKED), the report is what James pastes to the advisor.

## Structure

1. **Per tick, appended at the tick**: task (named by role, number in
   parentheses), lane, what landed, evidence locations (entry section,
   commits), reviewer verdict.
2. **Findings** attributed to their lane, with measurements and citations —
   not summaries of them.
3. **Reviewer rejections** and their resolutions.
4. **Dogfood defect list** when a run happened.
5. **PROPOSED items** gathered with evidence and recommendation, in entry
   order.
6. **BLOCKED items** with options and a recommendation.
7. **Workflow retrospective**:
   - which rules and prompt lines earned their keep, and which you worked
     around, ignored, or found ambiguous — name the specific line;
   - every subagent's WORKFLOW NOTE, quoted VERBATIM and attributed to its
     task — never editorialised into a summary; the raw answers are the
     data;
   - anything you had to decide about HOW to work that no doc covered.
   This section is diagnostic, not graded. "Everything was fine" is a
   suspicious answer.
