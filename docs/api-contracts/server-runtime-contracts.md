# Cube — Server Runtime Contracts (Draft)

**Status:** DRAFT 0.1.0-draft, **Proposed text, nothing approved.** Not part of the 45 numbered contracts and not assigned to a tier; P0/P1 are untouched (AOD-18).
**Basis:** AOD-05 (separate headless Rust server), AOD-12 (no child processes), AOD-13, AOD-14, AOD-22 (one authoritative server runtime, embedded or dedicated; in-process transport for embedded, external transport for dedicated; identical semantics), `docs/architecture/system-boundaries-v1.md`.
**Not decided here:** the external transport for the dedicated server (**OAM-05 Spike Required**; TCP and QUIC are *not* chosen), the module placement and owner of the runtime (OI-8), the protocol messages.
Field order as in `worldgen-contracts.md`. No language-specific signatures.

## SR-01 ServerRuntime

* **Purpose:** the single authoritative implementation of world state, Editing service, WorldGen service with Materialiser, persistence and replication.
* **Model:** a library with a host-independent API: `Create(config) → Runtime`, `Start`, `Stop(mode)`, `AttachTransport(Transport)`, `Status`. It contains no UI, no engine types and no process calls.
* **Rules:** one runtime instance owns one world; all authority logic lives here and is identical in both hosts.

## SR-02 RuntimeHost

* **Model:** a thin shell that creates the runtime, supplies a transport and owns the lifecycle trigger. Two hosts: **EmbeddedHost** (in the client process) and **DedicatedHost** (own process, headless binary). The hosts differ **only** in how they are started, stopped and connected (AOD-22).
* **Rules:** no gameplay, authority or persistence code in a host.

## SR-03 Lifecycle

* **States:** `Created → Starting → Running → Draining → Stopping → Stopped`; `Failed` from any state.
* **Start:** load the WorldRecord, run recovery (`WP-08`), start services, accept sessions only after recovery succeeded.
* **Stop:** `Draining` refuses new sessions and finishes in-flight commits; then flushes the journal, optionally compacts, then `Stopped`. A forced stop (process kill) is a normal case: the next start recovers.
* **Embedded lifecycle:** started by the client application for a single-player world; stopped when the world is closed; follows app lifecycle events (suspend: flush the journal; resume: continue; low-memory: shed caches); its death is the client's death, so recovery must be exercised after a kill.
* **Dedicated lifecycle:** started and supervised by the operator; graceful stop on a signal; restart with recovery; sessions reconnect.
* **Tests:** the same lifecycle test suite runs on both hosts; kill-during-write tests (WP-07).

## SR-04 Authoritative state

* **Rules:** the runtime's Core asset, journal and manifest are the only authoritative world state (AOD-13). A connected client holds a cache and a prediction and is corrected (AOD-14). There is no client-authoritative mode in any host.

## SR-05 Transport abstraction

* **Purpose:** hide the carrier from the runtime.
* **Model:** a session-oriented, message-framed channel with delivery classes at protocol level (reliable ordered for operations and results; bulk or loss-tolerant for chunk and entity data, proposal). The abstraction does not expose the carrier; protocol semantics are defined once.
* **Rules:** protocol semantics, authentication, authorisation, `OperationId` handling, revision/expected-token checks and conflict rules are **identical for every transport implementation**.

## SR-06 InProcessTransport (embedded)

* **Rules:** carries the same messages over in-process channels; the embedded client passes through the **same handshake, authentication, authorisation (E-02), OperationId and revision/conflict checks**; the embedded host grants **no special rights** (the `LocalClient` origin gives no bypass) and does not become client-authoritative; a client-side call that reaches the Core commit path directly is a defect.
* **Tests:** a client cannot commit except through an authorised Editing request; the system-commit port is unreachable from the transport.

## SR-07 ExternalTransport (dedicated)

* **Rules:** carries the same messages over a network carrier. The carrier (TCP, QUIC/UDP, an external library, a control/data channel split) is **not chosen**: OAM-05 `Spike Required`. The implementation may differ from the in-process one in framing, reliability, encryption and connection handling, never in semantics (SR-05).
* **Open:** carrier, serialisation, encryption, authentication provider, mobile network behaviour.

## SR-08 Sessions and authentication

* **Rules:** every participant, including the local player of an embedded world, is a session with an `AuthorizationContext`; credentials for the embedded case are minted by the embedded host but evaluated by the same code as for remote players.
* **Open:** authentication provider and token format (Gate 7/9).

## SR-09 Conformance requirements

A shared conformance suite is mandatory and **runs on both transport implementations** (and against both hosts' lifecycle):

* protocol handshake and version negotiation; authentication failure paths;
* authorisation: denied operations change nothing; a client cannot call the system-commit port;
* `OperationId` idempotence on retry and reconnect;
* revision and expected-token behaviour, conflict results, rejected stale operations;
* replication consistency (client cache equals server state after any message loss pattern);
* persistence parity: a world saved by one host loads in the other and gives an identical content hash;
* lifecycle: start with recovery, graceful stop, forced stop, restart;
* no child processes and no test hooks (AOD-12).

A transport implementation that fails the suite is not allowed to ship.

## SR-10 Persistence ownership and shutdown

* **Rules:** the runtime owns the persistence service (`world-persistence-contracts.md`); shutdown flushes the journal before reporting `Stopped`; embedded and dedicated use the same store contracts.

## Not approved

SR-01…SR-10 are Draft. Open: transport carrier (OAM-05), runtime placement and owner, messages, authentication provider, device feasibility of the embedded runtime (memory, thermal, battery; physical device required).
