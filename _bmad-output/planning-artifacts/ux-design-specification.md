stepsCompleted:

- "step-01-init.md"
- "step-02-discovery.md"
- "step-03-core-experience.md"
- "step-04-emotional-response.md"
- "step-05-inspiration.md"
- "step-06-design-system.md"
- "step-07-defining-experience.md"
- "step-08-visual-foundation.md"
  inputDocuments:
- "\_bmad-output/planning-artifacts/prd.md"
- "docs/project-context.md"
- "docs/architecture.md"
- "docs/project-overview.md"
- "\_bmad-output/implementation-artifacts/tech-spec-macos-native-plugin-editor-host-poc.md"

---

# UX Design Specification compose

**Author:** Aled
**Date:** 2026-03-20

---

## Executive Summary

### Project Vision

Compose should make realistic playback feel native to score writing rather than like a separate technical setup workflow. The product is not trying to become a DAW-like host with open-ended routing complexity. Instead, it should let composers stay in the notation environment while the app handles VST3-backed playback as a natural extension of the score. The defining UX promise is fast, believable, trustworthy playback with minimal friction and without forcing the user to think like an audio engineer.

The existing SCVI plugin model provides an important constraint for this work. Even though the current UI is somewhat clunky, its 1:1 plugin-instance-to-instrument mental model is simple and easy to reason about. The VST3 UX should preserve that simplicity wherever possible, even if the underlying runtime is heavier and later needs more careful resource management.

### Target Users

The primary users are composers and score authors who want realistic playback quickly enough to support active composition. They need to move from instrument creation to audible results with very little setup overhead, and they need to trust playback strongly enough to compose against it. A secondary audience includes advanced users and maintainers validating plugin compatibility, rescans, editor hosting, and transport reliability, but those needs should not distort the primary score-first workflow.

These users are likely comfortable choosing instruments and understanding musical playback concepts, but they should not need a host-engine debugging mindset. They care about whether an instrument plays, whether it sounds right, and whether they can keep moving.

### Key Design Challenges

The first challenge is making audio and plugin state legible without exposing unnecessary host complexity. Users need to understand whether instruments are routed, unrouted, missing, or unavailable, but should not need a debugging mindset to recover.

The second challenge is keeping plugin editing secondary to composition. Native VST editors already provide their own UI and should be treated as a handoff target rather than something Compose duplicates. The Compose UX should decide when and why a user opens that editor, not attempt to recreate its controls.

The third challenge is preserving user momentum through degraded states such as missing plugins, incomplete mappings, or unavailable routes. The fourth challenge is building transport trust, especially around seek, pause, resume, and repeated re-audition, so playback feels dependable rather than fragile.

The fifth challenge is future expression-map complexity. Expression maps form a bridge between score semantics and plugin playback behavior, which means they are more than simple plugin settings. They introduce a new configuration layer that must remain understandable even though it touches articulations, triggers, playback routes, and reusable mapping logic.

### Design Opportunities

Compose can create a strong UX advantage by making playback routing feel like a natural property of score instruments rather than a separate technical subsystem. Progressive disclosure can keep the common workflow simple by emphasizing ready-to-play states and surfacing deeper plugin controls only when needed.

The existing pattern of instrument cards in the instruments panel and routing dropdowns in sequencer tracks may be clunky, but it reveals a useful model: users like having a clear pool of available sound sources and a simple way to connect them to musical parts. That mental model can be preserved while improving the mechanics.

Expression maps create an opportunity to extend the product's existing score-context and app-window editing language into playback configuration. Rather than building a disconnected utility, Compose should keep assignment close to the instrument workflow while moving reusable map authoring into dedicated app-scoped windows.

Finally, the app can reinforce user trust through consistent playback-readiness and routing health signals that support confident composing without introducing a new visual language or replacing the existing design system.

## Core User Experience

### Defining Experience

The core Compose experience is preparing a believable playback route for a score instrument, writing or revising music, and hearing the result quickly and reliably enough that playback becomes part of composition rather than a separate technical task. The most important interaction to get right is the score-to-sound loop: confirm playback, edit music, and trust what comes back through playback.

The existing SCVI system reveals an important UX truth. Its one-plugin-instance-to-one-instrument model may be clunky in interface terms, but it is simple in mental-model terms. The VST3 UX should preserve that sense of directness even if the underlying implementation becomes more sophisticated. Users should feel that score instruments have playback routes into a current rack, not that they are managing a host graph or runtime topology.

