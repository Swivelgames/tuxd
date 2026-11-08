# Code organization and readability

This is shared engineering guidance for implementation and review across this
product. Apply the principles in the idioms of each language and repository.
The examples are illustrative pseudocode, not required APIs, filenames, directory
depths or a template to copy into every feature.

## Design for growth through simplicity

Scaling begins with a foundation that supports growth: clear ownership, focused
pieces and understandable connections. Resilience comes from the systems built
on that foundation and the guarantees they actually need. It does not require
every small component to start as a generalized framework.

A module should feel like a small library: it has a recognizable purpose, a
deliberate interface and implementation details that callers need not understand.
It need not become a published library, separate crate, service or project.
Keeping a module small is a successful outcome. Extracting it later is an option,
not an obligation or justification for speculative extension points today.

Ask what happens when a second and third capability arrive. Can each be added in
an obvious place with a small composition change? Or must unrelated files acquire
more branches, imports and knowledge of every capability? Improve the ownership
boundary before adding machinery to manage that accumulation.

## Organize around cohesive capabilities

At the level where capabilities differ, group the pieces that implement one
capability together. Inside that group, separate responsibilities where doing so
helps understanding. Preserve architectural layers and dependency direction;
feature grouping does not justify mixing domain policy with terminal or storage
details.

For example, an application with one action might initially look like this:

```text
application/
├── actions.rs
├── inputs.rs
├── views.rs
└── lib.rs
```

If each file collects every future action, following one feature requires finding
its fragments among unrelated features. A capability-oriented arrangement gives
those fragments a common home:

```text
application/
├── create/
│   ├── action.rs
│   ├── input.rs
│   ├── contract.rs
│   ├── view.rs
│   └── mod.rs
├── recover/
│   └── ...
└── lib.rs
```

This is an example of ownership, not a five-file minimum. A tiny capability can
start in one file and split when distinct responsibilities warrant it. Shared Git
or persistence adapters can remain in their appropriate layers; duplicating them
inside every capability would weaken separation rather than improve it.

## Five patterns for readable composition

### 1. Let the capability own its assembly

The capability knows which handler, inputs and views belong together. The
application composition root knows which capabilities are included.

```rust
// create module: assemble one capability.
fn entry() -> Result<Entry> {
	let handler = define_action("create", execute);
	let cli = cli_binding(arguments()).alias("checkout", checkout_defaults());
	let rpc = structured_binding();

	Entry::new(handler)
		.with_cli(cli, make_cli_view)
		.with_rpc(rpc)
}

// Application root: compose capabilities.
fn application() -> Result<Application> {
	let mut application = Application::new();
	application.register(create::entry()?)?;
	application.register(recover::entry()?)?;

	Ok(application)
}
```

The root should not need to know every capability's argument names, output fields
or formatting choices. Inject shared dependencies explicitly when needed; do not
replace a large composition root with a hidden global service registry.

### 2. Separate contracts from presentation

Input and outcome types describe what an operation accepts and reports. A view
decides how to present those facts for an interface. A terminal renderer should
not become the owner of types also required by RPC or other consumers.

```rust
// Contract: facts available to each interface.
struct Outcome {
	participants: Vec<ParticipantOutcome>,
	problem: Option<Problem>,
}

// Action: perform the operation and return facts.
async fn execute(input: Input, context: Context) -> Result<Outcome> {
	let request = input.normalize()?;
	let task = service.create(request, context).await?;
	Ok(Outcome::from(task))
}

// CLI view: human wording and exit policy.
fn finish(outcome: &Outcome, terminal: &mut Terminal) -> ExitCode {
	terminal.write(format_summary(outcome));
	exit_code(outcome)
}
```

Domain-to-output mapping can belong beside the contract; terminal wording belongs
with the view. Protocol envelopes belong to the relevant adapter. This permits
different presentations of the same operation without duplicating its policy.
Do not require a separate contract file for a tiny private result used by one helper.

### 3. Make functions read in meaningful stages

Named intermediate values and blank lines expose the sequence of ideas. They
reduce the number of nested constructions a reader must keep in mind.

