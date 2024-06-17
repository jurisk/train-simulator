#![allow(clippy::unnecessary_wraps, clippy::missing_errors_doc)]

use std::collections::HashMap;

use shared_domain::client_command::GameCommand;
use shared_domain::map_level::MapLevel;
use shared_domain::server_response::{
    AddressEnvelope, Colour, GameInfo, GameResponse, PlayerInfo, ServerResponse,
    ServerResponseWithAddress,
};
use shared_domain::{BuildingInfo, GameId, PlayerId, PlayerName, VehicleInfo};

// TODO: This one should only return `Res<GameResponse, GameError>` to make everything simpler
#[derive(Debug, Clone)]
pub(crate) struct GameState {
    pub game_id: GameId,
    map_level:   MapLevel,
    // TODO:    Should some of this be in a `FieldXZ` instead of `Vec`? A set of multiple tracks can exist on a single tile.
    buildings:   Vec<BuildingInfo>,
    vehicles:    Vec<VehicleInfo>,
    players:     HashMap<PlayerId, PlayerInfo>,
}

impl GameState {
    pub(crate) fn new(
        map_level: MapLevel,
        buildings: Vec<BuildingInfo>,
        vehicles: Vec<VehicleInfo>,
        players: HashMap<PlayerId, PlayerInfo>,
    ) -> Self {
        let game_id = GameId::random();
        Self {
            game_id,
            map_level,
            buildings,
            vehicles,
            players,
        }
    }

    pub(crate) fn from_prototype(prototype: &GameState) -> Self {
        let game_id = GameId::random();
        Self {
            game_id,
            map_level: prototype.map_level.clone(),
            buildings: prototype.buildings.clone(),
            vehicles: prototype.vehicles.clone(),
            players: prototype.players.clone(),
        }
    }

    pub(crate) fn join_game(
        &mut self,
        requesting_player_id: PlayerId,
        requesting_player_name: PlayerName,
    ) -> Result<Vec<ServerResponseWithAddress>, ServerResponse> {
        // Later: Don't allow joining multiple games

        let colour = Colour::random();
        let requesting_player_info = PlayerInfo {
            id: requesting_player_id,
            name: requesting_player_name,
            colour,
        };

        self.players
            .insert(requesting_player_id, requesting_player_info);

        Ok(vec![
            ServerResponseWithAddress::new(
                AddressEnvelope::ToPlayer(requesting_player_id),
                ServerResponse::Game(self.game_id, GameResponse::GameJoined),
            ),
            ServerResponseWithAddress::new(
                AddressEnvelope::ToAllPlayersInGame(self.game_id),
                ServerResponse::Game(
                    self.game_id,
                    GameResponse::PlayersUpdated(self.players.clone()),
                ),
            ),
            ServerResponseWithAddress::new(
                AddressEnvelope::ToPlayer(requesting_player_id),
                ServerResponse::Game(
                    self.game_id,
                    GameResponse::MapLevelProvided(self.map_level.clone()),
                ),
            ),
        ])
    }

    pub(crate) fn process_command(
        &mut self,
        requesting_player_id: PlayerId,
        game_command: GameCommand,
    ) -> Result<Vec<ServerResponseWithAddress>, ServerResponse> {
        match game_command {
            GameCommand::PurchaseVehicle(vehicle_info) => {
                self.process_purchase_vehicle(requesting_player_id, vehicle_info)
            },
            GameCommand::BuildBuildings(building_infos) => {
                self.process_build_buildings(requesting_player_id, building_infos)
            },
            GameCommand::QueryBuildings => self.process_query_buildings(requesting_player_id),
        }
    }

    fn process_query_buildings(
        &mut self,
        requesting_player_id: PlayerId,
    ) -> Result<Vec<ServerResponseWithAddress>, ServerResponse> {
        Ok(vec![ServerResponseWithAddress::new(
            AddressEnvelope::ToPlayer(requesting_player_id),
            ServerResponse::Game(
                self.game_id,
                GameResponse::BuildingsBuilt(self.buildings.clone()),
            ),
        )])
    }

    fn process_build_buildings(
        &mut self,
        requesting_player_id: PlayerId,
        building_infos: Vec<BuildingInfo>,
    ) -> Result<Vec<ServerResponseWithAddress>, ServerResponse> {
        let valid_player_id = building_infos
            .iter()
            .all(|building_info| building_info.owner_id == requesting_player_id);

        // TODO: Check that this is a valid building and there is enough money to build it, subtract money
        // TODO: Check that terrain matches building requirements

        // Later: This is an inefficient check, but it will have to do for now
        let tiles_are_free = building_infos.iter().all(|building_infos| {
            building_infos
                .covers_tiles
                .to_set()
                .into_iter()
                .all(|tile| {
                    !self
                        .buildings
                        .iter()
                        .any(|building| building.covers_tiles.to_set().contains(&tile))
                })
        });

        if valid_player_id && tiles_are_free {
            self.buildings.append(&mut building_infos.clone());

            Ok(vec![ServerResponseWithAddress::new(
                AddressEnvelope::ToAllPlayersInGame(self.game_id),
                ServerResponse::Game(self.game_id, GameResponse::BuildingsBuilt(building_infos)),
            )])
        } else {
            Err(ServerResponse::Game(
                self.game_id,
                GameResponse::CannotBuild(
                    building_infos
                        .into_iter()
                        .map(|building_info| building_info.building_id)
                        .collect(),
                ),
            ))
        }
    }

    fn process_purchase_vehicle(
        &mut self,
        requesting_player_id: PlayerId,
        vehicle_info: VehicleInfo,
    ) -> Result<Vec<ServerResponseWithAddress>, ServerResponse> {
        if requesting_player_id == vehicle_info.owner_id {
            // TODO: Check if the track / road / etc. is free and owned by the purchaser
            // TODO: Subtract money

            self.vehicles.push(vehicle_info.clone());
            Ok(vec![ServerResponseWithAddress::new(
                AddressEnvelope::ToAllPlayersInGame(self.game_id),
                ServerResponse::Game(self.game_id, GameResponse::VehicleCreated(vehicle_info)),
            )])
        } else {
            Err(ServerResponse::Game(
                self.game_id,
                GameResponse::CannotPurchase(vehicle_info.vehicle_id),
            ))
        }
    }

    pub(crate) fn player_ids(&self) -> Vec<PlayerId> {
        self.players.keys().copied().collect()
    }

    pub(crate) fn create_game_info(&self) -> GameInfo {
        GameInfo {
            game_id: self.game_id,
            players: self.players.clone(),
        }
    }

    pub(crate) fn remove_player(
        &mut self,
        player_id: PlayerId,
    ) -> Result<Vec<ServerResponseWithAddress>, ServerResponse> {
        self.players.remove(&player_id);
        Ok(vec![ServerResponseWithAddress::new(
            AddressEnvelope::ToAllPlayersInGame(self.game_id),
            ServerResponse::Game(self.game_id, GameResponse::GameLeft),
        )])
    }
}