### Platform Strategy

Compose remains a desktop-first, macOS-first application built around mouse and keyboard interaction, local plugin discovery, local project state, and offline-capable workflows. Native VST editors should be treated as secondary handoff surfaces for deep plugin-specific control, not embedded Compose-owned configuration systems. Compose should remain the primary workflow shell, while native plugin editors serve as focused external editing surfaces when needed.

Because VST3 instruments are heavier than SCVI instruments, the UX should avoid teaching users that every route is cheap or interchangeable. The platform strategy should therefore distinguish clearly between score-level routing, plugin-backed playback configuration, and deeper native plugin editing, while keeping that distinction mostly behind the scenes in everyday composing workflows.

### Effortless Interactions

Users should be able to scan the score or sequencer and immediately understand whether an instrument is ready, needs attention, is unrouted, or is missing on the current machine. Routing playback should feel like choosing a usable sound path for a musical part, not configuring a host architecture.

Opening a native plugin editor should feel deliberate and predictable from the right entry points, with Compose clearly handing off to the plugin’s own UI rather than attempting to recreate it. Reusing known-good playback setups should be straightforward.

Opening that editor must be a deep handoff into an already-running slot-owned plugin instance. It is not the action that initializes the score's canonical playback instance.

Future expression-map behavior should feel like configuring musical meaning rather than manipulating a low-level trigger matrix. Compose should reuse familiar panel-based editing patterns where possible, but it should distinguish clearly between selection-based properties and higher-level playback configuration. The properties panel can expose contextual information, current resolved behavior, and entry points into map editing, but expression maps themselves should be authored and managed at a higher, durable scope.

### Critical Success Moments

The first critical success moment is getting from instrument setup to believable audible playback quickly enough that the user feels momentum rather than setup cost. The second is surviving broken or missing playback resources without losing control of the score workflow. The current bounded third moment is hearing reliable one-second pitch previews while selecting or editing score tones, with broader seek, replay, and passage re-audition trust deferred to later transport work.

A future critical success moment will come from expression-map editing. The user should feel that they are teaching Compose how score instructions such as articulations and playing techniques should influence playback behavior, not building an opaque technical rules system. If this interaction becomes confusing or detached from the musical context, it will introduce disproportionate UX complexity.

### Experience Principles

Compose should preserve a simple score-first mental model even when the underlying playback engine becomes more sophisticated.

Compose should own the UX for score intent, route state, recovery, and score-semantic playback mapping, while handing plugin-specific parameter editing over to the native VST UI.

Compose should use progressive disclosure so common playback tasks stay lightweight, while more advanced configuration remains available without crowding the core workflow.

Compose should treat playback routing, deep plugin editing, and semantic playback mapping as related but distinct layers of experience.

Compose should distinguish between transient selection editing and durable playback-configuration authoring. Selection surfaces may reveal or enter playback semantics, but they should not become the primary home of reusable expression-map definitions.

Compose should reuse established interaction patterns such as panel-based editing and clear status surfaces rather than inventing a parallel design language for playback features.

Compose should make advanced playback concepts understandable in musical terms, especially for expression maps, so the user thinks about notation meaning and playback behavior rather than host internals.

## Desired Emotional Response

### Primary Emotional Goals

The primary emotional goal for Compose is confidence. Users should feel that playback is understandable, dependable, and aligned with the score-writing task rather than being a separate technical system they must manage. The secondary emotional goal is calm creative flow: the feeling that the app supports musical thinking without constantly demanding setup attention.

A key differentiating emotional goal is musical control without technical burden. Compose should not make users feel like audio engineers managing a host environment. It should make them feel like composers working with a tool that understands playback as part of the score.

### Emotional Journey Mapping

When users first encounter the playback workflow, they should feel intrigued but not intimidated. During routing and first playback, the emotional goal is relief turning into confidence: the sense that getting from score instrument to believable sound is straightforward. During active composition, the desired feeling is focused creative flow, with playback acting as a trusted partner in bounded pitch-preview auditioning while broader transport trust is still being built.

When something goes wrong, such as a missing plugin or incomplete mapping, users should still feel contained and in control rather than confused or punished. On returning to the product later, they should feel trust and orientation, not anxiety about whether playback state will still make sense.

### Micro-Emotions

