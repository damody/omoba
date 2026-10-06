# Bounded Grok patch proposal; NO TOOLS, NO FILE WRITES

Use only supplied context. No credentials/process/network/simulations/Git operations. Return minimal patch and pure tests; primary integrates/reviews/validates. Preserve current architecture/security. No runtime Lua, no weakened human receipt validation. Do not claim tests ran.

Root confirmed: omb/src/state/role_bots.rs merge_inputs maps bot player/input to `(player,input,0)` (zero correlation). build_canonical_accepted_inputs maps correlation to CanonicalAcceptedInput.input_id. In omoba-core/src/runtime/shop_receipt.rs project_shop_receipts consumes settlements by per-player FIFO, verifies command, builds receipt, calls valid. valid requires input_id !=0. Thus ordinary bot shop trades raise MalformedDisclosedState, projection partially changes team state then omits frame. Actual logs show tick15132 projection error -> MissingReplicaTick on both observers -> rebase -> Team2 enemy1903 cooldown/attack divergence. No need fabricate bot IDs or weaken codec; zero means uncorrelated authority input, no UI receipt.

Current exact loop:
for input in accepted.iter().filter(|input| input.team_id == team && matches!(input.action_kind,17|18)) {
 let Some(result)=queues.get_mut(&input.player_id).and_then(VecDeque::pop_front) else {continue;};
 if command(input).as_ref()!=Some(&result.command){return Err(ProjectionError::MalformedDisclosedState);}
 let (catalog_id,slot)=match &result.command { ShopCommand::Buy(id)=>(MOBA_ITEM_CATALOG.iter().find(|item|item.id==id).map_or(0,|item|u32::from(item.catalog_id)),0),ShopCommand::Sell(slot)=>(0,u32::try_from(*slot).map_err(|_|ProjectionError::MalformedDisclosedState)?) };
 let receipt=ShopReceipt{player_id:input.player_id,input_id:input.input_id,tick,action_kind:input.action_kind,catalog_id,slot,result_code:result.result.as_ref().err().map_or(0,|error|error.receipt_code())};
 if !receipt.valid(){return Err(ProjectionError::MalformedDisclosedState);}
 events.push(TeamPublicEvent{event_kind:FactKind::ShopReceipt as u32,stable_sub_index:events.len() as u32,sanitized_payload:receipt.encode(),..Default::default()});
}

Proposal requested: after consuming and verifying exact settlement command, skip receipt for input.input_id==0. Do NOT filter zero before pop (would misalign queues). Keep bot transaction and accepted gameplay input untouched, unchanged strict receipt codec rejects zero. Existing tests helpers `input(team,player,correlation,id)` and `result(player,id,Result<(),ShopError>)`; return new tests covering zero first followed by nonzero same-player distinct commands, zero mismatch still error, zero-only success no events, and valid human still emitted. Optional buy/sell mixed queues via helper if needed, no architecture expansion. Exact validation cargo test --manifest-path omoba-core/Cargo.toml --features compiled-content-only shop_receipt::tests -- --test-threads=1.