```rust
// Dense: the reader must unpack every construction together.
register(Entry::new(action(schema(), handler()))
	.cli(binding(arguments()).alias(alias()), view_factory())
	.rpc(protocol(settings(), serializer())))?;

// Staged: each name identifies a meaningful part of the assembly.
let handler = define_action(input_schema(), execute);
let cli = cli_binding(arguments()).alias(checkout_alias());
let rpc = structured_binding();

let entry = Entry::new(handler)
	.with_cli(cli, make_cli_view)
	.with_rpc(rpc);

application.register(entry)?;
```

Not every expression needs a variable or helper. Keep a short, cohesive builder
chain together. Introduce names where they explain a concept, ownership boundary
or phase; avoid names that merely repeat syntax. Use guards to close simple cases
and keep demanding logic shallow, while preserving category distinctions.

### 4. Group by meaning within functions

A builder can be syntactically uniform while containing several different ideas.
Separate those ideas visibly, just as paragraphs separate ideas in prose.

```rust
fn arguments() -> Bindings {
	let bindings = Bindings::new().positional("task");

	// Execution options.
	let bindings = bindings
		.option("checkout")
		.option("no-fetch")
		.option("pull");

	// Workspace placement.
	let bindings = bindings
		.path_option("tasks-root")
		.path_option("state-dir");

	// Repository selection and per-repository overrides.
	bindings
		.repeated_group("repo")
		.group_option("repo-name")
		.group_option("repo-from")
}
```

Use the same approach inside loops, closures and tests: arrange, act and verify;
validate, prepare and execute; observe, classify and report. Comments should label
a useful concept or explain a reason, not narrate every obvious call. Blank lines
separate groups, not every statement. Preserve builder ordering when it affects
semantics; visual regrouping is not permission to change behavior.

### 5. Give future changes an obvious home

Adding recovery should primarily add recovery-owned code and a registration at
the composition root. It should not require adding recovery-specific conditions
to unrelated creation views or input types.

Shared behavior belongs with its actual owner. Extract a common component when
consumers share meaning, invariants and reasons to change, not merely similar
syntax. A `common` or `utils` module must not become a storage area for code whose
owner was never decided. Prefer a focused name when a shared responsibility emerges.

A useful test is to describe a future change: "change creation's CLI wording,"
"add another interface," or "add removal." A reader should be able to predict
which module owns it without first reading the entire application. This does not
mean every feature touches only one file or never changes shared contracts.

## Keep equivalent responsibilities recognizable

Read a changed capability beside its existing peers, not only in isolation.
Equivalent responsibilities should use recognizable names, assembly stages and
ownership boundaries. Otherwise maintainers must repeatedly determine whether a
structural difference expresses different behavior or just a different author.
Local clarity is necessary, but it does not establish consistency across a system.

Start by identifying the shared responsibility. Two adjacent modules need not use
the same design; two distant modules implementing the same boundary often should.
For example, create admission and existing-state recovery have different policies.
Their interface registration, event encoding and terminal output may still share
the same responsibilities. Preserve the policy differences while aligning the
parts that mean the same thing.

### Use names that describe what is assembled

A factory that returns an interface entry should not be named as though it returns
only an action. Pick one vocabulary for equivalent factories and their supporting
steps, and use it across their actual consumers. For example:

```rust
// Illustrative vocabulary: build_entry returns an Entry; action is its handler.
fn build_entry() -> Result<Entry> {
	let action = define_action("create", execute);
	let cli = CliBinding::new(cli_bindings()).alias(checkout_alias());
	let rpc = progress_binding();

	Ok(Entry::new(action)
		.cli(cli, make_view)
		.rpc(rpc))
}

fn register_actions(app: &mut Application) -> Result<()> {
	app.register(create::build_entry()?)?;
	app.register(recover::build_entry()?)?;
	Ok(())
}
```

The exact names are a local convention, not a product-wide API requirement. Keep
the same level of abstraction across peer assembly functions: build argument
bindings in a focused helper when they obscure assembly, name both interface
bindings, then attach them to the entry. Do not name one interface while burying
another equivalent interface's construction inside the final expression.

Concise registration is useful when the entry has no intermediate use. Named
intermediates remain appropriate for configuration, inspection or reuse. Choose
based on meaning; avoid alternating styles for otherwise identical registrations.

### Improve the pattern across its consumers

When a new capability exposes a clearer pattern, revisit existing implementations
of that same responsibility in the authorized scope. Do not leave the old pattern
behind simply because it was implemented earlier. Conversely, do not reproduce an
awkward pattern only to match it. Compare both, choose the clearer form, and apply
it consistently while preserving public imports, wire contracts and behavior.