The most important micro-emotions are confidence over confusion, trust over skepticism, calm over setup anxiety, competence over fragility, and satisfaction over unnecessary cleverness. These subtle feelings will determine whether playback is experienced as part of composition or as a recurring source of friction.

Confidence matters when users scan routing state and playback readiness. Trust matters when they seek, replay, and compare what they hear to what they wrote. Calm matters when the VST3 system is heavier under the hood but does not feel heavier in everyday use. Competence matters when users need to recover from missing resources or edit higher-level playback behavior such as expression maps.

### Design Implications

If Compose should make users feel confident, status and readiness surfaces must be legible at a glance. If it should make users feel calm, progressive disclosure must keep plugin and mapping complexity mostly out of the common path. If it should build trust, error and recovery states must feel guided rather than diagnostic.

Native plugin editor access should feel like entering a deliberate secondary workspace rather than leaving the product’s main flow. Expression-map editing should be framed as teaching Compose musical playback behavior, not as configuring an abstract technical matrix. Selection-based surfaces can reveal current behavior and entry points, but durable playback configuration should live at a higher scope to preserve emotional clarity.

### Emotional Design Principles

Compose should make playback feel trustworthy before it feels powerful.

Compose should reduce setup anxiety by keeping the common path simple and the system state visible.

Compose should preserve creative momentum by treating playback problems as recoverable workflow states rather than technical failures.

Compose should make advanced playback behavior feel musically meaningful, especially when users define semantic mappings such as articulation-driven playback changes.

Compose should favor calm confidence over flashy complexity, letting users feel supported, capable, and in control throughout the playback workflow.

## UX Pattern Analysis & Inspiration

### Inspiring Products Analysis

Dorico is an important positive reference because it thinks about music in a structured, musical way. Its strongest pattern is not visual style but conceptual framing: the product consistently tries to interpret musical intent and turn it into correct, useful output. That is a strong model for Compose to follow. However, its playback area is a cautionary example because complexity becomes more visible there than in other parts of the app. For Compose, the lesson is to borrow Dorico’s musical thinking while avoiding playback workflows that surface too much underlying system complexity.

Cubase is a useful boundary reference. It demonstrates the upper limit of power, control, and tweakability in playback and audio production tooling, but it also shows a level of bells-and-whistles complexity that does not fit a notation-first product. The lesson from Cubase is not that power is bad, but that default experience should not be shaped around exhaustive control surfaces. Compose should aim for convincing playback emerging from structured musical input, much like polished notation output emerges from raw score data.

Current Compose is the most important local inspiration source. Its current playback model is visually clunky in places, but it is interaction-efficient and conceptually strong. The key strength is the directness of the relationship: users can understand without much indirection that this instrument routes to this playback destination. That low-indirection clarity is worth preserving.

The design opportunity is therefore not to replace the model with a more layered or abstract workflow, but to improve the presentation and ergonomics of the same direct relationship. The weakness is primarily visual and structural polish, not the underlying user-task model. The bespoke virtual instrument format remains a separate anti-pattern because it adds product-specific complexity without providing the right kind of user-facing simplification.

### Transferable UX Patterns

The strongest transferable pattern is musical-first framing. Compose should describe playback in terms of instruments, rack slots, routes, expression behavior, and routing health rather than in terms of host internals. A second transferable pattern is progressive depth: the common path should remain shallow, while deeper plugin editing and advanced playback semantics stay behind deliberate entry points. A third pattern is the visible rack-and-route model, where users can understand the available playback resources and how they connect to musical parts.

Another valuable pattern is raw intent in, refined output out. Compose should aspire to make users provide structured musical intent while the system produces convincing playback without requiring exhaustive manual tweaking. Native VST editors can remain available for specialist work, but they should function as secondary handoff surfaces rather than as the primary place where users make the system usable.

A key transferable pattern from current Compose is direct routing without unnecessary indirection. Users benefit from being able to see and change the relationship between an instrument and its playback source in one efficient interaction. Future VST3 UX should preserve this directness while improving the visual quality, hierarchy, and composure of the control surface.

### Anti-Patterns to Avoid

Compose should avoid Dorico-style playback complexity leakage, where the user has to understand too much of the playback system in order to predict behavior. It should also avoid Cubase-style default density, where the surface area of available control begins to define the product more than the musical goal does.

