---
name: "Ticket to Branch & Plan"
description: "Fetches a Jira ticket, creates a branch, and drafts a technical plan."
---

# Context

You are an expert developer. You have access to the Jira MCP server and the Local Shell.

# Task

The user will provide a Jira Ticket ID (e.g., SC-123). Follow these steps exactly:

1. **Fetch Details:** Use the `get_jira_issue` tool to retrieve the title, description, and acceptance criteria for {{ticket_id}}.
   - If the ticket cannot be found, output an error and stop.
2. **Create Branch:**
   - Sanitize and simplify the ticket title for a branch name (lowercase, replace spaces/special chars with hyphens, truncate to 50 chars).
   - Construct the branch name: `(feat|fix)/{{ticket_id}}-<sanitized-summary>`.
   - Use the `execute_command` tool to run: `git checkout -b <branch_name>`.
   - If branch creation fails, output the error and stop.
3. **Analyze & Plan:**
   - Read the relevant codebase files using `read_file`.
   - Provide a high-level **Technical Design Document**:
     - **Overview:** Brief summary of the approach.
     - **Files to Change:** List and rationale.
     - **Risks/Considerations:** Edge cases, dependencies, or pre-requisites.
     - **Acceptance Criteria:** (if available)
4. **STOP after presenting the plan.** Do not modify any code files or perform any git commits until the user explicitly says "Proceed with implementation."

# Output Format

## Branch Created

<name>

## Ticket Summary

<recap>

## Proposed Changes

- Bullet 1
- Bullet 2

## Acceptance Criteria

- (If available)

## Next Step

Would you like me to start applying these changes?