If alignment would exceed scope or change an accepted contract, record the concrete
divergence and proposed follow-up. Do not silently broaden the task or describe a
partial alignment as complete. Preserve intentional user edits while doing this.

### Treat duplication as an ownership question

Repeated code is a prompt to inspect shared meaning, not an automatic demand for
abstraction. Ask whether a bug fix or policy change should apply to every copy.
If so, establish a focused owner rather than requiring maintainers to remember all
the copies. Keep distinct policy at the call sites.

```rust
// Framework mechanics: serialization and consistent encoding-failure mapping.
fn serialized_progress_binding<Output, Error, Progress>(capacity: Capacity)
	-> StructuredBinding<Output, Error, Progress>
where Progress: Serialize
{
	structured_binding(encode_progress_with_serde, capacity)
}

// Application policy: this application's queue capacity and selected event type.
fn task_progress_binding<Output, Error>()
	-> StructuredBinding<Output, Error, TaskProgress>
{
	serialized_progress_binding(TASK_PROGRESS_CAPACITY)
}
```

These are pseudocode sketches, not requests for a new framework. A helper belongs
in a reusable library when its meaning and dependencies are library-level: generic
event serialization can qualify; application task statuses and queue-size choices do
not automatically qualify. A convenience constructor should compose the existing
customization path, so callers can still supply a different mapper or view.

Prefer the smallest useful shared component. Identical rendering can share a
renderer while actions retain their own view factories. Identical DTO shapes alone
do not justify merging independently evolving public contracts. Distinct native
process implementations can expose a consistent interface without sharing their
OS-specific mechanics. Avoid a universal executor, new trait hierarchy or catch-all
helper module merely to remove a few repeated lines.

### Compare semantics before aligning syntax

Ownership can change as usage reveals a concept's real domain. A helper originally
written inside recovery may turn out to describe Git output or state damage.
Move it to that domain when appropriate; a new shared abstraction is not the only
alternative to leaving it in its original home. Shared use does not erase domain
ownership.

Consistent presentation can come from small, composable rendering components.
Each component takes a focused piece of data and renders it; the top-level view
chooses which components to call, in what order, and what the operation means.
Components should not discover state, perform operations or infer recovery policy.

```rust
// Illustrative terminal components; each renders only the supplied facts.
render_participant(output, participant)?;
render_problem(output, problem)?;
render_state_location(output, record_path)?;
```

Create and recovery can compose the same components differently. This gives future
formatting improvements one home without forcing all actions into one view or
outcome shape. Start with ordinary functions or small types; component composition
does not require a UI framework. Keep interface-specific rendering outside domain
contracts, and keep completion/exit policy at the appropriate top-level boundary.

A more complex operation may legitimately need different internal orchestration.
Recovery across several operation kinds need not resemble a simple creation flow.
Align the equivalent boundaries and mechanics; do not impose identical internals
where responsibilities differ.

Prefer named fields when a tuple makes readers remember the roles of several
similar values. Tuples remain useful for small, obvious relationships; a fixture
with several paths, resources and lifetime ownership usually benefits from a named
type. Choose based on cognitive cost rather than a universal tuple-size rule.

Near-duplicate functions deserve particular care: one may check cancellation
before publishing a phase while another publishes first. That difference may be
intentional, a bug, or an undocumented contract. Determine which before extracting
shared code. Preserve ordering of locks, observations, publications and external
effects; preserve distinctions between missing, corrupt and incompatible state.
Use regression checks appropriate to those boundaries when implementing alignment.

Review examples, fixtures and diagnostics as consumers too. Shared fixture setup
should have an obvious owner, but scenario-specific mutations and assertions should
remain visible. A generic state helper should not borrow recovery-specific remedies
simply because its error constructor is convenient to reuse.

## One primary responsibility per file

Each file should have a purpose that can be stated clearly. A filename should help
the reader find that purpose. Splitting by line count alone does not establish
cohesion; neither does putting unrelated code into small files.

In Rust, a useful default is small supporting types, followed by one primary type
and its associated implementations. Put supporting types near the top so the
reader sees the vocabulary before encountering its use. Small support-type
implementations can remain beside those types.

