# Plan

1. Preserve the sealed owned-summon-authority generation. Introduce a separate client-only SummonCommandInput with two opaque disclosure-scoped references, closed MoveTo/Release shape, and a shared atomic seal into existing authority OwnedSummonOrder. Reject canonical raw orders at the edge.
2. Bump selective protocol to14; require exact negotiated secure session/player/team/view bindings. Validate both source references as own-team and input-tick disclosed, then revalidate exact player/summoner relationship at existing formal authority.
3. Canonical accepted kind29 carries owner as actor and summon as target; redact all canonical lives. Projectors/replicas reject aliases and malformed ACKs. ACK is post-step notification only; never route it back through hero commands or private summon behavior.
4. Client host builds commands only from current own hero and own kind4 summon disclosures. Add atomic sealing, malformed/stale/foreign/leak/alias/projection/client/formal tests; consolidate maintained builds/source/staging and preserve failures, API retirement, errors and handoff. No new renderer intent/UI claim this batch.
