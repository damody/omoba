//! Owner HUD derived exclusively from disclosed state and sanitized HUD metrics.
use omoba_core::{
    game_proto::{
        AbilityHudPresentation, HeroHudPresentation, MobaHudPresentation, SummonerHudPresentation,
    },
    runtime::*,
};

/// Schema 2 introduced mana; 3 negotiated summoners; 4 current filtered attributes; 5 ability charges.
pub const MANA_SCHEMA_VERSION: u32 = 2;
pub const SUMMONER_SCHEMA_VERSION: u32 = 3;
pub const SCHEMA_VERSION: u32 = 5;
pub const SUMMONER_SHAPE_UNAVAILABLE: u32 = 0;
pub const SUMMONER_SHAPE_SELF: u32 = 1;
pub const SUMMONER_SHAPE_POINT: u32 = 2;
pub const SUMMONER_SHAPE_ENTITY: u32 = 3;

pub fn valid_objective_notice(hud:&MobaHudPresentation)->bool {
    hud.objective_notice.as_ref().is_none_or(|n|hud.phase==1 && (1..=2).contains(&n.team_id)
        && (1..=4).contains(&n.kind) && n.sequence>0 && n.sequence<=i64::MAX as u64 && (1..=4096).contains(&n.remaining_raw))
}
pub fn objective_notice(snapshot:&FilteredRenderSnapshot)->Option<omoba_core::game_proto::ObjectiveNoticePresentation> {
    let read=|id:u64| {
        let mut rows=snapshot.public_events.iter().filter_map(|event| {
            let p=&event.sanitized_payload;
            (event.event_kind==FactKind::Hud as u32 && p.len()>=12 && p[4..12]==id.to_le_bytes()).then_some(p)
        });
        let p=rows.next()?;if rows.next().is_some() || p.len()!=20 || p[..4]!=0u32.to_le_bytes(){return None;}
        Some(i64::from_le_bytes(p[12..20].try_into().unwrap()))
    };
    let n=omoba_core::game_proto::ObjectiveNoticePresentation {team_id:u32::try_from(read(OBJECTIVE_TEAM_METRIC_ID)?).ok()?,
        kind:u32::try_from(read(OBJECTIVE_KIND_METRIC_ID)?).ok()?,sequence:u64::try_from(read(OBJECTIVE_SEQUENCE_METRIC_ID)?).ok()?,
        remaining_raw:read(OBJECTIVE_REMAINING_METRIC_ID)?};
    valid_objective_notice(&MobaHudPresentation {phase:1,objective_notice:Some(n),..Default::default()}).then_some(n)
}

pub fn valid_first_kill(hud:&MobaHudPresentation)->bool {
    let Some(notice)=&hud.first_kill else{return true;};
    if hud.phase!=1 || notice.killer_player_id==0 || notice.victim_player_id==0
        || notice.killer_player_id==notice.victim_player_id || notice.bonus_gold>1_000_000{return false;}
    let Some(board)=hud.scoreboard.as_ref().filter(|b|crate::scoreboard::valid(b))else{return false;};
    match (board.rows.iter().find(|r|r.player_id==notice.killer_player_id),board.rows.iter().find(|r|r.player_id==notice.victim_player_id)){
        (Some(killer),Some(victim))=>killer.team_id!=victim.team_id,_=>false,
    }
}
pub fn first_kill_notice(snapshot:&FilteredRenderSnapshot)->Option<omoba_core::game_proto::FirstKillPresentation>{
    let read=|id:u64| {
        let mut values=snapshot.public_events.iter().filter_map(|event|{
            let p=&event.sanitized_payload;
            (event.event_kind==FactKind::Hud as u32 && p.len()==20 && p[4..12]==id.to_le_bytes()).then(||p)
        });
        let p=values.next()?;if values.next().is_some() || p[..4]!=0u32.to_le_bytes(){return None;}
        u32::try_from(i64::from_le_bytes(p[12..20].try_into().unwrap())).ok()
    };
    let notice=omoba_core::game_proto::FirstKillPresentation {killer_player_id:read(FIRST_KILL_KILLER_METRIC_ID)?,
        victim_player_id:read(FIRST_KILL_VICTIM_METRIC_ID)?,bonus_gold:read(FIRST_KILL_GOLD_METRIC_ID)?};
    let candidate=MobaHudPresentation {phase:1,first_kill:Some(notice),scoreboard:crate::scoreboard::project(snapshot),..Default::default()};
    valid_first_kill(&candidate).then_some(notice)
}

pub fn valid_cast(value:&omoba_core::game_proto::AbilityCastPresentation)->bool {
    value.instance>0 && value.instance<=i64::MAX as u64 && (1..=60*1024).contains(&value.total_raw)
        && match value.state {1=>(1..=value.total_raw).contains(&value.remaining_raw),
            2|3=>value.remaining_raw==0,_=>false}
}

pub fn effective_mana_cost(snapshot:&FilteredRenderSnapshot,player:u32,slot:usize,id:&str,rank:u32,mana:i64)->(u32,i64) {
    if slot>=4 || player==0 || rank==0 || rank>255 {return (0,0);}
    let read=|namespace| {
        let metric=single_lane_player_metric(namespace,player);
        let mut values=snapshot.public_events.iter().filter_map(|event| {
            let p=&event.sanitized_payload;
            (event.event_kind==FactKind::Hud as u32 && p.len()==20 && p[..4]==snapshot.team_id.to_le_bytes()
                && p[4..12]==metric.to_le_bytes()).then(||i64::from_le_bytes(p[12..20].try_into().unwrap()))
        });let value=values.next()?;if values.next().is_some(){None}else{Some(value)}
    };
    if read(MANA_COST_TICK).and_then(|v|u64::try_from(v).ok())!=Some(snapshot.replica_tick) {return (0,0);}
    let Some(catalog)=omoba_template_ids::ability_by_name(id)else{return (0,0);};
    if read(MANA_COST_BINDING[slot])!=Some((i64::from(catalog.raw())<<8)|i64::from(rank)){return (0,0);}
    match read(MANA_COST_RAW[slot]) {
        Some(-2)=>(3,0),Some(cost) if cost>=0 =>(if mana>=cost{1}else{2},cost),_ =>(0,0),
    }
}

/// A malformed typed price must never be converted into an affordable zero.
pub fn checked_ability_mana_cost(row:&AbilityHudPresentation,mana:Option<i64>)->(u32,i64) {
    match row.mana_cost_state {
        0|3 if row.effective_mana_cost_raw==0 => (row.mana_cost_state,0),
        1|2 if row.level>0 && row.effective_mana_cost_raw>=0
            && mana.is_some_and(|m|m>=0 && ((m>=row.effective_mana_cost_raw)==(row.mana_cost_state==1))) =>
                (row.mana_cost_state,row.effective_mana_cost_raw),
        _=>(3,0),
    }
}