Another anti-pattern is scope confusion between selection-level editing, instrument-level playback configuration, reusable expression maps, and plugin-owned parameters. If these levels blur together, the user will not know what they are editing or what will persist. Compose should also avoid exposing implementation topology as a UX model; users should not need to think about plugin-host internals, instance strategy, or resource management unless a real workflow absolutely requires it.

Compose should avoid solving a presentation problem by introducing workflow indirection. If the current direct routing model is efficient, redesign should focus first on making it clearer and more polished before replacing it with more layered interaction patterns.

### Design Inspiration Strategy

Compose should adopt Dorico’s musical semantics and current Compose’s simple mental model while deliberately avoiding both bespoke-format burden and DAW-style control sprawl. It should adapt the existing routing flow into a richer playback-configuration model that remains just as understandable in the common case. It should reuse familiar panel-based UI grammar for contextual insight and entry points, while keeping reusable expression-map authoring at a higher, durable scope.

The guiding strategy is to make playback feel like another area where Compose interprets musical intent intelligently, much like notation layout already does. Users should provide the meaningful musical input and trust Compose to produce convincing, well-formed playback without requiring them to manage a second complex application inside the notation workflow.

## Design System Foundation

### 1.1 Design System Choice

Compose should continue using its existing custom design system and interaction language, extended deliberately for playback and VST3-specific workflows. This work should not introduce a new third-party design system or a parallel visual language. Instead, the goal is to preserve the existing Compose look, feel, and product identity while refining and extending the current UI patterns where playback complexity requires more expressive controls.

### Rationale for Selection

This approach fits the project because the core UX problem is not a lack of generic components. The challenge is designing playback interactions that preserve Compose’s directness, musical framing, and calm notation-first experience. Existing Compose patterns already support much of the desired product behavior, and the current UI’s main weakness in this area is more about presentation polish than conceptual structure.

A new external design system would risk solving the wrong problem by introducing generic component aesthetics or interaction assumptions that do not match Compose’s established product language. The better path is to refine existing patterns, especially where direct routing controls and playback-related surfaces currently feel visually clunky, while preserving the underlying efficiency of the workflow.

### Implementation Approach

The implementation approach should treat playback UX as an extension of the existing product system rather than a redesign initiative. Existing panel structures, information hierarchy, state language, and interaction patterns should remain the baseline. New playback-specific UX should be built through targeted additions and refinements such as stronger routing controls, clearer status representations, more intentional chooser surfaces, and better-scoped configuration entry points.

This work may require a small number of playback-specific component variants or new UI patterns, but those additions should still feel native to Compose. The objective is to make the playback experience more composed and legible without making it feel like it belongs to a different application.

### Customization Strategy

Customization should focus on interaction and pattern refinement rather than broad visual reinvention. Compose should preserve its current styling direction and component family while improving the quality of controls that currently feel awkward in form even when they are strong in function. In particular, direct routing interactions should remain low-indirection while becoming more visually intentional and easier to scan.

Playback-related additions should follow the same design grammar as the rest of the app: familiar panels, familiar status treatment, familiar hierarchy, and familiar editing boundaries. Where new concepts such as playback configuration and expression-map authoring require new surfaces, those surfaces should be introduced as carefully scoped extensions of the existing system rather than as standalone design languages.

## 2. Core User Experience

### 2.1 Defining Experience

The defining playback experience in Compose is direct musical routing with trustworthy results. Users should be able to look at a score instrument, understand what playback route it currently uses, change that relationship directly with minimal indirection, and hear convincing output without stepping through a separate technical setup workflow.

The experience should feel like giving an instrument a clear playback path rather than configuring a host system. If this interaction is clear, fast, and dependable, it anchors the rest of the playback UX. Plugin editing, expression maps, and deeper configuration then become secondary layers built around a strong central workflow rather than compensations for a weak one.

### 2.2 User Mental Model

Users should think in a direct relationship: this instrument routes to this playback destination. They should not need to think in terms of plugin-host topology, resource allocation, or audio-production routing. Their expectation is that notation and musical intent come first, and playback should attach to that intent in a way that feels legible and predictable.

For the current MVP, the user should only need to think this instrument routes to this playback destination. If Compose later grows into explicit multi-target routing, that should be added as a bounded extension of the same direct relationship rather than as a different mode of working.

Current Compose already points toward the right mental model because its direct routing flow is efficient even when the control styling is awkward. Users are likely to expect that same immediacy to remain. Confusion is most likely to appear when scopes blur, especially between direct instrument routing, reusable playback configuration, expression-map authoring, and plugin-owned settings.

