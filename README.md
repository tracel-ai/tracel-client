# Tracel Client

## Description

**Tracel Client** is a Rust crate providing the interface between external tools and the **Tracel** server.  
It contains all the HTTP calls required to communicate with the backend and acts as the single source of truth for Tracel’s API definitions.

This crate was extracted from the **Tracel SDK** to improve control over versioning and to make breaking changes explicit and easier to manage across projects.

> **Internal Note**  
> This crate is primarily intended for use by other Tracel crates (e.g., SDKs, CLIs, and internal services).  
> It is publicly visible for transparency and automation purposes but is **not meant for external integration**.

> **Important**  
> Always stay on the latest **major** release.  
> Major versions reflect breaking API changes in the backend, and older versions **will stop functioning** as server contracts evolve.

---

## Versioning

This crate follows **Semantic Versioning (SemVer)**:  
`MAJOR.MINOR.PATCH`

Although we aim to minimize breaking changes, the versioning of this crate is tightly coupled with **Tracel’s backend API**.  
Releases are driven by the server’s route definitions, not by internal library decisions.

> **Note**  
> We define a *breaking change* as a change that requires modifying how the code behaves or interacts with the API.  
> Because of the nature of this crate, many updates will be breaking in the traditional sense — this is acceptable, as the crate is not intended to abstract or stabilize backend definitions beyond their current state.  
> This is also fine because this crate is **not intended for use outside of Tracel-managed projects**.

### Major (`X.0.0`)

Major releases indicate **breaking changes** caused by backend modifications.  
These should be rare, as the backend typically introduces new route versions (e.g., `/v2/...`) to preserve compatibility.

If the data model or API contract changes fundamentally and older route definitions must be dropped, a new **major** version will be published.

### Minor (`0.X.0`)

Minor releases introduce **new functionality**, such as additional endpoints or call wrappers.  
Existing calls remain fully functional, making these releases non-breaking.

### Patch (`0.0.X`)

Patch releases include **non-breaking adjustments**, such as:
- Adding optional fields to existing request or response structures  
- Supporting new request body formats under existing routes  
- Minor consistency or type improvements  

All patch updates are **backward compatible**.

---

## Internal Guidelines

### Backend Rules

To minimize breaking changes, backend development should follow these principles:

- Avoid changing the **type** of existing fields — prefer adding a new field with the correct type.
- Avoid renaming existing fields — add new ones with the desired name instead.
- When updating request bodies, prefer **adding optional fields** rather than altering existing ones.
- When a route’s contract changes significantly, **create a new versioned route** (e.g., `/v2/route`) instead of replacing the current version.

### Crate Rules

Crate updates must adhere to the following internal standards:

- Avoid declaring response fields that are not yet used, as future type changes could introduce breaking changes.
- **Deprecate** older call wrappers when a newer route version is available, to help the SDK and CLI stay synchronized.
- Clearly document which **API version** each call targets.
- Keep changes small, explicit, and version-controlled — each modification should correspond to a backend change.

---

## Contribution Process

This crate is maintained by the **Tracel team** and should only be modified by internal contributors.  
External contributions to this crate **will not be accepted**.

If you wish to contribute to the **CLI** or **SDK**, you are more than welcome to do so.  
However, if your change requires updates to this crate, please open an issue in the [Tracel repository](https://github.com/tracel-ai/tracel) and request assistance from a team member.
