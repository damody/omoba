use crate::comp::*;
use specs::prelude::ParallelIterator;
use specs::{shred, Entities, Entity, ParJoin, Read, ReadStorage, SystemData, Write, WriteStorage};
use std::time::Instant;

#[derive(SystemData)]
pub struct NearbyRead<'a> {
    entities: Entities<'a>,
    time: Read<'a, Time>,
    creeps: ReadStorage<'a, Creep>,
    units: ReadStorage<'a, Unit>,
    heroes: ReadStorage<'a, Hero>,
    pos: ReadStorage<'a, Pos>,
}

#[derive(SystemData)]
pub struct NearbyWrite<'a> {
    entities: Entities<'a>,
    towers: WriteStorage<'a, Tower>,
    searcher: Write<'a, Searcher>,
}

#[derive(Default)]
pub struct Sys;

/// Push each `(Entity, Pos)` into a per-worker vector, then concatenate.
/// Rayon concatenation order is not sorted. No per-row `Vec` is allocated.
fn collect_entity_pos_rows<I>(rows: I) -> Vec<(Entity, Pos)>
where
    I: ParallelIterator<Item = (Entity, Pos)>,
{
    rows.fold(Vec::new, |mut acc: Vec<(Entity, Pos)>, row| {
        acc.push(row);
        acc
    })
    .reduce(Vec::new, |mut left, mut right| {
        left.append(&mut right);
        left
    })
}

/// Unit rows are passed before creep rows. `sort_by_key` is stable, so equal
/// `(id, generation)` keys keep that group order. f32 projection stays `xy_f32`.
fn sorted_f32_index_items(groups: &[&[(Entity, Pos)]]) -> Vec<(Entity, vek::Vec2<f32>)> {
    let mut items: Vec<(Entity, vek::Vec2<f32>)> = groups
        .iter()
        .flat_map(|rows| rows.iter())
        .map(|(entity, pos)| {
            let (x, y) = pos.xy_f32();
            (*entity, vek::Vec2::new(x, y))
        })
        .collect();
    items.sort_by_key(|(entity, _)| (entity.id(), entity.gen().id()));
    items
}

impl<'a> System<'a> for Sys {
    type SystemData = (NearbyRead<'a>, NearbyWrite<'a>);

    const NAME: &'static str = "nearby";

    fn run(_job: &mut Job<Self>, (tr, mut tw): Self::SystemData) {
        {
            //unit update (包含所有單位：creeps, units)
            // 收集所有 Unit 實體
            let unit_rows = collect_entity_pos_rows(
                (&tr.entities, &tr.pos, &tr.units).par_join().map_init(
                    || {
                        prof_span!(guard, "unit nearby update rayon job");
                        guard
                    },
                    |_guard, (ent, pos, _)| (ent, *pos),
                ),
            );

            // 收集所有 Creep 實體（保持向後兼容）
            let creep_rows = collect_entity_pos_rows(
                (&tr.entities, &tr.pos, &tr.creeps).par_join().map_init(
                    || {
                        prof_span!(guard, "creep nearby update rayon job");
                        guard
                    },
                    |_guard, (ent, pos, _)| (ent, *pos),
                ),
            );

            // 合併所有實體到 creep 索引中（向後兼容）— 走 CollisionIndex::rebuild_from
            // 注意：搜尋器/空間索引在內部使用 f32 來實作 instant_distance lib 相容性。
            // 根據權威 Pos 在每次更新時重建快取；下面按實體 ID 排序的條目
            // 用於 par_join 的確定性插入順序。呼叫者的最終距離檢查是固定64。
            // 確定性：par_join 收集順序是不確定的；依實體 id 排序
            // 確保空間索引的跨主機插入順序相同。
            let combined = sorted_f32_index_items(&[unit_rows.as_slice(), creep_rows.as_slice()]);
            tw.searcher.creep.rebuild_from(combined);

            log::debug!(
                "Updated searcher index: {} units, {} creeps",
                unit_rows.len(),
                creep_rows.len()
            );
        }
        {
            // hero update — 每 tick 重建（英雄會移動）
            let hero_rows = collect_entity_pos_rows(
                (&tr.entities, &tr.pos, &tr.heroes).par_join().map_init(
                    || {
                        prof_span!(guard, "hero nearby update rayon job");
                        guard
                    },
                    |_guard, (ent, pos, _)| (ent, *pos),
                ),
            );

            // 注意：搜尋器/空間索引在內部使用 f32 來實作 instant_distance lib 相容性。
            // 根據權威 Pos 在每次更新時重建快取；下面按實體 ID 排序的條目
            // 用於 par_join 的確定性插入順序。呼叫者的最終距離檢查是固定64。
            // 確定性：par_join 收集順序是不確定的；依實體 ID 排序。
            let hero_items = sorted_f32_index_items(&[hero_rows.as_slice()]);
            tw.searcher.hero.rebuild_from(hero_items);
        }
        if tw.searcher.tower.is_dirty() {
            let tower_rows = collect_entity_pos_rows(
                (&tr.entities, &tr.pos, &tw.towers).par_join().map_init(
                    || {
                        prof_span!(guard, "nearby update rayon job");
                        guard
                    },
                    |_guard, (ent, pos, _)| (ent, *pos),
                ),
            );
            if tw.searcher.tower.is_dirty() {
                let time1 = Instant::now();
                // 注意：搜尋器/空間索引在內部使用 f32 來實作 instant_distance lib 相容性。
                // 根據權威 Pos 在每次更新時重建快取；下面按實體 ID 排序的條目
                // 用於 par_join 的確定性插入順序。呼叫者的最終距離檢查是固定64。
                // 確定性：par_join 收集順序是不確定的；依實體 ID 排序。
                let tower_items = sorted_f32_index_items(&[tower_rows.as_slice()]);
                tw.searcher.tower.rebuild_from(tower_items);
                let time2 = Instant::now();
                let elpsed = time2.duration_since(time1);
                log::info!("build tower Sort pos time {:?}", elpsed);
            }
        }
    }
}

