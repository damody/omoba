use crate::comp::*;
use crate::runtime::native::item::{apply_item_stats, ItemStatAggregate, ItemStatApplication};
use specs::{shred, Entities, Join, Read, ReadExpect, SystemData, WriteStorage, Write};

#[derive(SystemData)]
pub struct ItemRead<'a> {
    entities: Entities<'a>,
    dt: Read<'a, DeltaTime>,
    item_reg: ReadExpect<'a, crate::item::ItemRegistry>,
}

#[derive(SystemData)]
pub struct ItemWrite<'a> {
    buffs: Write<'a,crate::runtime::BuffStore>,
    inventories: WriteStorage<'a, Inventory>,
    effects: WriteStorage<'a, ItemEffects>,
    properties: WriteStorage<'a, CProperty>,
    attacks: WriteStorage<'a, TAttack>,
}

#[derive(Default)]
pub struct Sys;

impl<'a> System<'a> for Sys {
    type SystemData = (ItemRead<'a>, ItemWrite<'a>);

    const NAME: &'static str = "item";

    fn run(_job: &mut Job<Self>, (tr, mut tw): Self::SystemData) {
        let dt = tr.dt.0.to_f32_for_render();

        // 1) 遞減所有 inventory CD
        for inv in (&mut tw.inventories).join() {
            for slot in inv.slots.iter_mut() {
                if let Some(inst) = slot {
                    if inst.cooldown_remaining > 0.0 {
                        inst.cooldown_remaining = (inst.cooldown_remaining - dt).max(0.0);
                    }
                }
            }
        }

        // 2) 若 ItemEffects.dirty → 重算 bonus 並同步到 CProperty/TAttack
        // 先收集需要重算的 entities
        let dirty_ents: Vec<specs::Entity> = (&tr.entities, &tw.effects)
            .join()
            .filter(|(_, eff)| eff.dirty)
            .map(|(e, _)| e)
            .collect();

        for e in dirty_ents {
            let totals = tw.inventories.get(e).map(|inv| ItemStatAggregate::from_inventory(inv, &tr.item_reg))
                .unwrap_or_default();
            let applied = tw.effects.get(e).copied().unwrap_or_default();
            let next = apply_item_stats(
                totals,
                &applied,
                tw.properties.get_mut(e),
                tw.attacks.get_mut(e),
                ItemStatApplication::ExistingBody,
            );
            if let Some(eff) = tw.effects.get_mut(e) {
                *eff = next;
            }
            crate::runtime::native::item::sync_item_passives(e,totals,&mut tw.buffs);

            log::info!(
                "ItemEffects 重算 entity={:?}: atk+{} hp+{} ms+{} armor+{} mp+{}",
                e,
                totals.atk,
                totals.hp,
                totals.ms,
                totals.armor,
                totals.mp
            );
        }
    }
}
