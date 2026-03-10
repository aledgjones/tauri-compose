<!-- BMAD:START -->

# BMAD Method — Project Instructions

## Project Configuration

- **Project**: compose
- **User**: Aled
- **Communication Language**: English
- **Document Output Language**: English
- **User Skill Level**: intermediate
- **Output Folder**: {project-root}/\_bmad-output
- **Planning Artifacts**: {project-root}/\_bmad-output/planning-artifacts
- **Implementation Artifacts**: {project-root}/\_bmad-output/implementation-artifacts
- **Project Knowledge**: {project-root}/docs

## BMAD Runtime Structure

- **Agent definitions**: `_bmad/bmm/agents/` (BMM module) and `_bmad/core/agents/` (core)
- **Workflow definitions**: `_bmad/bmm/workflows/` (organized by phase)
- **Core tasks**: `_bmad/core/tasks/` (help, editorial review, indexing, sharding, adversarial review)
- **Core workflows**: `_bmad/core/workflows/` (brainstorming, party-mode, advanced-elicitation)
- **Workflow engine**: `_bmad/core/tasks/workflow.xml` (executes YAML-based workflows)
- **Module configuration**: `_bmad/bmm/config.yaml`
- **Core configuration**: `_bmad/core/config.yaml`
- **Agent manifest**: `_bmad/_config/agent-manifest.csv`
- **Workflow manifest**: `_bmad/_config/workflow-manifest.csv`
- **Help manifest**: `_bmad/_config/bmad-help.csv`
- **Agent memory**: `_bmad/_memory/`

## Key Conventions

- Always load `_bmad/bmm/config.yaml` before any agent activation or workflow execution
- Store all config fields as session variables: `{user_name}`, `{communication_language}`, `{output_folder}`, `{planning_artifacts}`, `{implementation_artifacts}`, `{project_knowledge}`
- MD-based workflows execute directly — load and follow the `.md` file
- YAML-based workflows require the workflow engine — load `workflow.xml` first, then pass the `.yaml` config
- Follow step-based workflow execution: load steps JIT, never multiple at once
- Save outputs after EACH step when using the workflow engine
- The `{project-root}` variable resolves to the workspace root at runtime

## Available Agents

| Agent               | Persona     | Title                                                                | Capabilities                                                                             |
| ------------------- | ----------- | -------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| bmad-master         | BMad Master | BMad Master Executor, Knowledge Custodian, and Workflow Orchestrator | runtime resource management, workflow orchestration, task execution, knowledge custodian |
| analyst             | Mary        | Business Analyst                                                     | market research, competitive analysis, requirements elicitation, domain expertise        |
| architect           | Winston     | Architect                                                            | distributed systems, cloud infrastructure, API design, scalable patterns                 |
| dev                 | Amelia      | Developer Agent                                                      | story execution, test-driven development, code implementation                            |
| pm                  | John        | Product Manager                                                      | PRD creation, requirements discovery, stakeholder alignment, user interviews             |
| qa                  | Quinn       | QA Engineer                                                          | test automation, API testing, E2E testing, coverage analysis                             |
| quick-flow-solo-dev | Barry       | Quick Flow Solo Dev                                                  | rapid spec creation, lean implementation, minimum ceremony                               |
| sm                  | Bob         | Scrum Master                                                         | sprint planning, story preparation, agile ceremonies, backlog management                 |
| tech-writer         | Paige       | Technical Writer                                                     | documentation, Mermaid diagrams, standards compliance, concept explanation               |
| ux-designer         | Sally       | UX Designer                                                          | user research, interaction design, UI patterns, experience strategy                      |

## Slash Commands

Type `/bmad-` in Copilot Chat to see all available BMAD workflows and agent activators. Agents are also available in the agents dropdown.

<!-- BMAD:END -->

## MCP

When connected to atlassian-rovo-mcp:

- **MUST** use Jira project key = SC
- **MUST** use cloudId = "https://soloapps.atlassian.net" (do NOT call getAccessibleAtlassianResources)

## Coverage Analysis

- **MUST** use `npm run test:js:coverage:summary` to display coverage totals — do NOT write ad-hoc Python or Node scripts inline for this purpose
- **MUST** use `npm run test:js:coverage` to generate/refresh the coverage data before running `test:js:coverage:summary` if the data may be stale
- These scripts are safe to auto-approve; inline coverage scripts are not

## Visual Regression Fixtures

- Fixture source files live in `tests/fixtures/source/` — plain JSON, committed to the repo
- Built `.soloc` files live in `tests/fixtures/built/` — gitignored, regenerated on demand
- **MUST** run `npm run build:fixtures` after adding or modifying any fixture JSON
- **MUST** group fixture files by concern (e.g. `time-signatures/`, `rhythms/`, `clefs/`, `key-signatures/`)
- **MUST** use flows to isolate individual test cases within a fixture file
- **MUST NOT** commit `.soloc` files — they are binary output and are gitignored
- See [tests/fixtures/README.md](../tests/fixtures/README.md) for the authoritative fixture guide
- Fixture JSON `score.meta.app_version` **MUST** match the current `CARGO_PKG_VERSION` (currently `1.23.3`) — the build script will fail with a clear error if it is missing
- When the `.soloc` container format changes, fixtures must be regenerated with `npm run build:fixtures`
- **MUST NOT** use Compound (6/8, 9/8, 12/8, …) or Complex (5/4, 7/8, …) time signatures in fixtures — only Simple time signatures (beats 1–4) are implemented; Compound and Complex will **crash the app** (`todo!()` in the renderer)

## Spec and Documentation Requirements

- **MUST** include documentation impact as part of every tech spec or implementation spec
- **MUST** identify which docs should be updated whenever a spec changes behavior, architecture, persistence, workflows, user-visible features, or developer expectations
- **MUST** include explicit documentation update tasks and acceptance criteria in specs; docs are not a follow-up nice-to-have
- **MUST** update AI-agent instructions as part of the work when project rules or required implementation practices change
- **MUST** assess migration impact for any persisted-data change, `.soloc` payload/file-format change, or load/save semantic change; if older data would not load correctly without transformation, add or update the corresponding migration
- **MUST** author unreleased persisted-data migrations in `src-tauri/src/migrations/next.rs` until a human prepares the release
- **MUST** do persisted-data work in this order: implement the change, add/update pending migration steps in `src-tauri/src/migrations/next.rs`, then have a human run `prepare-release:*` and `finalize-release` last
- **MUST NOT** run release scripts or any release/tag/publish workflow; release commands are human-only
- **MUST** ensure shipped release states contain only concrete versioned migrations; `src-tauri/src/migrations/next.rs` is development-only and must be promoted/reseeded by the human prepare/finalize flow
- **MUST** update relevant docs whenever `.soloc` file-format behavior, persistence architecture, or migration rules change
- **MUST NOT** merge persisted-data changes that break older files without the required migration update and explicit migration-impact note
