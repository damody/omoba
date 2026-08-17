use crate::comp::{CProperty, Creep, DeltaTime, Job, System};
use crate::runtime::{resolve_td_regrow_parent, TD_REGROW_INTERVAL};
use omoba_sim::Fixed64;
use specs::{Join, Read, WriteStorage};

#[derive(Default)]
pub struct Sys;

impl<'a> System<'a> for Sys {
    type SystemData = (
        Read<'a, DeltaTime>,
        WriteStorage<'a, Creep>,
        WriteStorage<'a, CProperty>,
    );

    const NAME: &'static str = "td_regrow";

    fn run(_job: &mut Job<Self>, (dt, mut creeps, mut properties): Self::SystemData) {
        if dt.0 <= Fixed64::ZERO {
            return;
        }
        let catalog = omoba_template_ids::active_td_layer_catalog();
        for (creep, property) in (&mut creeps, &mut properties).join() {
            let Some(state) = creep.td_layer.as_mut() else {
                continue;
            };
            if state.properties & omoba_template_ids::td_rounds::layer_property::REGROW == 0 {
                continue;
            }
            state.regrow_elapsed += dt.0;
            let mut drained = 0usize;
            while state.regrow_elapsed >= TD_REGROW_INTERVAL && drained < catalog.len() {
                let elapsed = state.regrow_elapsed - TD_REGROW_INTERVAL;
                match resolve_td_regrow_parent(catalog, state) {
                    Ok(Some(mut parent)) => {
                        parent.state.regrow_elapsed = elapsed;
                        *state = parent.state;
                        property.hp = parent.hp;
                        property.mhp = parent.max_hp;
                        if let Some(metadata) =
                            omoba_template_ids::active_td_layer_by_name(&state.current_layer)
                        {
                            property.msd = Fixed64::from_i32(metadata.move_speed as i32);
                            creep.name = format!("td_btd_{}", state.current_layer);
                            creep.label = Some(metadata.label.to_string());
                        }
                    }
                    Ok(None) => {
                        state.regrow_elapsed = Fixed64::ZERO;
                        break;
                    }
                    Err(error) => {
                        log::error!(
                            "TD regrow rejected layer={} ceiling={}: {:?}",
                            state.current_layer,
                            state.regrow_ceiling,
                            error
                        );
                        state.regrow_elapsed = Fixed64::ZERO;
                        break;
                    }
                }
                drained += 1;
            }
            if drained == catalog.len() && state.regrow_elapsed >= TD_REGROW_INTERVAL {
                log::error!(
                    "TD regrow drain exceeded catalog bound layer={} elapsed_raw={}",
                    state.current_layer,
                    state.regrow_elapsed.raw()
                );
                state.regrow_elapsed =
                    Fixed64::from_raw(state.regrow_elapsed.raw() % TD_REGROW_INTERVAL.raw());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comp::{run_now, CreepStatus, TdLayerState};
    use crate::runtime::SimulationTickProfile;
    use specs::{Builder, World, WorldExt};

    fn regrow_state_for_profile(profile: SimulationTickProfile) -> (String, i64, u32, i64) {
        let mut world = World::new();
        world.register::<Creep>();
        world.register::<CProperty>();
        world.insert(DeltaTime(Fixed64::ZERO));
        world.insert(crate::comp::SysMetrics::default());
        world.insert(crate::comp::TickProfile::default());
        let entity = world
            .create_entity()
            .with(Creep {
                name: "td_btd_blue".to_string(),
                label: Some("Blue".to_string()),
                path: "main".to_string(),
                pidx: 0,
                path_remaining_distance: Fixed64::from_i32(100),
                block_tower: None,
                status: CreepStatus::Walk,
                td_layer: Some(TdLayerState {
                    base_archetype: "yellow".to_string(),
                    current_layer: "blue".to_string(),
                    properties: omoba_template_ids::td_rounds::layer_property::REGROW,
                    regrow_ceiling: "yellow".to_string(),
                    regrow_elapsed: Fixed64::ZERO,
                    remaining_leak_value: 2,
                    spawn_lineage: 9,
                }),
            })
            .with(CProperty {
                hp: Fixed64::ONE,
                mhp: Fixed64::ONE,
                msd: Fixed64::from_i32(140),
                def_physic: Fixed64::ZERO,
                def_magic: Fixed64::ZERO,
            })
            .build();

        for tick in 1..=u64::from(profile.ticks_per_game_second()) * 7 {
            world.write_resource::<DeltaTime>().0 =
                Fixed64::from_raw(profile.fixed_raw_for_tick(tick));
            run_now::<Sys>(&world);
        }

        let creeps = world.read_storage::<Creep>();
        let properties = world.read_storage::<CProperty>();
        let state = creeps.get(entity).unwrap().td_layer.as_ref().unwrap();
        (
            state.current_layer.clone(),
            state.regrow_elapsed.raw(),
            state.remaining_leak_value,
            properties.get(entity).unwrap().msd.raw(),
        )
    }

    #[test]
    fn coarse_tick_drains_due_regrow_steps_and_keeps_remainder() {
        let mut world = World::new();
        world.register::<Creep>();
        world.register::<CProperty>();
        world.insert(DeltaTime(Fixed64::from_raw(6 * 1024 + 512)));
        world.insert(crate::comp::SysMetrics::default());
        world.insert(crate::comp::TickProfile::default());
        let entity = world
            .create_entity()
            .with(Creep {
                name: "td_btd_blue".to_string(),
                label: Some("Blue".to_string()),
                path: "main".to_string(),
                pidx: 0,
                path_remaining_distance: Fixed64::from_i32(100),
                block_tower: None,
                status: CreepStatus::Walk,
                td_layer: Some(TdLayerState {
                    base_archetype: "yellow".to_string(),
                    current_layer: "blue".to_string(),
                    properties: omoba_template_ids::td_rounds::layer_property::REGROW,
                    regrow_ceiling: "yellow".to_string(),
                    regrow_elapsed: Fixed64::ZERO,
                    remaining_leak_value: 2,
                    spawn_lineage: 9,
                }),
            })
            .with(CProperty {
                hp: Fixed64::ONE,
                mhp: Fixed64::ONE,
                msd: Fixed64::from_i32(140),
                def_physic: Fixed64::ZERO,
                def_magic: Fixed64::ZERO,
            })
            .build();
        run_now::<Sys>(&world);
        let creeps = world.read_storage::<Creep>();
        let state = creeps.get(entity).unwrap().td_layer.as_ref().unwrap();
        assert_eq!(state.current_layer, "yellow");
        assert_eq!(state.regrow_elapsed, Fixed64::from_raw(512));
        assert_eq!(state.remaining_leak_value, 4);
        assert_eq!(
            world.read_storage::<CProperty>().get(entity).unwrap().msd,
            Fixed64::from_i32(185)
        );
    }

    #[test]
    fn regrow_drains_identically_at_fifteen_and_one_twenty_hz() {
        let coarse = regrow_state_for_profile(SimulationTickProfile::Coarse15Hz);
        let fine = regrow_state_for_profile(SimulationTickProfile::Production120Hz);

        assert_eq!(coarse, fine);
        assert_eq!(coarse, ("yellow".to_string(), 1024, 4, 185 * 1024));
    }
}
