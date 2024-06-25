#![allow(clippy::missing_errors_doc, clippy::result_unit_err)]

use std::collections::{HashMap, HashSet};

use pathfinding::prelude::dijkstra;
use serde::{Deserialize, Serialize};
use shared_util::direction_xz::DirectionXZ;

use crate::building_info::BuildingInfo;
use crate::building_state::BuildingState;
use crate::edge_xz::EdgeXZ;
use crate::map_level::MapLevel;
use crate::server_response::{GameInfo, PlayerInfo};
use crate::tile_coverage::TileCoverage;
use crate::track_type::TrackType;
use crate::transport_info::{TransportDynamicInfo, TransportInfo};
use crate::{BuildingId, BuildingType, GameId, PlayerId, TileCoordsXZ, TransportId};

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
        // TODO: Actually get the EdgeXZ that was closest to the mouse when selecting!
        let head_tile = *ordered_selected_tiles.first()?;
        let head = EdgeXZ::from_tile_and_direction(head_tile, DirectionXZ::North);
        // TODO: Actually get the EdgeXZ that was closest to the mouse when selecting!
        let tail_tile = *ordered_selected_tiles.last()?;
        let tail = EdgeXZ::from_tile_and_direction(tail_tile, DirectionXZ::East);
        let preferred_tiles: HashSet<TileCoordsXZ> =
            ordered_selected_tiles.iter().copied().collect();
        const PREFERRED_TILE_BONUS: u32 = 16; // How much shorter "length" do we assign to going through a preferred tile

        // Later: Consider switching to `a_star` or `dijkstra_all`
        let path: Option<(Vec<EdgeXZ>, u32)> = dijkstra(
            &head,
            |&edge| {
                // TODO: Is this even within bounds? Above water?
                // TODO: Is it free? Use `BuildingState::can_build_building`.

                edge.ordered_tiles().into_iter().flat_map(move |tile| {
                    EdgeXZ::for_tile(tile)
                        .into_iter()
                        .filter(|neighbour| *neighbour != edge)
                        .map(|neighbour| {
                            // TODO: Can we actually even build such a track there, are the vertex heights compatible?
                            let length = 1;
                            // TODO: Figure out the preferred tiles thing
                            // let length = if preferred_tiles.contains(&tile) {
                            //     1
                            // } else {
                            //     PREFERRED_TILE_BONUS
                            // };
                            // TODO: Shorter tracks are faster?
                            (neighbour, length)
                        })
                        .collect::<Vec<_>>()
                })
            },
            |edge| *edge == tail,
        );

        path.map(|(path, _length)| {
            let buildings = path
                .windows(2)
                .flat_map(|w| {
                    let a = w[0];
                    let b = w[1];

                    EdgeXZ::common_tile(a, b)
                        .into_iter()
                        .flat_map(|tile| {
                            TrackType::all()
                                .into_iter()
                                .flat_map(|track_type| {
                                    let (da, db) = track_type.connections_clockwise();
                                    let ea = EdgeXZ::from_tile_and_direction(tile, da);
                                    let eb = EdgeXZ::from_tile_and_direction(tile, db);
                                    // This track fits!
                                    if (ea == a && eb == b) || (ea == b && eb == a) {
                                        vec![BuildingInfo {
                                            owner_id:      player_id,
                                            building_id:   BuildingId::random(),
                                            covers_tiles:  TileCoverage::Single(tile),
                                            building_type: BuildingType::Track(track_type),
                                        }]
                                    } else {
                                        vec![]
                                    }
                                })
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();

            buildings
        })
    }
}
