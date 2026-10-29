//! Catalog/time presentation; versioned optional canonical total, never arbitrary payload or source.
use omoba_template_ids::{buff_by_name, try_buff_id_str, BuffId};
pub const SCHEMA_ID: u32 = 0x464f4711;
pub const MAX_VISUAL_BUFFS: usize = 64;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuffVisualState(pub Vec<(u16, i64)>);
impl BuffVisualState {
    pub fn from_store(store: &crate::runtime::BuffStore, entity: specs::Entity) -> Self {
        let mut values = std::collections::BTreeMap::<u16, i64>::new();
        for (id, remaining) in store.iter_for(entity).filter_map(|(name, entry)| {
            let timed=crate::runtime::BuffStore::timed_modifier_visual(name,entry).ok()?;
            if is_channel_status(name) && !channel_status_valid(name,entry) {return None;}
            use crate::runtime::ability_runtime::damage_zone::{DamageZoneSpec,KEY};
            let zone=entry.payload.get(KEY).is_some();
            if (zone || name=="ground_damage_zone") && (DamageZoneSpec::from_payload(&entry.payload).is_none()
                || entry.remaining<=omoba_sim::Fixed64::ZERO || entry.remaining>omoba_sim::Fixed64::from_i32(30)) {return None;}
            let id = (if let Some(id)=timed {buff_by_name(id)}else if zone {buff_by_name("ground_damage_zone")}else{buff_by_name(name)})
                .or_else(|| {
                    let (ability, stat) = omb_script_abi::buff_ids::declarative_buff_source(name)?;
                    omoba_template_ids::declarative_buff_visual_id(ability, stat)
                })
                // A v3 status's own buff, by its runtime id (E218).
                .or_else(|| omoba_template_ids::status_buff_by_key(name))?
                .raw();
            let raw = entry.remaining.raw();
            (raw != 0).then_some((id, if raw < 0 { -1 } else { raw }))
        }) {
            // A public icon means at least one source remains; no source count,
            // caster identity or internal key crosses the presentation boundary.
            values
                .entry(id)
                .and_modify(|old| {
                    *old = if *old == -1 || remaining == -1 {
                        -1
                    } else {
                        (*old).max(remaining)
                    };
                })
                .or_insert(remaining);
        }
        Self(values.into_iter().take(MAX_VISUAL_BUFFS).collect())
    }
    pub fn encode(&self) -> Option<Vec<u8>> {
        if self.0.len() > MAX_VISUAL_BUFFS {
            return None;
        }
        let mut bytes = b"BVS1".to_vec();
        bytes.extend((self.0.len() as u16).to_le_bytes());
        let mut previous = 0;
        for &(id, raw) in &self.0 {
            if id <= previous || try_buff_id_str(BuffId(id)).is_none() || (raw != -1 && raw <= 0) {
                return None;
            }
            previous = id;
            bytes.extend(id.to_le_bytes());
            bytes.extend(raw.to_le_bytes());
        }
        Some(bytes)
    }
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 6 || &bytes[..4] != b"BVS1" {
            return None;
        }
        let count = u16::from_le_bytes(bytes[4..6].try_into().ok()?) as usize;
        if count > MAX_VISUAL_BUFFS || bytes.len() != 6 + 10 * count {
            return None;
        }
        let state = Self(
            bytes[6..]
                .chunks_exact(10)
                .map(|row| {
                    (
                        u16::from_le_bytes(row[..2].try_into().unwrap()),
                        i64::from_le_bytes(row[2..].try_into().unwrap()),
                    )
                })
                .collect(),
        );
        state.encode()?;
        Some(state)
    }
    pub fn render_buffs(&self) -> Vec<crate::runtime::BuffSnapshot> {
        self.0
            .iter()
            .filter_map(|&(id, raw)| {
                Some(crate::runtime::BuffSnapshot {
                    buff_id: try_buff_id_str(BuffId(id))?.into(),
                    remaining_secs: if raw == -1 {
                        -1.0
                    } else {
                        omoba_sim::Fixed64::from_raw(raw).to_f32_for_render()
                    },
                    payload_json: String::new(),
                })
            })
            .collect()
    }
}
/// Public channel bars: remaining time plus an exact `total_raw`.
const CHANNEL_STATUSES:[&str;2]=[crate::runtime::native::recovery_channel::STATUS,crate::runtime::native::ability_channel::STATUS];
fn is_channel_status(name:&str)->bool {CHANNEL_STATUSES.contains(&name)}
/// One `{name:{"total_raw"}}` object, total within 60 s and not below the remaining time.
fn channel_status_valid(name:&str,entry:&crate::runtime::native::ability_runtime::BuffEntry)->bool {
    entry.payload.as_object().is_some_and(|o|o.len()==1)
        && entry.payload.get(name).and_then(|v|v.as_object()).is_some_and(|o|o.len()==1)
        && entry.payload[name]["total_raw"].as_i64().is_some_and(|raw|
            raw>0 && raw<=60*1024 && entry.remaining>omoba_sim::Fixed64::ZERO && entry.remaining.raw()<=raw)
}
/// The snapshot another team may see (E216): only the icons whose statuses are all
/// `reveal = "public"`, with their channel totals; empty when none is. Malformed bytes give None.
pub fn public_snapshot(bytes: &[u8]) -> Option<Vec<u8>> {
    let mut snapshot = BuffVisualSnapshot::decode(bytes)?;
    snapshot.buffs.0.retain(|(id, _)| omoba_template_ids::buff_is_public(BuffId(*id)));
    let kept = snapshot.buffs.0.clone();
    snapshot.channel_totals.retain(|(id, _)| kept.iter().any(|row| row.0 == *id));
    snapshot.encode()
}

