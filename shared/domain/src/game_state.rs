#![allow(clippy::missing_errors_doc, clippy::result_unit_err)]

use std::collections::{HashMap, HashSet};

use pathfinding::prelude::dijkstra;
use serde::{Deserialize, Serialize};
use shared_util::direction_xz::DirectionXZ;
use shared_util::random::choose_unsafe;

use crate::building_info::BuildingInfo;
use crate::building_state::BuildingState;
use crate::map_level::MapLevel;
use crate::server_response::{GameInfo, PlayerInfo};
use crate::tile_coverage::TileCoverage;
use crate::transport_info::{TransportDynamicInfo, TransportInfo};
use crate::{BuildingId, BuildingType, GameId, PlayerId, TileCoordsXZ, TrackType, TransportId};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default, PartialEq)]
pub struct GameTime(pub f32);

impl GameTime {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

// Later:   So this is used both on the server (to store authoritative game state), and on the client (to store the game state as known by the client).
//          So the API gets quite busy because of this. There may be better ways.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct GameState {
    game_id:    GameId,
    map_level:  MapLevel,
    buildings:  BuildingState,
    transports: Vec<TransportInfo>,
    players:    HashMap<PlayerId, PlayerInfo>,
    time:       GameTime,
    time_steps: u64,
}

impl GameState {
    #[must_use]
    pub fn new(
        map_level: MapLevel,
        buildings: Vec<BuildingInfo>,
        transports: Vec<TransportInfo>,
        players: HashMap<PlayerId, PlayerInfo>,
    ) -> Self {
        let game_id = GameId::random();
        Self {
            game_id,
            map_level,
            buildings: BuildingState::from_vec(buildings),
            transports,
            players,
            time: GameTime::new(),
            time_steps: 0,
        }
    }

    #[must_use]
    pub fn from_prototype(prototype: &GameState) -> Self {
        let game_id = GameId::random();
        Self {
            game_id,
            map_level: prototype.map_level.clone(),
            buildings: prototype.buildings.clone(),
            transports: prototype.transports.clone(),
            players: prototype.players.clone(),
            time: prototype.time,
            time_steps: prototype.time_steps,
        }
    }

    pub fn advance_time(&mut self, time: GameTime) {
        let diff = time.0 - self.time.0;
        for transport in &mut self.transports {
            // Later: If game is paused then no need to advance transports
            transport.advance(diff, &self.buildings);
        }
        self.time = time;
        self.time_steps += 1;
    }

    #[must_use]
    pub fn player_ids(&self) -> Vec<PlayerId> {
        self.players.keys().copied().collect()
    }

    #[must_use]
    pub fn create_game_info(&self) -> GameInfo {
        GameInfo {
            game_id: self.game_id,
            players: self.players.clone(),
        }
    }

    #[must_use]
    pub fn game_id(&self) -> GameId {
        self.game_id
    }

    #[must_use]
    pub fn time_steps(&self) -> u64 {
        self.time_steps
    }

    #[must_use]
    pub fn transport_infos(&self) -> Vec<TransportInfo> {
        self.transports.clone()
    }

    #[must_use]
    pub fn building_infos(&self) -> Vec<BuildingInfo> {
        self.buildings.to_vec()
    }

    #[must_use]
    pub fn map_level(&self) -> &MapLevel {
        &self.map_level
    }

    #[must_use]
    pub fn players(&self) -> &HashMap<PlayerId, PlayerInfo> {
        &self.players
    }

    pub fn update_player_infos(&mut self, new_player_infos: &HashMap<PlayerId, PlayerInfo>) {
        self.players.clone_from(new_player_infos);
    }

    pub fn insert_player(&mut self, player_info: PlayerInfo) {
        self.players.insert(player_info.id, player_info);
    }

    pub fn remove_player(&mut self, player_id: PlayerId) {
        self.players.remove(&player_id);
    }

    pub fn upsert_transport(&mut self, transport: TransportInfo) {
        let transport_id = transport.id();
        if let Some(existing_transport) =
            self.transports.iter_mut().find(|t| t.id() == transport_id)
        {
            existing_transport.clone_from(&transport);
        } else {
            self.transports.push(transport);
        }
    }

    pub fn append_buildings(&mut self, buildings: Vec<BuildingInfo>) {
        self.buildings.append_all(buildings);
    }

    pub fn build_buildings(
        &mut self,
        requesting_player_id: PlayerId,
        buildings: Vec<BuildingInfo>,
    ) -> Result<(), ()> {
        self.buildings.build(requesting_player_id, buildings)
    }

    #[must_use]
    pub fn building_state(&self) -> &BuildingState {
        &self.buildings
    }

    pub fn update_transport_dynamic_info(
        &mut self,
        transport_id: TransportId,
        transport_dynamic_info: &TransportDynamicInfo,
    ) {
        for transport in &mut self.transports {
            if transport.id() == transport_id {
                transport.update_dynamic_info(transport_dynamic_info);
                return;
            }
        }
    }

    #[must_use]
    pub fn get_transport_info(&self, transport_id: TransportId) -> Option<&TransportInfo> {
        self.transports
            .iter()
            .find(|transport| transport.id() == transport_id)
    }

    // TODO HIGH: We have a disconnect here, as our pathfinding kind of works with tiles, but our tracks with tile edges...
    #[allow(clippy::items_after_statements)]
    #[must_use]
    pub fn plan_track(
        &self,
        player_id: PlayerId,
        ordered_selected_tiles: &[TileCoordsXZ],
    ) -> Option<Vec<BuildingInfo>> {
        let head = *ordered_selected_tiles.first()?;
        let tail = *ordered_selected_tiles.last()?;
        let preferred_tiles: HashSet<TileCoordsXZ> =
            ordered_selected_tiles.iter().copied().collect();
        const PREFERRED_TILE_BONUS: u32 = 4; // How much shorter "length" do we assign to going through a preferred tile

        // Later: Consider switching to `a_star`
        let path: Option<(Vec<TileCoordsXZ>, u32)> = dijkstra(
            &head,
            |tile| {
                DirectionXZ::cardinal()
                    .into_iter()
                    .map(|direction| {
                        // TODO: Is this even within bounds? Above water?
                        let neighbour = *tile + direction;
                        // TODO: Is it free? Use `BuildingState::can_build_building`.
                        let length = if preferred_tiles.contains(&neighbour) {
                            1
                        } else {
                            PREFERRED_TILE_BONUS
                        };
                        (neighbour, length)
                    })
                    .collect::<Vec<_>>()
            },
            |tile| *tile == tail,
        );

        path.map(|(path, _length)| {
            let buildings = path
                .iter()
                .map(|tile| {
                    let track_options = &TrackType::all();
                    let tmp_selected_track = choose_unsafe(track_options);

                    BuildingInfo {
                        owner_id:      player_id,
                        building_id:   BuildingId::random(),
                        covers_tiles:  TileCoverage::Single(*tile),
                        building_type: BuildingType::Track(*tmp_selected_track),
                    }
                })
                .collect();

            buildings
        })
    }
}