#[cfg(test)]
mod nearby_collection {
    use super::{collect_entity_pos_rows, sorted_f32_index_items};
    use crate::comp::{CollisionIndex, Pos};
    use crate::runtime::SpatialIndexParams;
    use rayon::prelude::*;
    use specs::world::Generation;
    use specs::Entity;

    fn row(index: u32, generation: i32, x: f32, y: f32) -> (Entity, Pos) {
        (
            Entity::new(index, Generation::new(generation)),
            Pos::from_xy_f32(x, y),
        )
    }

    fn f32_at(pos: Pos) -> vek::Vec2<f32> {
        let (x, y) = pos.xy_f32();
        vek::Vec2::new(x, y)
    }

    #[test]
    fn empty_rows_stay_empty_through_sort_and_index() {
        let rows: Vec<(Entity, Pos)> = Vec::new();
        let collected = collect_entity_pos_rows(rows.into_par_iter());
        assert!(collected.is_empty());

        let items = sorted_f32_index_items(&[collected.as_slice(), &[], &[]]);
        assert!(items.is_empty());

        let mut index = CollisionIndex::new("hash_grid", SpatialIndexParams::default());
        index.rebuild_from(items);
        assert_eq!(index.count(), 0);
    }

    #[test]
    fn parallel_many_rows_match_serial_pairs_after_same_sort() {
        let serial: Vec<(Entity, Pos)> = (0..1024u32)
            .map(|i| row(i.wrapping_mul(3).wrapping_add(7), (i % 17) as i32 + 1, i as f32, -(i as f32)))
            .collect();
        let mut scrambled = serial.clone();
        scrambled.reverse();

        let parallel = collect_entity_pos_rows(scrambled.clone().into_par_iter());
        assert_eq!(parallel.len(), serial.len());

        let parallel_items = sorted_f32_index_items(&[parallel.as_slice()]);
        let serial_items = sorted_f32_index_items(&[serial.as_slice()]);
        assert_eq!(parallel_items, serial_items);

        let mut paired = parallel;
        paired.sort_by_key(|(entity, _)| (entity.id(), entity.gen().id()));
        let mut serial_pairs = serial;
        serial_pairs.sort_by_key(|(entity, _)| (entity.id(), entity.gen().id()));
        assert_eq!(paired, serial_pairs);
        assert!(paired.windows(2).all(|w| {
            (w[0].0.id(), w[0].0.gen().id()) <= (w[1].0.id(), w[1].0.gen().id())
        }));
    }

    #[test]
    fn duplicate_generation_order_is_kept_in_creep_index_count() {
        let unit_same = row(2, 1, 1.0, 0.0);
        let unit_later_gen = row(2, 3, 3.0, 0.0);
        let creep_same = row(2, 1, 9.0, 0.0);
        let creep_lower_id = row(1, 5, 0.0, 4.0);

        let unit_rows = collect_entity_pos_rows(vec![unit_later_gen, unit_same].into_par_iter());
        let creep_rows = collect_entity_pos_rows(vec![creep_same, creep_lower_id].into_par_iter());
        let items = sorted_f32_index_items(&[unit_rows.as_slice(), creep_rows.as_slice()]);

        let keys: Vec<_> = items
            .iter()
            .map(|(entity, _)| (entity.id(), entity.gen().id()))
            .collect();
        assert_eq!(keys, vec![(1, 5), (2, 1), (2, 1), (2, 3)]);
        assert_eq!(items[0].1, f32_at(creep_lower_id.1));
        assert_eq!(items[1].1, f32_at(unit_same.1));
        assert_eq!(items[2].1, f32_at(creep_same.1));
        assert_eq!(items[3].1, f32_at(unit_later_gen.1));

        let mut index = CollisionIndex::new("hash_grid", SpatialIndexParams::default());
        index.rebuild_from(items.iter().copied());
        assert_eq!(index.count(), items.len());
        assert_eq!(index.count(), 4);
    }
}
