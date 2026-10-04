//! Public persistent scores only. No entity lookup or private economy inference.
use omoba_core::{game_proto::{ScoreboardPresentation, ScoreboardRowPresentation}, runtime::*};
use std::collections::BTreeMap;

pub fn project(snapshot: &FilteredRenderSnapshot) -> Option<ScoreboardPresentation> {
    let namespaces = [SINGLE_LANE_SCORE_TEAM_METRIC_ID, SINGLE_LANE_SCORE_KILLS_METRIC_ID,
        SINGLE_LANE_SCORE_DEATHS_METRIC_ID, SINGLE_LANE_SCORE_ASSISTS_METRIC_ID];
    let mut count = None;
    let mut players: BTreeMap<u32, [Option<u32>; 4]> = BTreeMap::new();
    for event in &snapshot.public_events {
        let p = &event.sanitized_payload;
        if event.event_kind != FactKind::Hud as u32 || p.len() != 20 { continue; }
        let id = u64::from_le_bytes(p[4..12].try_into().ok()?);
        let value = || u32::try_from(i64::from_le_bytes(p[12..20].try_into().unwrap())).ok();
        if id == SINGLE_LANE_SCORE_COUNT_METRIC_ID {
            if p[..4] != 0u32.to_le_bytes() || count.replace(value()?).is_some() { return None; }
        } else {
            for (index, ns) in namespaces.into_iter().enumerate() {
                if id & 0xffff_ffff_0000_0000 != ns { continue; }
                let player = id as u32;
                if player == 0 || p[..4] != 0u32.to_le_bytes() { return None; }
                let row = players.entry(player).or_default();
                if row[index].replace(value()?).is_some() || players.len() > 10 { return None; }
            }
        }
    }
    let count = count?;
    if !(2..=10).contains(&count) || players.len() != count as usize { return None; }
    let mut rows = Vec::new();
    for (player_id, [team, kills, deaths, assists]) in players {
        rows.push(ScoreboardRowPresentation { player_id, team_id: team?, kills: kills?, deaths: deaths?, assists: assists? });
    }
    rows.sort_by_key(|row| (row.team_id, row.player_id));
    let board = ScoreboardPresentation { schema_version: 1, rows };
    valid(&board).then_some(board)
}

pub fn valid(board: &ScoreboardPresentation) -> bool {
    if board.schema_version != 1 || !(2..=10).contains(&board.rows.len()) { return false; }
    let mut teams = BTreeMap::new();
    let mut players = std::collections::BTreeSet::new();
    let mut previous = None;
    for row in &board.rows {
        let key = (row.team_id, row.player_id);
        if row.team_id == 0 || row.player_id == 0 || !players.insert(row.player_id)
            || previous.is_some_and(|p| p >= key) { return false; }
        previous = Some(key);
        *teams.entry(row.team_id).or_insert(0) += 1;
    }
    teams.len() == 2 && teams.values().all(|count| *count <= 5)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn snapshot(n: u32) -> FilteredRenderSnapshot {
        let mut s = FilteredRenderSnapshot { team_id: 1, replica_tick: 1, entities: vec![], public_events: vec![],
            external_effects: vec![], memory_directives: vec![], remembered_presentations: Default::default() };
        let mut metric = |id: u64, v: i64| s.public_events.push(omoba_core::game_proto::TeamPublicEvent {
            event_kind: FactKind::Hud as u32, sanitized_payload: [0u32.to_le_bytes().as_slice(),
                id.to_le_bytes().as_slice(), v.to_le_bytes().as_slice()].concat(), ..Default::default() });
        metric(SINGLE_LANE_SCORE_COUNT_METRIC_ID, i64::from(n));
        for player in (1..=n).rev() {
            for (ns,v) in [(SINGLE_LANE_SCORE_TEAM_METRIC_ID,if player%2==1 {1} else {2}),
                (SINGLE_LANE_SCORE_KILLS_METRIC_ID,u32::MAX),(SINGLE_LANE_SCORE_DEATHS_METRIC_ID,2),
                (SINGLE_LANE_SCORE_ASSISTS_METRIC_ID,3)] {
                metric(single_lane_player_metric(ns,player),i64::from(v));
            }
        }
        s
    }
    #[test]
    fn ten_players_are_public_without_live_entities_and_sorted() {
        let mut s = snapshot(10);
        let b = project(&s).unwrap();
        assert_eq!(b.rows.len(),10);
        assert_eq!(b.rows[0].kills,u32::MAX);
        assert_eq!(b.rows.iter().map(|r|r.player_id).collect::<Vec<_>>(),vec![1,3,5,7,9,2,4,6,8,10]);
        s.team_id=2;
        assert_eq!(project(&s).unwrap(),b);
    }
    #[test]
    fn schema_order_and_two_bounded_teams_are_required() {
        let good = project(&snapshot(10)).unwrap();
        let mut bad = good.clone();
        bad.schema_version = 2;
        assert!(!valid(&bad));
        bad = good.clone();
        bad.rows.reverse();
        assert!(!valid(&bad));
        bad = good.clone();
        bad.rows[5].team_id = 1;
        bad.rows.sort_by_key(|r| (r.team_id, r.player_id));
        assert!(!valid(&bad), "six players on one team");
        bad = good.clone();
        bad.rows.last_mut().unwrap().team_id = 3;
        assert!(!valid(&bad), "third team");
        bad = good.clone();
        bad.rows[0].team_id = 0;
        assert!(!valid(&bad));
        bad = good.clone();
        bad.rows[0].player_id = 0;
        assert!(!valid(&bad));
        assert!(valid(&good));
    }
    #[test]
    fn incomplete_duplicate_invalid_and_wrong_audience_fail_closed() {
        for mode in 0..6 {
            let mut s = snapshot(3);
            match mode {
                0 => {s.public_events.pop();},
                1 => s.public_events.push(s.public_events[1].clone()),
                2 => s.public_events[1].sanitized_payload[12..20].copy_from_slice(&(-1i64).to_le_bytes()),
                3 => s.public_events[1].sanitized_payload[..4].copy_from_slice(&1u32.to_le_bytes()),
                4 => s.public_events[1].sanitized_payload[4..12].copy_from_slice(&SINGLE_LANE_SCORE_TEAM_METRIC_ID.to_le_bytes()),
                _ => s.public_events[0].sanitized_payload[12..20].copy_from_slice(&11i64.to_le_bytes()),
            }
            assert!(project(&s).is_none(),"mode {mode}");
        }
        assert!(project(&snapshot(0)).is_none());
    }
}