pub fn cast_progress(snapshot:&FilteredRenderSnapshot,player:u32,slot:u32)
    ->Option<omoba_core::game_proto::AbilityCastPresentation> {
    if player==0 || slot as usize>=omoba_core::runtime::HERO_ABILITY_SLOTS {return None;}
    let read=|namespace| {
        let id=single_lane_player_metric(namespace,player);
        let mut values=snapshot.public_events.iter().filter_map(|event| {
            let p=&event.sanitized_payload;
            (event.event_kind==FactKind::Hud as u32 && p.len()==20
                && p[..4]==snapshot.team_id.to_le_bytes() && p[4..12]==id.to_le_bytes())
                .then(||i64::from_le_bytes(p[12..20].try_into().unwrap()))
        });
        let value=values.next()?;if values.next().is_some() {None}else{Some(value)}
    };
    if read(CAST_SLOT)?!=i64::from(slot+1) {return None;}
    let value=omoba_core::game_proto::AbilityCastPresentation {
        state:u32::try_from(read(CAST_STATE)?).ok()?,remaining_raw:read(CAST_REMAINING)?,
        total_raw:read(CAST_TOTAL)?,instance:u64::try_from(read(CAST_INSTANCE)?).ok()?,
    };
    valid_cast(&value).then_some(value)
}

/// Version 1 never carried mana. Schema 2 and 3 share the same mana rules.
pub fn valid_mana(schema: u32, hero: &HeroHudPresentation) -> bool {
    if hero.mana_supported {
        (MANA_SCHEMA_VERSION..=SCHEMA_VERSION).contains(&schema)
            && (0..=1_000_000 * 1024).contains(&hero.max_mana_raw)
            && (0..=hero.max_mana_raw).contains(&hero.mana_raw)
    } else {
        hero.mana_raw == 0 && hero.max_mana_raw == 0
    }
}

/// Schema 5 charge rows: absent (all zero) or `charges <= max <= 10` with a
/// positive per-use duration, and a running timer exactly when not full.
pub fn valid_charges(schema: u32, ability: &AbilityHudPresentation) -> bool {
    if ability.max_charges == 0 {
        return ability.charges == 0 && ability.charge_remaining_raw == 0 && ability.charge_total_raw == 0;
    }
    schema >= 5 && ability.max_charges <= 10 && ability.charges <= ability.max_charges
        && (1..=600 * 1024).contains(&ability.charge_total_raw)
        && (0..=ability.charge_total_raw).contains(&ability.charge_remaining_raw)
        && ((ability.charges == ability.max_charges) == (ability.charge_remaining_raw == 0))
}

pub fn valid_combat_attributes(schema: u32, hero: &HeroHudPresentation) -> bool {
    hero.combat_attributes.as_ref().is_none_or(|value| schema == SCHEMA_VERSION
        && omoba_core::runtime::hero_attributes::valid_attributes(value))
}

pub fn project(snapshot: &FilteredRenderSnapshot, player_id: u32) -> Option<MobaHudPresentation> {
    let metric = |team: u32, id: u64| {
        snapshot.public_events.iter().find_map(|event| {
            let p = &event.sanitized_payload;
            (event.event_kind == FactKind::Hud as u32
                && p.len() == 20
                && p[0..4] == team.to_le_bytes()
                && p[4..12] == id.to_le_bytes())
            .then(|| i64::from_le_bytes(p[12..20].try_into().unwrap()))
        })
    };
    let phase = metric(0, SINGLE_LANE_PHASE_METRIC_ID)?;
    let elapsed = metric(0, SINGLE_LANE_ELAPSED_METRIC_ID)?;
    let winner = metric(0, SINGLE_LANE_WINNER_METRIC_ID).unwrap_or(0);
    if player_id == 0 || !(0..=2).contains(&phase) || elapsed < 0 || !(0..=2).contains(&winner) {
        return None;
    }
    let hero = snapshot.entities.iter().find_map(|entity| {
        let render =
            decode_demo_render_state(entity.components.get(&DEMO_RENDER_COMPONENT_SCHEMA_ID)?)?;
        if render.kind != 1
            || render.team_id != snapshot.team_id
            || render.owner_player_id != player_id
        {
            return None;
        }
        let hero: Hero =
            serde_json::from_slice(entity.components.get(&DISCLOSED_HERO_COMPONENT_SCHEMA_ID)?)
                .ok()?;
        let p = entity
            .components
            .get(&DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID)?;
        if p.len() != 40 {
            return None;
        }
        let raw = |offset| i64::from_be_bytes(p[offset..offset + 8].try_into().unwrap());
        let abilities = hero
            .abilities
            .iter()
            .take(omoba_core::runtime::HERO_ABILITY_SLOTS)
            .enumerate()
            .map(|(slot, id)| {
                let level = hero.ability_levels.get(id).copied().unwrap_or(0).max(0) as u32;
                let total = omoba_template_ids::ability_by_name(id)
                    .and_then(omoba_template_ids::active_ability_const)
                    .and_then(|def| {
                        level
                            .checked_sub(1)
                            .and_then(|index| def.levels.get(index as usize))
                    })
                    .map_or(0, |data| data.cooldown.raw());
                let total = hero.cooldown_total(id, omoba_template_ids::Fixed64::from_raw(total))?.raw();
                let (mana_cost_state,effective_mana_cost_raw)=hero.mana_pool.as_ref().map_or((0,0),|pool|
                    effective_mana_cost(snapshot,player_id,slot,id,level,pool.current().raw()));
                Some(AbilityHudPresentation {
                    slot: slot as u32,
                    ability_id: id.clone(),
                    level,
                    cooldown_raw: hero
                        .ability_cooldowns
                        .get(id)
                        .map_or(0, |value| value.raw().max(0)),
                    total_cooldown_raw: total,
                    cast:cast_progress(snapshot,player_id,slot as u32),
                    mana_cost_state,effective_mana_cost_raw,
                    max_charges:hero.ability_charges.get(id).map_or(0,|state|u32::from(state.max)),
                    charges:hero.ability_charges.get(id).map_or(0,|state|u32::from(state.charges)),
                    charge_remaining_raw:hero.ability_charges.get(id).map_or(0,|state|state.remaining.raw().max(0)),
                    charge_total_raw:hero.ability_charges.get(id).map_or(0,|state|state.recharge.raw().max(0)),
                })
            })
            .collect::<Option<Vec<_>>>()?;
        let (mana_supported, mana_raw, max_mana_raw) = hero.mana_pool.as_ref()
            .map_or((false, 0, 0), |pool| (true, pool.current().raw(), pool.maximum().raw()));
        Some(HeroHudPresentation {
            render_id: entity.replica_id,
            disclosure_epoch: entity.disclosure_epoch,
            content_id: hero.id,
            hp_raw: raw(0),
            max_hp_raw: raw(8),
            level: hero.level.max(0) as u32,
            experience: hero.experience.max(0) as u32,
            skill_points: hero.skill_points.max(0) as u32,
            abilities,
            mana_supported,
            mana_raw,
            max_mana_raw,
            combat_attributes: snapshot.hero_attributes.get(&(entity.replica_id, entity.disclosure_epoch))
                .filter(|attributes| attributes.player_id == player_id && attributes.replica_tick == snapshot.replica_tick
                    && omoba_core::runtime::hero_attributes::valid_attributes(&attributes.values))
                .map(|attributes| attributes.values.clone()),
        })
    });
    Some(MobaHudPresentation {
        schema_version: SCHEMA_VERSION,
        player_id,
        phase: phase as u32,
        elapsed_raw: elapsed,
        winner_team: u32::try_from(winner).ok()?,
        respawn_remaining_raw: metric(snapshot.team_id, single_lane_player_metric(SINGLE_LANE_RESPAWN_METRIC_ID, player_id))
            .unwrap_or(0)
            .max(0),
        hero,
        lane_length_raw: metric(0, SINGLE_LANE_LENGTH_METRIC_ID)
            .filter(|value| (2_000 * 1024..=1_000_000 * 1024).contains(value))
            .unwrap_or(0),
        recall_remaining_raw: metric(snapshot.team_id, single_lane_player_metric(SINGLE_LANE_RECALL_REMAINING_METRIC_ID, player_id))
            .unwrap_or(0).clamp(0, i64::from(omoba_template_ids::MOBA_RECALL_CHANNEL_SECONDS) * 1024),
        recall_protocol_enabled: false,
        scoreboard: crate::scoreboard::project(snapshot),
        first_kill: if phase==1{first_kill_notice(snapshot)}else{None},
        objective_notice: if phase==1{objective_notice(snapshot)}else{None},
        score: (|| {
            Some(omoba_core::game_proto::OwnerScorePresentation {
                player_id,
                kills: u32::try_from(metric(snapshot.team_id, single_lane_player_metric(SINGLE_LANE_KILLS_METRIC_ID, player_id))?).ok()?,
                deaths: u32::try_from(metric(snapshot.team_id, single_lane_player_metric(SINGLE_LANE_DEATHS_METRIC_ID, player_id))?).ok()?,
                assists: u32::try_from(metric(snapshot.team_id, single_lane_player_metric(SINGLE_LANE_ASSISTS_METRIC_ID, player_id))?).ok()?,
            })
        })(),
        summoner_protocol_enabled: false,
        summoner_slot0: Some(closed_summoner_slot(0)),
        summoner_slot1: Some(closed_summoner_slot(1)),
    })
}