### 2.3 Success Criteria

The core interaction succeeds when users can answer three questions quickly and confidently: what is this instrument routed to, how do I change it, and is it ready to play correctly. Users should feel that routing is direct, the system state is obvious, and believable playback follows without needing a separate troubleshooting mindset.

Key success indicators include immediate readability of routing state, low-indirection rerouting, clear escalation paths into deeper configuration, and playback that feels trustworthy after the change. The interaction should feel fast enough that users stay in composition mode rather than entering setup mode.

### 2.4 Novel UX Patterns

The defining experience should primarily use established patterns rather than novel ones. Direct routing, contextual inspection, status visibility, and deliberate drill-down are all patterns users already understand. The innovation is not in inventing a new interaction grammar, but in combining those familiar patterns with a more musical mental model and cleaner scope boundaries than many playback-heavy tools provide.

Where Compose may become distinctive is in how it separates scopes: direct routing for the common case, higher-level playback configuration for durable behavior, and native plugin UI for plugin-specific depth. That combination should feel coherent rather than novel-for-the-sake-of-novel.

### 2.5 Experience Mechanics

**Initiation:** The user encounters an instrument in the score, sequencer, or related instrument surface and can immediately see its current playback state and route summary.

**Interaction:** The user changes playback through a direct routing control with minimal indirection. That control should prioritize clarity and speed over density. If needed, the user can branch into a higher-level playback configuration view or open the native plugin editor.

**Feedback:** The system clearly indicates readiness, missing resources, unrouted state, or attention-needed state. Users can tell at a glance whether their action succeeded and whether playback is ready to trust.

When playback is already running, feedback must also explain whether a supported edit applied immediately or has been deferred to the next safe playback boundary. The product should make that update rule feel predictable and musical rather than technical.

**Completion:** The interaction is complete when the user feels confident that the instrument is associated with the right playback behavior and can immediately audition that result. A successful outcome is not merely that a route changed, but that the user believes the musical playback path is coherent and dependable.

## Visual Design Foundation

### Color System

Compose should preserve its current visual language rather than introducing a new playback-specific theme. Existing Compose colors, especially the established blue highlight language and current state treatments, should remain the baseline for playback UI. Playback features should feel like native extensions of the product rather than a redesign layered on top of it.

Color should communicate state supportively, not carry the full burden of meaning. Ready, attention-needed, unavailable, and missing-resource states should use the existing Compose status language, reinforced with text labels, iconography, and clear hierarchy so the experience remains legible without depending on color alone.

### Typography System

Typography should continue following the existing Compose hierarchy and tone. Playback UX does not need a new typographic voice; it needs disciplined reuse of current heading, label, and body treatments so the new controls feel familiar inside the product's existing panel and window grammar.

The typographic emphasis should support scope clarity. Compact labels and status text should keep score-context actions scannable, while dedicated app-scoped windows such as libraries and editors can use slightly clearer sectional hierarchy to signal that the user is now working at a broader scope than the current score selection.

### Spacing & Layout Foundation

Playback surfaces should preserve the established Compose preference for compact, efficient layouts rather than moving toward spacious, card-heavy redesign patterns. Instrument rows should remain intentionally compact and information-dense. Supporting panels can expose more metadata and controls, but should still respect the current Compose density and panel structure.

Layout should express scope through containment. Score-context relationships belong in panels attached to the current working context, while app-scoped configuration and deep editing belong in separate windows. This creates a clear visual and spatial language: panels explain links and state within the score workflow; windows own reusable assets, app-level management, and object-specific deep editing.

### Accessibility Considerations

Playback state should never depend on color alone. Compact row and panel states should combine text, icons, and consistent placement so users can identify readiness and problem states quickly. Status language should stay plain and musical rather than technical wherever possible.

Because playback introduces new layers of scope, accessibility also depends on conceptual clarity. The interface should make it obvious whether the user is editing a relationship in the score, a reusable app-scoped asset, or a plugin-specific control surface. Clear window titles, section labels, and contextual headings are therefore part of accessibility, not just styling.

## 3. Detailed Workflow & Scope Model

### 3.1 Direct Routing Surfaces

The primary playback routing surface should remain close to the instrument itself. The instrument row should continue to own quick visibility and quick rerouting because routing is a core musical action rather than an advanced setup action. The row must stay compact, so it should use a concise routing control that shows current playback truth and a small readiness or health signal.