/// Current owner-team snapshot. BVS1 remains an exact legacy codec.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuffVisualSnapshot {
    pub buffs: BuffVisualState,
    pub channel_totals: Vec<(u16,i64)>,
}
impl BuffVisualSnapshot {
    pub fn from_store(store:&crate::runtime::BuffStore,entity:specs::Entity)->Self {
        let buffs=BuffVisualState::from_store(store,entity);
        let mut channel_totals:Vec<(u16,i64)>=store.iter_for(entity).filter_map(|(name,entry)|{
            if !is_channel_status(name) || !channel_status_valid(name,entry){return None;}
            let id=buff_by_name(name)?.raw();if !buffs.0.iter().any(|row|row.0==id){return None;}
            Some((id,entry.payload[name]["total_raw"].as_i64()?))
        }).collect();
        channel_totals.sort_unstable();channel_totals.dedup_by_key(|row|row.0);
        Self {buffs,channel_totals}
    }
    fn valid(&self)->bool {
        if self.buffs.encode().is_none()||self.channel_totals.len()>MAX_VISUAL_BUFFS{return false;}
        let mut previous=0;
        for &(id,total) in &self.channel_totals {
            if id<=previous||!CHANNEL_STATUSES.iter().any(|name|buff_by_name(name).is_some_and(|known|known.raw()==id))
                ||!(1..=60*1024).contains(&total)||!self.buffs.0.iter().any(|&(key,remaining)|key==id&&remaining>0&&remaining<=total){return false;}
            previous=id;
        }
        true
    }
    pub fn encode(&self)->Option<Vec<u8>> {
        if !self.valid(){return None;}
        if self.channel_totals.is_empty(){return self.buffs.encode();}
        let mut bytes=b"BVS2".to_vec();bytes.extend((self.buffs.0.len() as u16).to_le_bytes());
        for &(id,remaining) in &self.buffs.0 {
            let total=self.channel_totals.iter().find(|row|row.0==id).map_or(0,|row|row.1);
            bytes.extend(id.to_le_bytes());bytes.extend(remaining.to_le_bytes());bytes.extend(total.to_le_bytes());
        }
        Some(bytes)
    }
    pub fn decode(bytes:&[u8])->Option<Self> {
        if bytes.starts_with(b"BVS1") {return Some(Self {buffs:BuffVisualState::decode(bytes)?,channel_totals:vec![]});}
        if bytes.len()<6||&bytes[..4]!=b"BVS2"{return None;}
        let count=u16::from_le_bytes(bytes[4..6].try_into().ok()?) as usize;
        if count>MAX_VISUAL_BUFFS||bytes.len()!=6+18*count{return None;}
        let mut buffs=vec![];let mut channel_totals=vec![];
        for row in bytes[6..].chunks_exact(18) {
            let id=u16::from_le_bytes(row[..2].try_into().ok()?);
            buffs.push((id,i64::from_le_bytes(row[2..10].try_into().ok()?)));
            let total=i64::from_le_bytes(row[10..18].try_into().ok()?);if total!=0{channel_totals.push((id,total));}
        }
        let state=Self {buffs:BuffVisualState(buffs),channel_totals};
        (state.valid()&&!state.channel_totals.is_empty()).then_some(state)
    }
    pub fn render_buffs(&self)->Vec<crate::runtime::BuffSnapshot> {
        if !self.valid(){return vec![];}
        let mut buffs=self.buffs.render_buffs();
        for row in &mut buffs {
            let Some(id)=buff_by_name(&row.buff_id)else{continue;};
            if let Some((_,total))=self.channel_totals.iter().find(|value|value.0==id.raw()) {
                row.payload_json=serde_json::json!({(row.buff_id.as_str()):{"total_raw":total}}).to_string();
            }
        }
        buffs
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use specs::{Builder, WorldExt};
    fn channel_snapshot(remaining:i64,total:i64)->BuffVisualSnapshot {
        let id=buff_by_name(crate::runtime::native::recovery_channel::STATUS).unwrap().raw();
        BuffVisualSnapshot {buffs:BuffVisualState(vec![(id,remaining)]),channel_totals:vec![(id,total)]}
    }
    #[test]
    fn versioned_buff_snapshot_preserves_legacy_and_strict_channel_bounds() {
        let legacy=BuffVisualState(vec![(buff_by_name("slow").unwrap().raw(),1024)]).encode().unwrap();
        let old=BuffVisualSnapshot::decode(&legacy).unwrap();assert!(old.channel_totals.is_empty());assert_eq!(old.encode().unwrap(),legacy);
        assert!(old.render_buffs()[0].payload_json.is_empty());
        let current=channel_snapshot(1536,3072);let bytes=current.encode().unwrap();assert_eq!(bytes.len(),24);assert_eq!(&bytes[..4],b"BVS2");
        assert_eq!(BuffVisualSnapshot::decode(&bytes),Some(current.clone()));assert!(BuffVisualState::decode(&bytes).is_none(),"do not claim old binaries understand BVS2");
        for length in 0..bytes.len(){assert!(BuffVisualSnapshot::decode(&bytes[..length]).is_none());}
        let mut extra=bytes.clone();extra.push(0);assert!(BuffVisualSnapshot::decode(&extra).is_none());
        for (remaining,total) in [(0,3072),(-1,3072),(3073,3072),(1,0),(1,-1),(1,61441)] {assert!(channel_snapshot(remaining,total).encode().is_none());}
        for total in [0,-1,61441] {let mut malformed=bytes.clone();malformed[16..24].copy_from_slice(&i64::to_le_bytes(total));assert!(BuffVisualSnapshot::decode(&malformed).is_none());}
        let mut duplicate=bytes.clone();duplicate[4..6].copy_from_slice(&2u16.to_le_bytes());duplicate.extend_from_slice(&bytes[6..]);assert!(BuffVisualSnapshot::decode(&duplicate).is_none());
        let mut oversized=bytes.clone();oversized[4..6].copy_from_slice(&65u16.to_le_bytes());assert!(BuffVisualSnapshot::decode(&oversized).is_none());
        let mut unknown=bytes.clone();unknown[6..8].copy_from_slice(&u16::MAX.to_le_bytes());assert!(BuffVisualSnapshot::decode(&unknown).is_none());
        let slow=buff_by_name("slow").unwrap().raw();assert!(BuffVisualSnapshot {buffs:BuffVisualState(vec![(slow,1024)]),channel_totals:vec![(slow,3072)]}.encode().is_none());
        assert!(BuffVisualSnapshot {buffs:current.buffs.clone(),channel_totals:vec![current.channel_totals[0],current.channel_totals[0]]}.encode().is_none());
    }
    #[test]
    fn public_channel_snapshot_from_store_sanitizes_and_clears_without_inferred_total() {
        use omoba_sim::Fixed64;let mut world=specs::World::new();let entity=world.create_entity().build();let mut store=crate::runtime::BuffStore::default();
        let status=crate::runtime::native::recovery_channel::STATUS;
        store.add(entity,status,Fixed64::from_raw(1536),serde_json::json!({(status):{"total_raw":3072}}));
        store.add(entity,"slow",Fixed64::ONE,serde_json::json!({"private_origin":"never-disclose","clock":987654}));
        let first=BuffVisualSnapshot::from_store(&store,entity);assert_eq!(first.channel_totals.len(),1);
        let rows=BuffVisualSnapshot::decode(&first.encode().unwrap()).unwrap().render_buffs();
        let channel=rows.iter().find(|row|row.buff_id==status).unwrap();assert_eq!(channel.remaining_secs,1.5);
        assert_eq!(serde_json::from_str::<serde_json::Value>(&channel.payload_json).unwrap(),serde_json::json!({(status):{"total_raw":3072}}));
        assert!(rows.iter().find(|row|row.buff_id=="slow").unwrap().payload_json.is_empty());
        store.tick(Fixed64::from_raw(512));let later=BuffVisualSnapshot::from_store(&store,entity);assert_eq!(later.channel_totals,first.channel_totals);
        assert_eq!(later.buffs.0.iter().find(|row|row.0==first.channel_totals[0].0).unwrap().1,1024);
        store.remove(entity,status);let cleared=BuffVisualSnapshot::from_store(&store,entity);assert!(cleared.channel_totals.is_empty());assert_eq!(&cleared.encode().unwrap()[..4],b"BVS1");
        store.add(entity,status,Fixed64::ONE,serde_json::json!({(status):{"total_raw":3072,"origin":"private"}}));
        assert!(BuffVisualSnapshot::from_store(&store,entity).channel_totals.is_empty());
    }
    #[test]
    fn pulse_channel_bar_uses_the_same_exact_public_total_contract() {
        use omoba_sim::Fixed64;let mut world=specs::World::new();let entity=world.create_entity().build();let mut store=crate::runtime::BuffStore::default();
        let status=crate::runtime::native::ability_channel::STATUS;
        store.add(entity,status,Fixed64::from_raw(2048),serde_json::json!({(status):{"total_raw":3264}}));
        let snapshot=BuffVisualSnapshot::from_store(&store,entity);
        assert_eq!(snapshot.channel_totals,vec![(buff_by_name(status).unwrap().raw(),3264)]);
        let rows=BuffVisualSnapshot::decode(&snapshot.encode().unwrap()).unwrap().render_buffs();
        let bar=rows.iter().find(|row|row.buff_id==status).unwrap();assert_eq!(bar.remaining_secs,2.0);
        assert_eq!(serde_json::from_str::<serde_json::Value>(&bar.payload_json).unwrap(),serde_json::json!({(status):{"total_raw":3264}}));
        // A total above 60 s or below the remaining time is not disclosed.
        store.remove(entity,status);store.add(entity,status,Fixed64::from_raw(4000),serde_json::json!({(status):{"total_raw":3264}}));
        assert!(BuffVisualSnapshot::from_store(&store,entity).channel_totals.is_empty());
    }
    #[test]
    fn timed_modifier_sources_publish_only_canonical_time_and_expire_independently() {
        use crate::runtime::ability_runtime::ItemTimedModifier;
        use omoba_sim::Fixed64;
        let mut world=specs::World::new();let e=world.create_entity().build();let other=world.create_entity().build();
        let mut store=crate::runtime::BuffStore::default();
        assert!(store.grant_timed_modifier(e,"@summoner:private-spell",ItemTimedModifier::Sprint,Fixed64::from_i32(80),Fixed64::from_i32(2)));
        assert!(store.grant_item_timed_modifier(e,"private-item",ItemTimedModifier::Sprint,Fixed64::from_i32(60),Fixed64::from_i32(6)));
        assert!(store.grant_item_timed_modifier(e,"private-defense",ItemTimedModifier::DamageReduction,Fixed64::from_raw(256),Fixed64::from_i32(3)));
        let state=BuffVisualSnapshot::from_store(&store,e);assert_eq!(state.buffs.0.len(),2);assert!(state.channel_totals.is_empty());
        let bytes=state.encode().unwrap();assert_eq!(&bytes[..4],b"BVS1");
        let rows=BuffVisualSnapshot::decode(&bytes).unwrap().render_buffs();
        assert_eq!(rows.iter().find(|r|r.buff_id=="movement_boost").unwrap().remaining_secs,6.);
        assert_eq!(rows.iter().find(|r|r.buff_id=="damage_reduction").unwrap().remaining_secs,3.);
        assert!(rows.iter().all(|r|r.payload_json.is_empty()));assert!(BuffVisualState::from_store(&store,other).0.is_empty());
        store.tick(Fixed64::from_i32(2));let rows=BuffVisualState::from_store(&store,e).render_buffs();
        assert_eq!(rows.iter().find(|r|r.buff_id=="movement_boost").unwrap().remaining_secs,4.);
        store.tick(Fixed64::ONE);let rows=BuffVisualState::from_store(&store,e).render_buffs();
        assert_eq!(rows.len(),1);assert_eq!(rows[0].buff_id,"movement_boost");assert_eq!(rows[0].remaining_secs,3.);
        store.tick(Fixed64::from_i32(3));assert!(BuffVisualState::from_store(&store,e).0.is_empty());
    }
    #[test]
    fn timed_modifier_public_status_rejects_forged_names_malformed_payloads_and_times() {
        use omoba_sim::Fixed64;
        let mut world=specs::World::new();let e=world.create_entity().build();
        for name in ["movement_boost","damage_reduction"] {
            let mut s=crate::runtime::BuffStore::default();s.add(e,name,Fixed64::ONE,serde_json::json!({}));
            assert!(BuffVisualState::from_store(&s,e).0.is_empty());
        }
        for case in 0..10 {
            let key=omb_script_abi::stat_keys::StatKey::MoveSpeedBonusBuff.as_str();
            let mut s=crate::runtime::BuffStore::default();let mut data=serde_json::json!({"__aggregation_family":"item_sprint",(key):81920});
            let time=match case {6=>Fixed64::ZERO,7=>Fixed64::from_i32(61),8=>Fixed64::from_raw(-1),_=>Fixed64::ONE};
            match case {0=>{data.as_object_mut().unwrap().remove(key);},1=>data[key]=serde_json::json!(80.0),2=>data[key]=serde_json::json!(0),
                3=>data[key]=serde_json::json!(-1),4=>data[key]=serde_json::json!(10_000*1024+1),5=>data["private"]=serde_json::json!(123),
                9=>data["__aggregation_family"]=serde_json::json!("item_damage_reduce"),_=>{}}
            s.add(e,"private-malformed",time,data);assert!(BuffVisualState::from_store(&s,e).0.is_empty(),"case{case}");
        }
    }
    #[test]
    fn timed_modifier_refresh_remove_and_reused_source_do_not_retain_stale_status() {
        use crate::runtime::ability_runtime::ItemTimedModifier;use omoba_sim::Fixed64;
        let mut world=specs::World::new();let e=world.create_entity().build();let mut s=crate::runtime::BuffStore::default();
        s.grant_item_timed_modifier(e,"same",ItemTimedModifier::Sprint,Fixed64::ONE,Fixed64::ONE);
        s.grant_item_timed_modifier(e,"same",ItemTimedModifier::Sprint,Fixed64::from_i32(2),Fixed64::from_i32(5));
        assert_eq!(BuffVisualState::from_store(&s,e).render_buffs()[0].remaining_secs,5.);
        s.remove_all_for(e);assert!(BuffVisualState::from_store(&s,e).0.is_empty());
        s.grant_item_timed_modifier(e,"same",ItemTimedModifier::DamageReduction,Fixed64::from_raw(128),Fixed64::ONE);
        let rows=BuffVisualState::from_store(&s,e).render_buffs();assert_eq!(rows.len(),1);assert_eq!(rows[0].buff_id,"damage_reduction");
    }
    fn zone_spec()->crate::runtime::ability_runtime::damage_zone::DamageZoneSpec {
        use omoba_sim::{Fixed64,Vec2};
        crate::runtime::ability_runtime::damage_zone::DamageZoneSpec {end:None,grow:None,linger:None,independent:false,center:Vec2::new(Fixed64::from_i32(123),Fixed64::ZERO),
            radius:Fixed64::from_i32(100),amount:Fixed64::from_i32(25),interval:Fixed64::from_raw(512),
            kind:omb_script_abi::types::DamageKind::Magical,profile:omb_script_abi::types::DamageProfile::FIRE.bits()}
    }
    #[test]
    fn public_zone_status_aggregates_time_and_never_exposes_private_source_or_geometry() {
        use omoba_sim::Fixed64;
        let mut world=specs::World::new();let source=world.create_entity().build();let mut store=crate::runtime::BuffStore::default();
        store.add(source,"private-zone-a:123:99",Fixed64::from_i32(3),zone_spec().payload());
        store.add(source,"private-zone-b:456:88",Fixed64::from_i32(2),zone_spec().payload());
        let state=BuffVisualState::from_store(&store,source);assert_eq!(state.0,vec![(buff_by_name("ground_damage_zone").unwrap().raw(),3072)]);
        let wire=state.encode().unwrap();let buffs=BuffVisualState::decode(&wire).unwrap().render_buffs();
        assert_eq!(buffs.len(),1);assert_eq!(buffs[0].buff_id,"ground_damage_zone");assert!(buffs[0].payload_json.is_empty());
        store.tick(Fixed64::from_raw(512));assert_eq!(BuffVisualState::from_store(&store,source).0[0].1,2560);
        store.add(source,"private-zone-a:123:99",Fixed64::ONE,zone_spec().payload());
        assert_eq!(BuffVisualState::from_store(&store,source).0[0].1,1536,"other active zone retains max remaining time");
        store.remove(source,"private-zone-b:456:88");assert_eq!(BuffVisualState::from_store(&store,source).0[0].1,1024);
        store.tick(Fixed64::ONE);assert!(BuffVisualState::from_store(&store,source).0.is_empty());
    }
    #[test]
    fn public_zone_status_rejects_forged_name_malformed_payload_and_invalid_lifetime() {
        use omoba_sim::Fixed64;
        let mut world=specs::World::new();let source=world.create_entity().build();
        for case in 0..5 {
            let mut store=crate::runtime::BuffStore::default();let mut payload=zone_spec().payload();
            let duration=match case {2=>Fixed64::ZERO,3=>Fixed64::from_i32(31),_=>Fixed64::ONE};
            if case==0 {payload=serde_json::json!({});}if case==1 {payload["damage_zone"]["amount"]=serde_json::json!("25");}
            if case==4 {payload["secret"]=serde_json::json!(123);}
            store.add(source,"ground_damage_zone",duration,payload);assert!(BuffVisualState::from_store(&store,source).0.is_empty(),"case{case}");
        }
    }
    #[test]
    fn explicit_death_retires_zone_status_before_next_buff_tick() {
        use crate::runtime::*;use omoba_sim::Fixed64;
        let mut world=StateInitializer::setup_campaign_ecs_world(&StateInitializer::create_thread_pool());world.insert(TowerTemplateRegistry::default());
        let source=world.create_entity().with(Pos(omoba_sim::Vec2::ZERO)).with(Faction::new(FactionType::Player,1))
            .with(CProperty{hp:Fixed64::from_i32(100),mhp:Fixed64::from_i32(100),msd:Fixed64::ZERO,def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO}).build();
        world.write_resource::<BuffStore>().add(source,"active-zone",Fixed64::ONE,zone_spec().payload());
        assert!(!BuffVisualState::from_store(&world.read_resource::<BuffStore>(),source).0.is_empty());
        world.write_resource::<Vec<Outcome>>().push(Outcome::Death{ent:source,pos:omoba_sim::Vec2::ZERO});
        process_outcomes(&mut world,&mut RuntimeEventVecSink::default()).unwrap();
        assert!(BuffVisualState::from_store(&world.read_resource::<BuffStore>(),source).0.is_empty());
    }
    #[test]
    fn buff_visual_state_codec_is_strict() {
        let id = buff_by_name("slow").unwrap().raw();
        let state = BuffVisualState(vec![(id, 1024)]);
        let bytes = state.encode().unwrap();
        assert_eq!(BuffVisualState::decode(&bytes), Some(state));
        for length in 0..bytes.len() {
            assert!(BuffVisualState::decode(&bytes[..length]).is_none());
        }
        let mut extra = bytes;
        extra.push(0);
        assert!(BuffVisualState::decode(&extra).is_none());
        for rows in [
            vec![(0, 1)],
            vec![(id, 0)],
            vec![(id, -2)],
            vec![(id, 1), (id, 2)],
            vec![(u16::MAX, 1)],
            vec![(id, 1); 65],
        ] {
            assert!(BuffVisualState(rows).encode().is_none());
        }
        assert!(BuffVisualState(vec![(id, -1)]).encode().is_some());
        assert_eq!(
            BuffVisualState::decode(&BuffVisualState(vec![]).encode().unwrap()),
            Some(BuffVisualState(vec![]))
        );
    }
    #[test]
    fn buff_visual_state_excludes_private_payload_and_dynamic_names() {
        let mut world = specs::World::new();
        let entity = world.create_entity().build();
        let mut store = crate::runtime::BuffStore::default();
        store.add(
            entity,
            "slow",
            omoba_sim::Fixed64::from_i32(2),
            serde_json::json!({"source":999,"secret":"private"}),
        );
        store.add(
            entity,
            "generic_mana:hidden:123:456",
            omoba_sim::Fixed64::ONE,
            serde_json::json!({}),
        );
        let state = BuffVisualState::from_store(&store, entity);
        assert_eq!(state.0.len(), 1);
        let buffs = state.render_buffs();
        assert_eq!(buffs[0].buff_id, "slow");
        assert!(buffs[0].payload_json.is_empty());
        assert_eq!(buffs[0].remaining_secs, 2.0);
    }

    #[test]
    fn v3_statuses_show_their_own_buff_by_runtime_id() {
        let mut world = specs::World::new();
        let entity = world.create_entity().build();
        let mut store = crate::runtime::BuffStore::default();
        for key in ["b01_sniper_mode", "generic_slow:a34_iceberg_strike:5:1", "a07_strongest_r2:5:1", "stun"] {
            store.add(entity, key, omoba_sim::Fixed64::from_i32(3), serde_json::json!({}));
        }
        let mut names: Vec<String> = BuffVisualState::from_store(&store, entity).render_buffs().into_iter().map(|b| b.buff_id.to_string()).collect();
        names.sort();
        // The status id names the buff; a key a hand-written buff names (stun) keeps it.
        assert_eq!(names, ["a07t_strongest", "a34e_slow", "b01w_sniper", "stun"]);
    }

    #[test]
    fn buff_visual_state_declared_sources_merge_without_private_identity() {
        let mut world = specs::World::new();
        let entity = world.create_entity().build();
        let mut store = crate::runtime::BuffStore::default();
        for (key, seconds) in [
            ("slow", 2),
            ("generic_slow:ranger_shot:100:3", 4),
            ("generic_slow:ranger_shot:101:3", 3),
            ("generic_dot:ranger_shot:100:3",4),
            ("generic_dot:ranger_shot:101:3",3),
            ("generic_mana:ranger_patch:mana_regen_constant:100:3", 5),
            ("generic_mana:vanguard_recover:mana_bonus:100:3", 6),
            ("generic_mana:ranger_patch:mana_bonus:100:3", 9),
            ("generic_slow:missing:100:3", 9),
            ("generic_slow:ranger_shot:0100:3", 9),
        ] {
            store.add(
                entity,
                key,
                omoba_sim::Fixed64::from_i32(seconds),
                serde_json::json!({"secret":999}),
            );
        }
        let state = BuffVisualState::from_store(&store, entity);
        assert_eq!(state.0.len(), 4);
        assert!(
            state.encode().is_some(),
            "merged sources keep strict unique catalog IDs"
        );
        let buffs = state.render_buffs();
        assert_eq!(buffs.iter().find(|b|b.buff_id=="bleed").unwrap().remaining_secs,4.0);
        assert!(buffs
            .iter()
            .all(|b| b.payload_json.is_empty() && !b.buff_id.contains(':')));
        assert_eq!(
            buffs
                .iter()
                .find(|b| b.buff_id == "slow")
                .unwrap()
                .remaining_secs,
            4.0
        );
        assert_eq!(
            buffs
                .iter()
                .find(|b| b.buff_id == "mana_regeneration")
                .unwrap()
                .remaining_secs,
            5.0
        );
        assert_eq!(
            buffs
                .iter()
                .find(|b| b.buff_id == "mana_capacity")
                .unwrap()
                .remaining_secs,
            6.0
        );
        store.remove(entity, "generic_slow:ranger_shot:100:3");
        assert_eq!(
            BuffVisualState::from_store(&store, entity)
                .render_buffs()
                .iter()
                .find(|b| b.buff_id == "slow")
                .unwrap()
                .remaining_secs,
            3.0
        );
        store.tick(omoba_sim::Fixed64::from_i32(6));
        assert!(BuffVisualState::from_store(&store, entity).0.is_empty());
    }
}