fn catalog_spell(catalog_id: u32) -> Option<&'static omoba_template_ids::SummonerSpellConst> {
    let id = u16::try_from(catalog_id).ok()?;
    omoba_template_ids::SUMMONER_SPELL_CATALOG
        .iter()
        .find(|spell| spell.catalog_id == id)
}

fn shape_and_range(spell: &omoba_template_ids::SummonerSpellConst) -> (u32, i64) {
    match spell.target {
        omoba_template_ids::SummonerTargetConst::None => (SUMMONER_SHAPE_SELF, 0),
        omoba_template_ids::SummonerTargetConst::Point => {
            let range = match spell.operation {
                omoba_template_ids::SummonerSpellOperationConst::PointBlink { range, .. } => range.raw(),
                _ => 0,
            };
            (SUMMONER_SHAPE_POINT, range)
        }
    }
}

fn closed_summoner_slot(slot: u32) -> SummonerHudPresentation {
    SummonerHudPresentation {
        slot,
        catalog_id: 0,
        shape: SUMMONER_SHAPE_UNAVAILABLE,
        range_raw: 0,
        cooldown_raw: 0,
        total_cooldown_raw: 0,
        spell_id: String::new(),
        owner_player_id: 0,
    }
}

fn empty_owner_slot(slot: u32, player_id: u32) -> SummonerHudPresentation {
    SummonerHudPresentation {
        owner_player_id: player_id,
        ..closed_summoner_slot(slot)
    }
}

fn open_row(slot: u32, player_id: u32, catalog_id: u16, cooldown_raw: i64) -> Option<SummonerHudPresentation> {
    if catalog_id == 0 {
        return (cooldown_raw == 0).then(|| empty_owner_slot(slot, player_id));
    }
    let spell = catalog_spell(u32::from(catalog_id))?;
    let (shape, range_raw) = shape_and_range(spell);
    let total = spell.cooldown.raw();
    (0..=total).contains(&cooldown_raw).then(|| SummonerHudPresentation {
        slot,
        catalog_id: u32::from(catalog_id),
        shape,
        range_raw,
        cooldown_raw,
        total_cooldown_raw: total,
        spell_id: spell.id.to_owned(),
        owner_player_id: player_id,
    })
}

fn row_closed(slot: u32, row: Option<&SummonerHudPresentation>) -> bool {
    row.is_none_or(|row| row == &closed_summoner_slot(slot))
}

fn valid_open_row(player_id: u32, slot: u32, row: &SummonerHudPresentation) -> bool {
    if row.slot != slot || row.owner_player_id != player_id || player_id == 0 || row.shape == SUMMONER_SHAPE_ENTITY {
        return false;
    }
    if row.catalog_id == 0 {
        return row.shape == SUMMONER_SHAPE_UNAVAILABLE
            && row.range_raw == 0
            && row.cooldown_raw == 0
            && row.total_cooldown_raw == 0
            && row.spell_id.is_empty();
    }
    let Some(spell) = catalog_spell(row.catalog_id) else { return false; };
    let (shape, range) = shape_and_range(spell);
    row.shape == shape
        && row.range_raw == range
        && row.total_cooldown_raw == spell.cooldown.raw()
        && row.spell_id == spell.id
        && (0..=row.total_cooldown_raw).contains(&row.cooldown_raw)
}

/// Schema below 3, or a disabled flag, accepts only canonical closed slots.
/// An enabled schema-3 row must match the authored catalog and the owner.
pub fn valid_summoner_rows(hud: &MobaHudPresentation) -> bool {
    let open = (SUMMONER_SCHEMA_VERSION..=SCHEMA_VERSION).contains(&hud.schema_version) && hud.summoner_protocol_enabled;
    if !open {
        return !hud.summoner_protocol_enabled
            && row_closed(0, hud.summoner_slot0.as_ref())
            && row_closed(1, hud.summoner_slot1.as_ref());
    }
    match (&hud.summoner_slot0, &hud.summoner_slot1) {
        (Some(slot0), Some(slot1)) => {
            valid_open_row(hud.player_id, 0, slot0) && valid_open_row(hud.player_id, 1, slot1)
        }
        _ => false,
    }
}

/// Admitted owner rows. `None` means the roster itself is not publishable.
/// A missing owner is canonical empty equipment, not another player's row.
pub fn disclosed_summoner_rows(
    snapshot: &FilteredRenderSnapshot,
    player_id: u32,
) -> Option<(SummonerHudPresentation, SummonerHudPresentation)> {
    if player_id == 0 {
        return Some((closed_summoner_slot(0), closed_summoner_slot(1)));
    }
    if !matches!(snapshot.team_id, 1 | 2) {
        return Some((empty_owner_slot(0, player_id), empty_owner_slot(1, player_id)));
    }
    let mut seen = std::collections::BTreeSet::new();
    for row in &snapshot.disclosed_summoners.players {
        if row.player_id == 0 || !seen.insert(row.player_id) {
            return None;
        }
    }
    let Some(row) = snapshot.disclosed_summoners.players.iter().find(|row| row.player_id == player_id) else {
        return Some((empty_owner_slot(0, player_id), empty_owner_slot(1, player_id)));
    };
    Some((
        open_row(0, player_id, row.slots[0].catalog_id, row.slots[0].cooldown_raw)?,
        open_row(1, player_id, row.slots[1].catalog_id, row.slots[1].cooldown_raw)?,
    ))
}

