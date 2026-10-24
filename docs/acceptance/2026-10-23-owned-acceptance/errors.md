# Owned acceptance errors and prevention

Observed parent incident: projected-map sourceV2 printed a successful intermediate source check, then failed its inheritance step. Existing process.run correctly returned failure and the child runner asserted. The historical Saved wall-guidance launcher ignored child exit and searched stdout for any exit=0, producing a false-positive receipt. Parent final sourceV3 and sealer separately validated explicit proof and all source/stage/native artifacts; original failure retained in projected-map/index.json.

Correction: reusable formal Lua acceptance launcher consumes the existing identity-bound Windows exit receipt. Log strings never determine success. A final summary cannot override a failed exit; optional summaries must also be fresh and successful. Exclusive evidence, journalled original identity cleanup, bounded monitoring and source-hash binding are common mechanisms. Historical Saved helpers are not rewritten. OpenSpec CLI remains absent; direct artifact workflow retained.

Pre-execution review added a sample immediately after child retirement, before exit-receipt/waiter collection, preventing a long final wait from extending the normal <=170second progress-sample gap. Cleanup failures now retain their actual diagnostic instead of an empty error when child execution itself succeeded.
