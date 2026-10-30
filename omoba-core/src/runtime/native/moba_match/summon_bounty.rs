//! 召喚物賞金（原作 bounty；v3 unit_templates `bounty.gold`／`bounty.xp`，summons.lua
//! `bounty_gold`／`bounty_xp`）：第一個致命傷害的歸屬（jungle_credit，與野怪相同）若是敵方玩家，
//! 召喚物死亡時把金錢與英雄經驗給那名玩家（credit_moba_player）。過期、被自己隊伍殺死或
//! 非比賽中都不給。
use super::*;

/// The first lethal hit on a summon with a bounty records its enemy killer player.
pub(super) fn lethal(world: &World, state: &MobaMatch, credit: Option<jungle_credit::JungleCredit>, target: Entity) {
    let side = world.read_storage::<Faction>().get(target)
        .and_then(|faction| state.config.teams.iter().position(|team| i64::from(*team) == i64::from(faction.team_id)));
    let mut summons = world.write_storage::<SummonedUnit>();
    let Some(summon) = summons.get_mut(target).filter(|summon| summon.bounty != (0, 0) && summon.killer_player.is_none()) else { return };
    summon.killer_player = credit.filter(|credit| Some(credit.side) != side).and_then(|credit| credit.player);
}

/// At the summon's death: its bounty to the recorded killer player.
pub(super) fn death(world: &World, state: &mut MobaMatch, entity: Entity) {
    if state.phase != MobaMatchPhase::Playing || world.read_resource::<GamePause>().is_paused { return; }
    let Some((gold, xp, killer)) = world.read_storage::<SummonedUnit>().get(entity)
        .and_then(|summon| Some((summon.bounty.0, summon.bounty.1, summon.killer_player?))) else { return };
    if let Some(index) = state.heroes.iter().position(|slot| slot.player_id == killer) {
        credit_moba_player(world, state, index, gold, xp);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{process_outcomes, RuntimeEventVecSink, TowerTemplateRegistry};
    use specs::{Builder, Join};

    fn lethal(source: Entity, target: Entity) -> Outcome {
        Outcome::Damage { pos: Vec2::ZERO, phys: Fixed64::ZERO, magi: Fixed64::ZERO, real: Fixed64::from_i32(100_000),
            source, target, damage_profile: 0, predeclared: false }
    }

    fn death(ent: Entity) -> Outcome { Outcome::Death { pos: Vec2::ZERO, ent } }

    #[test]
    fn an_enemy_kill_of_a_summon_pays_its_bounty_once_and_an_own_team_kill_pays_none() {
        let pool = crate::runtime::StateInitializer::create_thread_pool();
        let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        world.insert(TowerTemplateRegistry::default());
        setup_single_lane_match(&mut world, SingleLaneConfig { warmup: Fixed64::ZERO, ..Default::default() }).unwrap();
        let (mine, theirs) = { let mut state = world.write_resource::<MobaMatch>(); state.phase = MobaMatchPhase::Playing;
            (state.heroes[0].entity.unwrap(), state.heroes[1].entity.unwrap()) };
        for gold in (&mut world.write_storage::<Gold>()).join() { gold.0 = 0; }
        let summon = |world: &mut World, owner: Entity| {
            let faction = world.read_storage::<Faction>().get(owner).unwrap().clone();
            let mut body = SummonedUnit::new_fixed(owner, None);
            body.bounty = (40, 25);
            world.create_entity().with(body).with(faction).with(Pos(Vec2::ZERO))
                .with(CProperty { hp: Fixed64::from_i32(100), mhp: Fixed64::from_i32(100), msd: Fixed64::ZERO,
                    def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO }).build()
        };
        let flush = |world: &mut World, outcomes: Vec<Outcome>| {
            world.write_resource::<Vec<Outcome>>().extend(outcomes);
            process_outcomes(world, &mut RuntimeEventVecSink::default()).unwrap();
            world.maintain();
        };
        let gold = |world: &World, hero: Entity| world.read_storage::<Gold>().get(hero).unwrap().0;
        let enemy = summon(&mut world, theirs);
        // The lethal hit records the killer; the death (the next settlement) pays.
        flush(&mut world, vec![lethal(mine, enemy), lethal(mine, enemy)]);
        assert_eq!(world.read_storage::<SummonedUnit>().get(enemy).unwrap().killer_player, Some(1));
        flush(&mut world, vec![death(enemy), death(enemy)]);
        assert_eq!(gold(&world, mine), 40, "the killer's player, once");
        assert_eq!(gold(&world, theirs), 0);
        let own = summon(&mut world, mine);
        flush(&mut world, vec![lethal(mine, own)]);
        flush(&mut world, vec![death(own)]);
        assert_eq!(gold(&world, mine), 40, "an own-team kill pays none");
    }
}