/// Capability is independent of recall and shop. A bad roster closes the rows
/// and leaves the rest of the HUD in place. Clocks are copied, never advanced.
pub fn gate_summoner_capability(hud: &mut MobaHudPresentation, snapshot: &FilteredRenderSnapshot, enabled: bool) {
    let close = |hud: &mut MobaHudPresentation| {
        hud.summoner_protocol_enabled = false;
        hud.summoner_slot0 = Some(closed_summoner_slot(0));
        hud.summoner_slot1 = Some(closed_summoner_slot(1));
    };
    if !(SUMMONER_SCHEMA_VERSION..=SCHEMA_VERSION).contains(&hud.schema_version) || !enabled {
        close(hud);
        return;
    }
    let Some((slot0, slot1)) = disclosed_summoner_rows(snapshot, hud.player_id) else {
        close(hud);
        return;
    };
    if slot0.owner_player_id == 0 || slot1.owner_player_id == 0 {
        close(hud);
        return;
    }
    hud.summoner_protocol_enabled = true;
    hud.summoner_slot0 = Some(slot0);
    hud.summoner_slot1 = Some(slot1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effective_cost_metrics_are_current_bound_unique_and_keep_free_distinct() {
        let mut s=snapshot();let id="ranger_patch";let player=7;let slot=1;
        let bind=(i64::from(omoba_template_ids::ability_by_name(id).unwrap().raw())<<8)|1;
        let team=s.team_id;
        let rows=|tick,cost|vec![metric(team,single_lane_player_metric(MANA_COST_TICK,player),tick),
            metric(team,single_lane_player_metric(MANA_COST_BINDING[slot],player),bind),
            metric(team,single_lane_player_metric(MANA_COST_RAW[slot],player),cost)];
        let tick=s.replica_tick as i64;
        for (cost,mana,state) in [(23040,30720,1),(92160,30720,2),(0,0,1),(-2,30720,3),(-1,30720,0)] {
            s.public_events=rows(tick,cost);
            assert_eq!(effective_mana_cost(&s,player,slot,id,1,mana),(state,if cost>=0{cost}else{0}));
        }
        s.public_events=rows(tick-1,23040);assert_eq!(effective_mana_cost(&s,player,slot,id,1,30720),(0,0));
        s.public_events=rows(tick,23040);assert_eq!(effective_mana_cost(&s,player,slot,id,2,30720),(0,0));
        assert_eq!(effective_mana_cost(&s,8,slot,id,1,30720),(0,0));
        assert_eq!(effective_mana_cost(&s,player,slot,"lumen_touch",1,30720),(0,0));
        s.public_events.push(s.public_events.last().unwrap().clone());
        assert_eq!(effective_mana_cost(&s,player,slot,id,1,30720),(0,0));
        s.public_events=rows(tick,23040);s.team_id+=1;
        assert_eq!(effective_mana_cost(&s,player,slot,id,1,30720),(0,0));
    }

    #[test]
    fn effective_cost_owner_hud_uses_disclosed_price_and_clears_missing_samples() {
        let mut s=snapshot();let mut owner=entity(3,1,7);
        let mut h:Hero=serde_json::from_slice(&owner.components[&DISCLOSED_HERO_COMPONENT_SCHEMA_ID]).unwrap();
        h.abilities[0]="ranger_patch".into();h.ability_levels.insert("ranger_patch".into(),1);
        h.mana_pool=Some(omoba_core::runtime::ability_runtime::ManaPool::new(Fixed64::from_i32(30),Fixed64::from_i32(100)).unwrap());
        owner.components.insert(DISCLOSED_HERO_COMPONENT_SCHEMA_ID,serde_json::to_vec(&h).unwrap());s.entities=vec![owner];
        let binding=(i64::from(omoba_template_ids::ability_by_name("ranger_patch").unwrap().raw())<<8)|1;
        for (ns,value) in [(MANA_COST_TICK,s.replica_tick as i64),(MANA_COST_BINDING[0],binding),(MANA_COST_RAW[0],23040)] {
            s.public_events.push(metric(s.team_id,single_lane_player_metric(ns,7),value));
        }
        let row=project(&s,7).unwrap().hero.unwrap().abilities.remove(0);
        assert_eq!((row.mana_cost_state,row.effective_mana_cost_raw),(1,23040));
        s.public_events.retain(|e|e.sanitized_payload[4..12]!=single_lane_player_metric(MANA_COST_RAW[0],7).to_le_bytes());
        assert_eq!(project(&s,7).unwrap().hero.unwrap().abilities[0].mana_cost_state,0);
    }

    #[test]
    fn effective_cost_absence_bad_slot_and_unlearned_do_not_become_free_prices() {
        let s=snapshot();
        for (player,slot,rank) in [(7,0,1),(0,0,1),(7,4,1),(7,0,0),(7,0,256)] {
            assert_eq!(effective_mana_cost(&s,player,slot,"ranger_patch",rank,0),(0,0));
        }
    }

    #[test]
    fn typed_effective_cost_rejects_negative_contradictory_unsupported_and_unknown_states() {
        let valid=AbilityHudPresentation {level:1,mana_cost_state:1,effective_mana_cost_raw:23040,..Default::default()};
        assert_eq!(checked_ability_mana_cost(&valid,Some(30720)),(1,23040));
        for (state,cost,level,mana) in [(1,-1,1,Some(30720)),(1,23040,1,Some(1)),(2,23040,1,Some(30720)),
            (1,23040,0,Some(30720)),(1,23040,1,None),(0,23040,1,Some(30720)),(4,0,1,Some(30720))] {
            assert_eq!(checked_ability_mana_cost(&AbilityHudPresentation {mana_cost_state:state,effective_mana_cost_raw:cost,level,..Default::default()},mana),(3,0));
        }
        assert_eq!(checked_ability_mana_cost(&AbilityHudPresentation {level:1,mana_cost_state:1,..Default::default()},Some(0)),(1,0));
    }

    #[test]
    fn owner_cooldown_hud_uses_actual_instance_total_across_roundtrip_and_tick() {
        let mut snapshot=snapshot();let mut owner=entity(3,1,7);
        let mut hero:Hero=serde_json::from_slice(&owner.components[&DISCLOSED_HERO_COMPONENT_SCHEMA_ID]).unwrap();
        for total in [10,40] {
            hero.start_cooldown("slot_0",Fixed64::from_i32(total));
            hero.tick_cooldowns(Fixed64::from_i32(total/2));
            owner.components.insert(DISCLOSED_HERO_COMPONENT_SCHEMA_ID,serde_json::to_vec(&hero).unwrap());snapshot.entities=vec![owner.clone()];
            let row=project(&snapshot,7).unwrap().hero.unwrap().abilities.remove(0);
            assert_eq!(row.total_cooldown_raw,i64::from(total)*1024);assert_eq!(row.cooldown_raw,row.total_cooldown_raw/2);
            assert!(project(&snapshot,8).unwrap().hero.is_none());
        }
    }

    #[test]
    fn owner_cooldown_hud_falls_back_for_legacy_but_rejects_corrupt_present_totals() {
        let mut snapshot=snapshot();let mut owner=entity(3,1,7);
        let mut hero:Hero=serde_json::from_slice(&owner.components[&DISCLOSED_HERO_COMPONENT_SCHEMA_ID]).unwrap();
        owner.components.insert(DISCLOSED_HERO_COMPONENT_SCHEMA_ID,serde_json::to_vec(&hero).unwrap());snapshot.entities=vec![owner.clone()];
        let row=project(&snapshot,7).unwrap().hero.unwrap().abilities.remove(0);
        assert_eq!(row.total_cooldown_raw,3072,"unknown legacy total must still contain remaining");
        for raw in [-1,1,1_024_000_001] {
            hero.ability_cooldown_totals.insert("slot_0".into(),Fixed64::from_raw(raw));
            owner.components.insert(DISCLOSED_HERO_COMPONENT_SCHEMA_ID,serde_json::to_vec(&hero).unwrap());snapshot.entities=vec![owner.clone()];
            assert!(project(&snapshot,7).unwrap().hero.is_none(),"corrupt owner cooldown cannot become a ready spell");
        }
    }
    #[test]
    fn objective_notice_is_complete_unique_public_bounded_and_roundtrips() {
        use prost::Message;
        let mut s=snapshot();
        let fields=[(OBJECTIVE_TEAM_METRIC_ID,2),(OBJECTIVE_KIND_METRIC_ID,4),(OBJECTIVE_SEQUENCE_METRIC_ID,7),(OBJECTIVE_REMAINING_METRIC_ID,4096)];
        for (id,value) in fields {s.public_events.push(metric(0,id,value));}
        let n=objective_notice(&s).unwrap();assert_eq!((n.team_id,n.kind,n.sequence,n.remaining_raw),(2,4,7,4096));
        assert_eq!(omoba_core::game_proto::ObjectiveNoticePresentation::decode(n.encode_to_vec().as_slice()).unwrap(),n);
        assert_eq!(project(&s,7).unwrap().objective_notice,Some(n));s.team_id=2;assert_eq!(project(&s,8).unwrap().objective_notice,Some(n));
        let mut typed=MobaHudPresentation {phase:1,objective_notice:Some(n),..Default::default()};
        for phase in [0,2] {typed.phase=phase;assert!(!valid_objective_notice(&typed));}typed.phase=1;
        for (index,bad) in [(0,0),(0,3),(1,0),(1,5),(2,0),(2,-1),(3,0),(3,4097),(3,-1)] {
            let mut copy=s.clone();copy.public_events[index+2]=metric(0,fields[index].0,bad);assert!(objective_notice(&copy).is_none());
        }
        for index in 0..4 {
            let mut copy=s.clone();copy.public_events.remove(index+2);assert!(objective_notice(&copy).is_none());
            let mut copy=s.clone();copy.public_events.push(copy.public_events[index+2].clone());assert!(objective_notice(&copy).is_none());
            let mut copy=s.clone();copy.public_events[index+2]=metric(1,fields[index].0,fields[index].1);assert!(objective_notice(&copy).is_none());
            let mut copy=s.clone();let mut malformed=copy.public_events[index+2].clone();malformed.sanitized_payload.pop();
            copy.public_events.push(malformed);assert!(objective_notice(&copy).is_none());
        }
        for event in &mut s.public_events {event.event_kind=FactKind::Ability as u32;}
        assert!(objective_notice(&s).is_none());
    }
    #[test]
    fn first_kill_notice_requires_complete_public_tuple_opposing_roster_and_codec() {
        use prost::Message;
        let mut s=snapshot();
        s.public_events.push(metric(0,SINGLE_LANE_SCORE_COUNT_METRIC_ID,2));
        for (player,team)in [(7,1),(8,2)]{for (ns,value)in [(SINGLE_LANE_SCORE_TEAM_METRIC_ID,team),
            (SINGLE_LANE_SCORE_KILLS_METRIC_ID,0),(SINGLE_LANE_SCORE_DEATHS_METRIC_ID,0),(SINGLE_LANE_SCORE_ASSISTS_METRIC_ID,0)]{
            s.public_events.push(metric(0,single_lane_player_metric(ns,player),value));}}
        assert!(first_kill_notice(&s).is_none());
        for (id,value)in [(FIRST_KILL_KILLER_METRIC_ID,7),(FIRST_KILL_VICTIM_METRIC_ID,8),(FIRST_KILL_GOLD_METRIC_ID,100)]{s.public_events.push(metric(0,id,value));}
        let n=first_kill_notice(&s).unwrap();assert_eq!((n.killer_player_id,n.victim_player_id,n.bonus_gold),(7,8,100));
        assert_eq!(omoba_core::game_proto::FirstKillPresentation::decode(n.encode_to_vec().as_slice()).unwrap(),n);
        assert_eq!(project(&s,7).unwrap().first_kill,Some(n));s.team_id=2;assert_eq!(project(&s,8).unwrap().first_kill,Some(n));
        let mut typed=project(&s,8).unwrap();typed.phase=2;assert!(!valid_first_kill(&typed));
        typed.phase=1;typed.first_kill.as_mut().unwrap().victim_player_id=7;assert!(!valid_first_kill(&typed));
        typed.first_kill=Some(n);typed.scoreboard=None;assert!(!valid_first_kill(&typed));
        for value in [-1,1_000_001]{*s.public_events.last_mut().unwrap()=metric(0,FIRST_KILL_GOLD_METRIC_ID,value);assert!(first_kill_notice(&s).is_none());}
        *s.public_events.last_mut().unwrap()=metric(1,FIRST_KILL_GOLD_METRIC_ID,100);assert!(first_kill_notice(&s).is_none());
        *s.public_events.last_mut().unwrap()=metric(0,FIRST_KILL_GOLD_METRIC_ID,100);s.public_events.push(s.public_events.last().unwrap().clone());assert!(first_kill_notice(&s).is_none());
        s.public_events.pop();s.public_events.pop();assert!(first_kill_notice(&s).is_none());
    }
    #[test]
    fn cast_progress_is_owner_team_scoped_complete_and_rejects_invalid_tuples() {
        let mut s=snapshot();
        let fill=|s:&mut FilteredRenderSnapshot,state,remaining| {
            s.public_events.clear();
            for (ns,value) in [(CAST_SLOT,2),(CAST_STATE,state),(CAST_REMAINING,remaining),(CAST_TOTAL,512),(CAST_INSTANCE,4)] {
                s.public_events.push(metric(1,single_lane_player_metric(ns,7),value));
            }
        };
        fill(&mut s,1,256);
        let cast=cast_progress(&s,7,1).unwrap();assert_eq!((cast.state,cast.remaining_raw,cast.total_raw,cast.instance),(1,256,512,4));
        use prost::Message;
        let mut bytes=Vec::new();cast.encode(&mut bytes).unwrap();
        assert_eq!(omoba_core::game_proto::AbilityCastPresentation::decode(bytes.as_slice()).unwrap(),cast);
        assert!(cast_progress(&s,8,1).is_none());assert!(cast_progress(&s,7,0).is_none());
        s.team_id=2;assert!(cast_progress(&s,7,1).is_none());s.team_id=1;
        for (state,remaining) in [(0,0),(4,0),(1,0),(1,513),(2,1),(3,-1)] {
            fill(&mut s,state,remaining);assert!(cast_progress(&s,7,1).is_none());
        }
        for state in [2,3] {fill(&mut s,state,0);assert_eq!(cast_progress(&s,7,1).unwrap().state,u32::try_from(state).unwrap());}
        s.public_events.pop();assert!(cast_progress(&s,7,1).is_none());
        fill(&mut s,1,256);s.public_events.push(s.public_events[0].clone());assert!(cast_progress(&s,7,1).is_none());
    }
    #[test]
    fn mana_hud_is_owner_scoped_and_distinguishes_disabled_from_empty() {
        use omoba_core::runtime::ability_runtime::ManaPool;
        let mut s = snapshot();
        let mut owner = entity(3, 1, 7);
        let mut hero: Hero = serde_json::from_slice(&owner.components[&DISCLOSED_HERO_COMPONENT_SCHEMA_ID]).unwrap();
        for pool in [None, Some(ManaPool::new(Fixed64::ZERO, Fixed64::from_i32(280)).unwrap()),
            Some(ManaPool::from_raw_state(45123, 280 * 1024, 17).unwrap())] {
            hero.mana_pool = pool.clone();
            owner.components.insert(DISCLOSED_HERO_COMPONENT_SCHEMA_ID, serde_json::to_vec(&hero).unwrap());
            s.entities = vec![entity(1, 2, 7), entity(2, 1, 8), owner.clone()];
            let hud = project(&s, 7).unwrap();
            assert_eq!(hud.schema_version, SCHEMA_VERSION);
            let projected = hud.hero.unwrap();
            assert_eq!(projected.render_id, 3);
            assert_eq!(projected.mana_supported, pool.is_some());
            assert_eq!((projected.mana_raw, projected.max_mana_raw),
                pool.as_ref().map_or((0,0), |p| (p.current().raw(), p.maximum().raw())));
            assert!(valid_mana(SCHEMA_VERSION, &projected));
            assert!(valid_mana(MANA_SCHEMA_VERSION, &projected));
        }
        s.entities.retain(|e| e.replica_id != 3);
        assert!(project(&s, 7).unwrap().hero.is_none(), "no stale dead/hidden owner mana");
    }

    #[test]
    fn mana_hud_rejects_invalid_ranges_and_legacy_managed_payloads() {
        let mut hero = HeroHudPresentation {mana_supported: true, mana_raw: 0, max_mana_raw: 280*1024, ..Default::default()};
        assert!(valid_mana(SCHEMA_VERSION, &hero));
        assert!(valid_mana(MANA_SCHEMA_VERSION, &hero));
        assert!(!valid_mana(1, &hero));
        for (current, maximum) in [(-1,280*1024),(280*1024+1,280*1024),(0,-1),(0,1_000_000*1024+1)] {
            hero.mana_raw = current; hero.max_mana_raw = maximum;
            assert!(!valid_mana(SCHEMA_VERSION, &hero));
        }
        hero.mana_supported = false; hero.mana_raw = 0; hero.max_mana_raw = 0;
        assert!(valid_mana(1, &hero));
        hero.max_mana_raw = 1;
        assert!(!valid_mana(SCHEMA_VERSION, &hero));
    }
    #[test]
    fn recall_projection_is_owner_team_only_and_legacy_capability_stays_closed() {
        let mut s = snapshot();
        s.public_events.extend([
            metric(2, single_lane_player_metric(SINGLE_LANE_RECALL_REMAINING_METRIC_ID, 7), 7 * 1024),
            metric(1, single_lane_player_metric(SINGLE_LANE_RECALL_REMAINING_METRIC_ID, 7), 3 * 1024),
            metric(1, single_lane_player_metric(SINGLE_LANE_RECALL_REMAINING_METRIC_ID, 8), 5 * 1024),
        ]);
        let hud = project(&s, 7).unwrap();
        assert_eq!(hud.recall_remaining_raw, 3 * 1024);
        assert_eq!(project(&s, 8).unwrap().recall_remaining_raw, 5 * 1024);
        assert!(!hud.recall_protocol_enabled);
        s.public_events.retain(|e| e.sanitized_payload[0..4] != 1_u32.to_le_bytes());
        assert_eq!(project(&s, 7).unwrap().recall_remaining_raw, 0);
    }
    use omoba_core::game_proto::TeamPublicEvent;
    use omoba_sim::Fixed64;
    use std::collections::BTreeMap;

    #[test]
    fn score_is_persistent_player_scoped_and_absent_or_invalid_is_not_zero() {
        let mut s = snapshot();
        assert!(project(&s,7).unwrap().score.is_none());
        for (team,player,values) in [(1,7,[2,3,4]),(1,8,[9,8,7]),(2,7,[99,99,99])] {
            for (ns,value) in [SINGLE_LANE_KILLS_METRIC_ID,SINGLE_LANE_DEATHS_METRIC_ID,SINGLE_LANE_ASSISTS_METRIC_ID].into_iter().zip(values) {
                s.public_events.push(metric(team,single_lane_player_metric(ns,player),value));
            }
        }
        let score = project(&s,7).unwrap().score.unwrap();
        assert_eq!((score.player_id,score.kills,score.deaths,score.assists),(7,2,3,4));
        assert_eq!(project(&s,8).unwrap().score.unwrap().kills,9);
        assert!(project(&s,9).unwrap().score.is_none());
        // No live entity is needed: death and a reconnect baseline retain score.
        assert!(s.entities.is_empty());
        for invalid in [-1,i64::from(u32::MAX)+1] {
            let id = single_lane_player_metric(SINGLE_LANE_KILLS_METRIC_ID,7);
            s.public_events.retain(|e| !(e.sanitized_payload[0..4]==1_u32.to_le_bytes() && e.sanitized_payload[4..12]==id.to_le_bytes()));
            s.public_events.push(metric(1,id,invalid));
            assert!(project(&s,7).unwrap().score.is_none());
        }
    }

    #[test]
    fn teammate_respawn_metrics_never_overwrite_owner_and_namespace_preserves_full_id() {
        let mut s = snapshot();
        s.public_events.extend([
            metric(1,single_lane_player_metric(SINGLE_LANE_RESPAWN_METRIC_ID,8),9000),
            metric(1,single_lane_player_metric(SINGLE_LANE_RESPAWN_METRIC_ID,7),5000),
            metric(1,single_lane_player_metric(SINGLE_LANE_RESPAWN_METRIC_ID,u32::MAX),3000),
        ]);
        assert_eq!(project(&s,7).unwrap().respawn_remaining_raw,5000);
        assert_eq!(project(&s,8).unwrap().respawn_remaining_raw,9000);
        assert_eq!(project(&s,u32::MAX).unwrap().respawn_remaining_raw,3000);
        assert_eq!(project(&s,9).unwrap().respawn_remaining_raw,0);
        for ns in [SINGLE_LANE_RESPAWN_METRIC_ID,SINGLE_LANE_RECALL_ACTIVE_METRIC_ID,SINGLE_LANE_RECALL_REMAINING_METRIC_ID] {
            assert_eq!(single_lane_metric_player(single_lane_player_metric(ns,u32::MAX),ns),Some(u32::MAX));
            assert_eq!(single_lane_metric_player(ns,ns),None);
        }
    }

    fn metric(team: u32, id: u64, value: i64) -> TeamPublicEvent {
        TeamPublicEvent {
            event_kind: FactKind::Hud as u32,
            sanitized_payload: [
                team.to_le_bytes().as_slice(),
                id.to_le_bytes().as_slice(),
                value.to_le_bytes().as_slice(),
            ]
            .concat(),
            ..Default::default()
        }
    }
    fn snapshot() -> FilteredRenderSnapshot {
        FilteredRenderSnapshot {
            team_id: 1,
            replica_tick: 99,
            entities: vec![],
            public_events: vec![
                metric(0, SINGLE_LANE_PHASE_METRIC_ID, 1),
                metric(0, SINGLE_LANE_ELAPSED_METRIC_ID, 10240),
            ],
            external_effects: vec![],
            memory_directives: vec![],
            remembered_presentations: BTreeMap::new(),
            hero_attributes: Default::default(), disclosed_summoners: Default::default(), authoritative_mode: None, combat_preferences: None,
        }
    }
    fn entity(id: u64, team: u32, player: u32) -> FilteredRenderEntity {
        let mut hero = Hero::new("training_hero".into(), "Training".into(), "".into());
        hero.abilities = (0..4).map(|slot| format!("slot_{slot}")).collect();
        for ability in &hero.abilities {
            hero.ability_levels.insert(ability.clone(), 1);
            hero.ability_cooldowns
                .insert(ability.clone(), Fixed64::from_i32(3));
        }
        let property = [
            Fixed64::from_i32(40).raw(),
            Fixed64::from_i32(100).raw(),
            0,
            0,
            0,
        ]
        .into_iter()
        .flat_map(i64::to_be_bytes)
        .collect();
        FilteredRenderEntity {
            replica_id: id,
            disclosure_epoch: 8,
            entity_kind: 1,
            components: BTreeMap::from([
                (
                    DEMO_RENDER_COMPONENT_SCHEMA_ID,
                    encode_demo_render_state(DemoRenderState {
                        x_raw: 0,
                        y_raw: 0,
                        team_id: team,
                        kind: 1,
                        owner_player_id: player,
                    }),
                ),
                (
                    DISCLOSED_HERO_COMPONENT_SCHEMA_ID,
                    serde_json::to_vec(&hero).unwrap(),
                ),
                (DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID, property),
            ]),
        }
    }
    #[test]
    fn owner_hud_preserves_unlearned_rank_and_first_learning_without_prediction() {
        let mut snapshot = snapshot();
        let mut owner = entity(3,1,7);
        let mut hero:Hero = serde_json::from_slice(&owner.components[&DISCLOSED_HERO_COMPONENT_SCHEMA_ID]).unwrap();
        for rank in [0,1] {
            hero.ability_levels.insert("slot_0".into(),rank);
            hero.skill_points = 1-rank;
            owner.components.insert(DISCLOSED_HERO_COMPONENT_SCHEMA_ID,serde_json::to_vec(&hero).unwrap());
            snapshot.entities = vec![owner.clone()];
            let hud = project(&snapshot,7).unwrap().hero.unwrap();
            assert_eq!(hud.abilities[0].level,rank as u32);
            assert_eq!(hud.skill_points,(1-rank) as u32);
            assert_eq!(hud.abilities[0].cooldown_raw,3072,"projection does not reset cooldown");
        }
    }

    #[test]
    fn owner_only_hud_ignores_enemy_and_teammate_order() {
        let mut snapshot = snapshot();
        snapshot.entities = vec![entity(1, 2, 7), entity(2, 1, 8), entity(3, 1, 7)];
        let hud = project(&snapshot, 7).unwrap();
        let hero = hud.hero.unwrap();
        assert_eq!((hero.render_id, hero.disclosure_epoch), (3, 8));
        assert_eq!((hero.hp_raw, hero.max_hp_raw), (40960, 102400));
        assert!(!hero.mana_supported);
        assert_eq!(hero.abilities.len(), 4);
        for (slot, ability) in hero.abilities.iter().enumerate() {
            assert_eq!(ability.slot, slot as u32);
            assert_eq!(ability.cooldown_raw, 3072);
        }
        assert!(project(&snapshot, 9).unwrap().hero.is_none());
    }
    #[test]
    fn death_and_finished_states_are_persistent_without_a_live_hero() {
        let mut snapshot = snapshot();
        snapshot
            .public_events
            .push(metric(1, single_lane_player_metric(SINGLE_LANE_RESPAWN_METRIC_ID, 7), 5120));
        snapshot
            .public_events
            .push(metric(2, single_lane_player_metric(SINGLE_LANE_RESPAWN_METRIC_ID, 7), 99999));
        let hud = project(&snapshot, 7).unwrap();
        assert!(hud.hero.is_none());
        assert_eq!(hud.respawn_remaining_raw, 5120);
        snapshot.public_events[0] = metric(0, SINGLE_LANE_PHASE_METRIC_ID, 2);
        snapshot
            .public_events
            .push(metric(0, SINGLE_LANE_WINNER_METRIC_ID, 2));
        assert_eq!(project(&snapshot, 7).unwrap().winner_team, 2);
        assert_eq!(project(&snapshot.clone(), 7), project(&snapshot, 7));
    }
    #[test]
    fn current_attributes_require_exact_owner_body_epoch_and_snapshot_tick() {
        use omoba_core::{game_proto::HeroCombatAttributesPresentation, runtime::hero_attributes::DisclosedHeroAttributes};
        let mut source = snapshot(); source.entities.push(entity(3,1,7));
        let values = HeroCombatAttributesPresentation { attack_damage_raw: 50*1024, physical_armor_raw: -3*1024,
            magic_resistance_raw: 256, move_speed_raw: 300*1024, attack_range_raw: 550*1024, attack_interval_raw: 512 };
        source.hero_attributes.insert((3,8),DisclosedHeroAttributes { player_id: 7, replica_tick: source.replica_tick, values: values.clone() });
        assert_eq!(project(&source,7).unwrap().hero.unwrap().combat_attributes,Some(values));
        let mut wrong = source.clone();wrong.hero_attributes.get_mut(&(3,8)).unwrap().player_id=8;
        assert!(project(&wrong,7).unwrap().hero.unwrap().combat_attributes.is_none());
        let mut wrong = source.clone();wrong.replica_tick+=1;
        assert!(project(&wrong,7).unwrap().hero.unwrap().combat_attributes.is_none());
        let mut wrong = source.clone();wrong.entities[0].disclosure_epoch+=1;
        assert!(project(&wrong,7).unwrap().hero.unwrap().combat_attributes.is_none());
        let mut wrong = source.clone();wrong.hero_attributes.get_mut(&(3,8)).unwrap().values.magic_resistance_raw=1025;
        assert!(project(&wrong,7).unwrap().hero.unwrap().combat_attributes.is_none());
        source.entities.clear();assert!(project(&source,7).unwrap().hero.is_none());
    }

    #[test]
    fn attribute_absence_and_explicit_zero_survive_proto_with_legacy_schema_rules() {
        use prost::Message;
        use omoba_core::game_proto::HeroCombatAttributesPresentation;
        let mut hero = HeroHudPresentation::default();
        for version in [1,MANA_SCHEMA_VERSION,SUMMONER_SCHEMA_VERSION,SCHEMA_VERSION] { assert!(valid_combat_attributes(version,&hero)); }
        hero.combat_attributes=Some(HeroCombatAttributesPresentation::default());
        assert!(valid_combat_attributes(SCHEMA_VERSION,&hero));
        for version in [1,MANA_SCHEMA_VERSION,SUMMONER_SCHEMA_VERSION] { assert!(!valid_combat_attributes(version,&hero)); }
        let decoded=HeroHudPresentation::decode(hero.encode_to_vec().as_slice()).unwrap();
        assert!(decoded.combat_attributes.is_some(),"zero is still an explicitly provided message");
        hero.combat_attributes.as_mut().unwrap().attack_interval_raw=-1;
        assert!(!valid_combat_attributes(SCHEMA_VERSION,&hero));
    }

    #[test]
    fn malformed_or_non_moba_control_does_not_invent_hud() {
        let mut snapshot = snapshot();
        assert!(project(&snapshot, 0).is_none());
        snapshot.entities.push(entity(3, 1, 7));
        snapshot.entities[0]
            .components
            .insert(DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID, vec![0; 39]);
        assert!(project(&snapshot, 7).unwrap().hero.is_none());
        snapshot.public_events[0].sanitized_payload.truncate(19);
        assert!(project(&snapshot, 7).is_none());
        snapshot.public_events.clear();
        assert!(project(&snapshot, 7).is_none());
    }
    #[test]
    fn static_map_length_is_public_validated_and_independent_of_units() {
        let mut s = snapshot();
        assert_eq!(project(&s, 7).unwrap().lane_length_raw, 0);
        s.public_events.push(metric(2, SINGLE_LANE_LENGTH_METRIC_ID, 9_000 * 1024));
        assert_eq!(project(&s, 7).unwrap().lane_length_raw, 0, "team-private data is not map metadata");
        s.public_events.push(metric(0, SINGLE_LANE_LENGTH_METRIC_ID, 3_200 * 1024));
        assert_eq!(project(&s, 7).unwrap().lane_length_raw, 3_200 * 1024);
        s.entities.push(entity(33, 2, 99));
        assert_eq!(project(&s, 7).unwrap().lane_length_raw, 3_200 * 1024);
        for length in [-1, 0, 1_999 * 1024, 1_000_001 * 1024, i64::MAX] {
            *s.public_events.last_mut().unwrap() = metric(0, SINGLE_LANE_LENGTH_METRIC_ID, length);
            assert_eq!(project(&s, 7).unwrap().lane_length_raw, 0);
        }
    }

    fn projected_player(player: u32, catalog: [u16; 2], cooldown: [i64; 2]) -> omoba_core::runtime::summoner_transport::ProjectedSummonerPlayer {
        use omoba_core::runtime::summoner_transport::{ProjectedSummonerPlayer, ProjectedSummonerSlot};
        ProjectedSummonerPlayer {
            player_id: player,
            slots: [0, 1].map(|index| ProjectedSummonerSlot {
                catalog_id: catalog[index],
                cooldown_raw: cooldown[index],
            }),
        }
    }

    #[test]
    fn summoner_rows_require_schema_3_and_reject_bad_catalog_clock_shape_and_owner() {
        use omoba_core::game_proto::SummonerHudPresentation;
        let blink = omoba_template_ids::summoner_spell_by_name("training_point_blink_v1").unwrap();
        let heal = omoba_template_ids::summoner_spell_by_name("training_self_heal_v1").unwrap();
        let mut snapshot = snapshot();
        let closed = project(&snapshot, 7).unwrap();
        assert_eq!(closed.schema_version, SCHEMA_VERSION);
        assert!(!closed.summoner_protocol_enabled);
        assert!(valid_summoner_rows(&closed));
        let mut legacy = closed.clone();
        legacy.schema_version = MANA_SCHEMA_VERSION;
        assert!(valid_summoner_rows(&legacy));
        legacy.summoner_protocol_enabled = true;
        assert!(!valid_summoner_rows(&legacy));
        legacy.summoner_protocol_enabled = false;
        legacy.summoner_slot0 = Some(SummonerHudPresentation { catalog_id: 1, owner_player_id: 7, ..closed_summoner_slot(0) });
        assert!(!valid_summoner_rows(&legacy));

        snapshot.disclosed_summoners.players = vec![projected_player(7, [1, 2], [0, heal.cooldown.raw()])];
        let mut hud = project(&snapshot, 7).unwrap();
        assert!(hud.hero.is_none());
        gate_summoner_capability(&mut hud, &snapshot, true);
        let again = hud.clone();
        gate_summoner_capability(&mut hud, &snapshot, true);
        assert_eq!(hud, again, "a second projection does not tick the authority clock");
        assert!(hud.summoner_protocol_enabled);
        assert!(valid_summoner_rows(&hud));
        let slot0 = hud.summoner_slot0.as_ref().unwrap();
        let slot1 = hud.summoner_slot1.as_ref().unwrap();
        assert_eq!((slot0.catalog_id, slot0.shape, slot0.spell_id.as_str()), (1, SUMMONER_SHAPE_POINT, blink.id));
        assert_eq!(slot0.range_raw, match blink.operation {
            omoba_template_ids::SummonerSpellOperationConst::PointBlink { range, .. } => range.raw(),
            _ => panic!("blink range"),
        });
        assert_eq!(slot0.total_cooldown_raw, blink.cooldown.raw());
        assert_eq!((slot1.catalog_id, slot1.shape, slot1.range_raw, slot1.cooldown_raw), (2, SUMMONER_SHAPE_SELF, 0, heal.cooldown.raw()));
        assert_eq!(slot1.owner_player_id, 7);

        let mut bad = hud.clone();
        bad.summoner_slot0.as_mut().unwrap().catalog_id = 99;
        assert!(!valid_summoner_rows(&bad));
        bad = hud.clone();
        bad.summoner_slot1.as_mut().unwrap().slot = 0;
        assert!(!valid_summoner_rows(&bad));
        bad = hud.clone();
        bad.summoner_slot1.as_mut().unwrap().cooldown_raw = -1;
        assert!(!valid_summoner_rows(&bad));
        bad = hud.clone();
        bad.summoner_slot1.as_mut().unwrap().cooldown_raw = heal.cooldown.raw() + 1;
        assert!(!valid_summoner_rows(&bad));
        bad = hud.clone();
        bad.summoner_slot0.as_mut().unwrap().shape = SUMMONER_SHAPE_ENTITY;
        assert!(!valid_summoner_rows(&bad));
        bad = hud.clone();
        bad.summoner_slot0.as_mut().unwrap().shape = SUMMONER_SHAPE_SELF;
        assert!(!valid_summoner_rows(&bad));
        bad = hud.clone();
        bad.summoner_slot0.as_mut().unwrap().owner_player_id = 8;
        assert!(!valid_summoner_rows(&bad));
        bad = hud.clone();
        bad.summoner_slot0 = Some(empty_owner_slot(0, 7));
        assert!(valid_summoner_rows(&bad), "catalog 0 is canonical unavailable");

        snapshot.disclosed_summoners.players = vec![projected_player(7, [99, 2], [0, 0])];
        let mut rejected = project(&snapshot, 7).unwrap();
        gate_summoner_capability(&mut rejected, &snapshot, true);
        assert!(!rejected.summoner_protocol_enabled);
        assert_eq!(rejected.phase, 1);
        snapshot.disclosed_summoners.players = vec![
            projected_player(7, [1, 2], [0, 0]),
            projected_player(7, [1, 0], [0, 0]),
        ];
        gate_summoner_capability(&mut rejected, &snapshot, true);
        assert!(!rejected.summoner_protocol_enabled);
        snapshot.disclosed_summoners.players = vec![projected_player(7, [0, 0], [1, 0])];
        gate_summoner_capability(&mut rejected, &snapshot, true);
        assert!(!rejected.summoner_protocol_enabled, "unequipped catalog cannot carry a clock");
    }
}

#[cfg(test)]
mod charge_hud_tests {
    use super::*;
    #[test]
    fn charge_rows_are_absent_or_consistent() {
        let none = AbilityHudPresentation::default();
        assert!(valid_charges(4, &none) && valid_charges(SCHEMA_VERSION, &none));
        let row = |max, charges, remaining, total| AbilityHudPresentation { max_charges: max, charges,
            charge_remaining_raw: remaining, charge_total_raw: total, ..Default::default() };
        assert!(valid_charges(SCHEMA_VERSION, &row(3, 3, 0, 15 * 1024)));
        assert!(valid_charges(SCHEMA_VERSION, &row(3, 1, 7 * 1024, 15 * 1024)));
        for bad in [row(3, 3, 1, 15 * 1024), row(3, 1, 0, 15 * 1024), row(3, 4, 0, 1024), row(11, 1, 1, 1024),
            row(3, 1, 16 * 1024, 15 * 1024), row(3, 1, 1, 0), row(0, 1, 0, 0)] {
            assert!(!valid_charges(SCHEMA_VERSION, &bad), "{bad:?}");
        }
        assert!(!valid_charges(4, &row(3, 3, 0, 15 * 1024)), "schema 4 never carries charges");
    }
}