```rust
// Small supporting vocabulary.
enum PublishMode {
	Create,
	Replace,
}

struct PublishReceipt {
	path: PathBuf,
}

impl PublishReceipt {
	fn path(&self) -> &Path { &self.path }
}

// Primary responsibility: publishing one state document.
struct DocumentPublisher {
	directory: PathBuf,
}

impl DocumentPublisher {
	fn publish(&self, document: &Document, mode: PublishMode) -> Result<PublishReceipt> {
		// Validate, publish and return the observed result.
	}
}

impl Publisher for DocumentPublisher {
	// The same primary responsibility through its supported interface.
}
```

This is a default, not a one-struct law. A focused parser, action handler,
registration module or collection of closely related contract types may have no
primary struct. Do not invent a struct merely to comply. Several impl blocks for
one type often belong together, and tests may stay near the behavior they verify.

When supporting types or their implementations grow into a substantial concept,
move that concept to a focused submodule of its owner:

```text
publication/
├── mod.rs          # Primary publisher and narrow public surface.
├── receipt.rs      # Receipt types and their substantial behavior.
└── verification.rs # Focused publication verification.
```

Extract when the supporting concept has an independent explanation, meaningful
behavior or distracting detail. Keep it together when extraction would force the
reader through several files to understand a few inseparable lines. Keep private
internals private and expose the minimum useful interface. Moving files must not
silently break supported imports or widen visibility just to make tests compile.

## Apply the guidance during implementation and review

Read complete changed files in their final state, not only diff fragments. Examine
the directory structure and composition root as well as expressions and functions.
Formatting and passing tests cannot establish that the organization is understandable.

For each changed capability, check:

1. Can its purpose and each file's primary responsibility be stated plainly?
2. Does the capability own its assembly, with a small composition root?
3. Are contracts and operation policy separate from interface presentation?
4. Do functions reveal meaningful stages without excessive nesting or indirection?
5. Are related statements grouped within builders, loops, closures and tests?
6. Would the next related capability have an obvious home without bloating shared files?
7. Are support types easy to find, and are extracted submodules cohesive?
8. Have public interfaces, behavior and ordering survived the restructuring?
9. Have equivalent peers been compared for naming, assembly and responsibility drift?
10. If a pattern improved, were its relevant existing consumers aligned too?
11. Do repeated mechanics have one appropriate owner, with policy differences explicit?
12. Does each remaining difference express a reason a maintainer can understand?

Use these questions to find concrete improvements, not to demand a prescribed
number of files, helpers, traits or abstractions. Apply them to the authorized
change; do not turn a focused task into an unrelated repository-wide rewrite.

A creation workflow illustrates these principles:
capability-owned assembly, contracts independent of views, staged construction,
semantic argument groups and an obvious home for future actions. The lesson applies
equally to storage components, protocol handlers, UI features and other cohesive
units. The goal is code whose structure explains how to understand and extend it.

## Rust review practices in tuxd

Apply these concrete practices during implementation and review:

- Name operation modes and logical states. A pane is `Editor`, `Code` or `Tools`,
	not a numeric convention that each caller must remember. Keep numeric indices
	where they actually index an ordered collection.
- Put module declarations together, followed by exports and imports. Keep the
	capability's assembly visible without interleaving it with implementation details.
- Separate declarations and methods with blank lines. Inside operations, use
	paragraphs for preparation, validation, mutation and publication; do not split
	every statement or add comments that merely repeat the code.
- Extract complete, named operations. A parser helper consumes a sequence and
	returns its result; callers should not repair its cursor or partially apply it.
- Give shared render mechanics factual inputs, not the application. Tab labels,
	active/focused indices and bounds are enough to paint a tab group. The owner
	still decides which tabs exist and what activating one does.
- Keep domain-specific presentation with its owner. Color swatch geometry and
	contrast belong to Color; the tool panel coordinates their placement and hits.
- Review peers together: raw/escaped import, Editor/Code, color/character tabs,
	mouse/keyboard actions, and document/configuration publication. Share mechanics
	only when their invariants match. Preserve intentional differences in permissions,
	clipboard confirmation, draft activation and mutation ordering.
- Keep tests close to their contracts. A file already included with `#[cfg(test)]`
	does not need another nested test module. Import test dependencies explicitly
	when production imports move to their actual owner.

These are review criteria, not a demand to change every file or replace every
boolean with an enum. Preserve supported library interfaces, avoid unrelated
whitespace churn, and use the smallest structure that explains the responsibility.