For the current MVP, the routing flow should stay single-target and explicit. Users should be able to see which playback destination the current instrument is using without being asked to manage parts, channels, or shared-slot relationships before first-audible playback is working.

The instrument panel should act as the expanded score-context surface for that same route. It should not introduce a second model. Instead, it should reveal more metadata, richer status explanation, and deeper contextual controls for the currently selected instrument and its linked playback route.

### 3.2 Playback State And Recovery

The compact user-facing playback state model should collapse into three states: Ready, Needs Attention, and Unavailable. Internal technical causes may be more numerous, but the row and compact score-context surfaces should expose only the amount of complexity users actually need in order to decide whether playback is trustworthy.

That three-state model is a presentation layer, not a replacement for deeper backend truth. Expanded panels should still distinguish the more precise causes that matter for recovery, such as unrouted state, missing plugin state, incomplete expression-map linkage, or deferred live-edit updates.

Non-ready states should use one primary contextual recovery action: open the relevant rack or route panel in the correct score context. That keeps recovery aligned with the existing selection-based panel model and avoids inventing a separate troubleshooting workflow. The row signals trust and opens context; the panel explains the issue and offers the appropriate next action.

### 3.3 Expression Maps

Expression maps should be treated as durable, reusable playback assets rather than transient properties of whichever score item happens to be selected. The Sequence instrument row is responsible for showing which route-owned Expression Map is linked to the current instrument's playback route, using the assigned map name or `--` as the bounded MVP user-facing state, and for providing a compact jump outward to map editing.

Two legitimate entry paths should exist. The first is the instrument-driven path, used when a user is diagnosing why a specific instrument is not behaving as expected and wants to assign or edit the currently linked map from the row. The second is a detached library-management path, used when the user wants to create, import, duplicate, export, or generally curate reusable Expression Maps without starting from a particular instrument.

Both paths should stay within one dedicated app-scoped Expression Map workflow window. That window should use a left library pane for search, browsing, and CRUD actions, and a right editor pane for the currently selected map. The right pane owns the map itself, while the Sequence row only owns the score-context relationship to the map.

### 3.4 Plugin Inventory And Discovery

Plugin selection during routing should stay contextual and score-focused. Instrument-row and instrument-panel pickers should present available plugins and targets as routing options for the current musical task, using a filtered and musician-friendly view of what Compose can currently use.

Full plugin discovery and organisation should live in a separate app-scoped Plugin Manager window. That window should be the canonical home for browsing the complete plugin inventory, scan health, missing or unavailable plugins, and any organisational tools such as search, filtering, hiding, or favouriting if supported. The contextual picker consumes that inventory, but does not own its management.

### 3.5 Panels Versus Windows

Panels should be treated as relational, score-context surfaces. They explain how the currently selected score object relates to playback concepts such as rack slots, playback routes, and expression maps. They are the right place for linkage, current state, and contextual actions.

Windows should be treated as object-owning or app-scoped surfaces. They are the right place for reusable libraries, plugin inventory management, expression-map editing, and other deeper configuration that exists outside the current score selection. This distinction should be visible in the UI and act as a general rule rather than a set of feature-by-feature exceptions.

### 3.6 Native Plugin UI Escalation

Compose should remain the primary surface for musical intent, playback routing, health state, expression-map linkage, and reusable playback-asset workflows. Native plugin UI should be a deliberate, secondary handoff used when the user needs to control plugin-specific internals such as vendor parameters, mixer state, patch details, or other plugin-owned configuration.

The native plugin window should not become the default answer to routine playback problems. If the user is trying to understand score semantics, mapping, or routing readiness, the answer should stay inside Compose. The native plugin UI should be available from relevant score-context surfaces, but framed as deeper plugin control rather than as a substitute for Compose-owned playback UX. Opening that window attaches to the running slot-owned instance for the selected route; it does not create the canonical playback instance.

### 3.7 Canonical App-Scoped Entry Points

The system app menu should provide canonical entry points for app-scoped playback resources such as the Expression Map workflow window and Plugin Manager. This reinforces that these surfaces are not owned by the current score selection. However, those windows should still be reachable contextually from within the score workflow so users are not forced to discover them only through the menu bar.

This creates a consistent scope model across the product: score-context relationships are discovered and managed from within the score workflow, while app-scoped inventories and deep editors are owned by dedicated windows that can be launched both from the menu bar and from contextual score surfaces.
