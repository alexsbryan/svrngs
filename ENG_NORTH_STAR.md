svrngs is local-first event
sourcing: each group's record is an append-only log of signed events (acts),
replicated as a grow-only CRDT, and every piece of state is a projection
folded from it. Each node is a functional core of deciders and process
managers behind ports and adapters, sans-IO, and the log is written ahead of
any effect that cannot be undone. That rigidity binds the library, the source
of primitives; above it, apps and communities do what they want with them —
mechanism, not policy, and the infra holds no app's noun (`vocab_check.py`).