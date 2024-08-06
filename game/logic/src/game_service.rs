#![allow(clippy::unnecessary_wraps, clippy::missing_errors_doc)]

use shared_domain::building::building_info::{BuildingInfo, WithBuildingDynamicInfo};
use shared_domain::building::track_info::TrackInfo;
use shared_domain::client_command::GameCommand;
use shared_domain::game_state::GameState;
use shared_domain::game_time::GameTime;
use shared_domain::server_response::{
    AddressEnvelope, GameError, GameInfo, GameResponse, PlayerInfo,
};
use shared_domain::transport::movement_orders::MovementOrders;
use shared_domain::transport::transport_info::TransportInfo;
use shared_domain::{GameId, PlayerId, TransportId};

#[derive(Clone)]
pub(crate) struct GameResponseWithAddress {
    pub address:  AddressEnvelope,
    pub response: GameResponse,
}

impl GameResponseWithAddress {
    fn new(address: AddressEnvelope, response: GameResponse) -> Self {
        Self { address, response }
    }
}

pub struct GameService {
    state: GameState,
}

const SYNC_EVERY_N_TIMESTEPS: u64 = 100;

impl GameService {
    #[must_use]
    pub fn from_prototype(prototype: &GameState) -> Self {
        Self {
            state: GameState::from_prototype(prototype),
        }
    }

    #[must_use]
    pub fn game_id(&self) -> GameId {
        self.state.game_id()
    }

    pub(crate) fn process_command(
        &mut self,
        requesting_player_id: PlayerId,
        game_command: &GameCommand,
    ) -> Result<Vec<GameResponseWithAddress>, GameError> {
        match game_command {
            GameCommand::PurchaseTransport(transport_info) => {
                self.process_purchase_transport(requesting_player_id, transport_info)
            },
            GameCommand::BuildIndustryBuildings(industry_buildings) => {
                self.process_build_industry_buildings(requesting_player_id, industry_buildings)
            },
            GameCommand::BuildStations(stations) => {
                self.process_build_stations(requesting_player_id, stations)
            },
            GameCommand::BuildTracks(track_infos) => {
                self.process_build_tracks(requesting_player_id, track_infos)
            },
            GameCommand::QueryBuildings => self.process_query_buildings(requesting_player_id),
            GameCommand::QueryTracks => self.process_query_tracks(requesting_player_id),
            GameCommand::QueryTransports => self.process_query_transports(requesting_player_id),
            GameCommand::UpdateTransportMovementOrders(transport_id, movement_orders) => {
                self.process_update_transport_movement_orders(
                    requesting_player_id,
                    *transport_id,
                    movement_orders,
                )
            },
        }
    }

    fn process_query_transports(
        &mut self,
        requesting_player_id: PlayerId,
    ) -> Result<Vec<GameResponseWithAddress>, GameError> {
        Ok(vec![GameResponseWithAddress::new(
            AddressEnvelope::ToPlayer(requesting_player_id),
            GameResponse::TransportsAdded(self.state.transport_infos().clone()),
        )])
    }

    fn process_query_buildings(
        &mut self,
        requesting_player_id: PlayerId,
    ) -> Result<Vec<GameResponseWithAddress>, GameError> {
        Ok(vec![
            GameResponseWithAddress::new(
                AddressEnvelope::ToPlayer(requesting_player_id),
                GameResponse::IndustryBuildingsAdded(
                    self.state.building_state().all_industry_buildings().clone(),
                ),
            ),
            GameResponseWithAddress::new(
                AddressEnvelope::ToPlayer(requesting_player_id),
                GameResponse::StationsAdded(self.state.building_state().all_stations().clone()),
            ),
        ])
    }

    fn process_query_tracks(
        &mut self,
        requesting_player_id: PlayerId,
    ) -> Result<Vec<GameResponseWithAddress>, GameError> {
        Ok(vec![GameResponseWithAddress::new(
            AddressEnvelope::ToPlayer(requesting_player_id),
            GameResponse::TracksAdded(self.state.track_infos()),
        )])
    }

    fn process_build_industry_buildings(
        &mut self,
        requesting_player_id: PlayerId,
        industry_buildings: &[BuildingInfo],
    ) -> Result<Vec<GameResponseWithAddress>, GameError> {
        self.state
            .build_industry_buildings(requesting_player_id, industry_buildings)
            .map(|()| {
                vec![GameResponseWithAddress::new(
                    AddressEnvelope::ToAllPlayersInGame(self.game_id()),
                    GameResponse::IndustryBuildingsAdded(industry_buildings.to_vec()),
                )]
            })
            .map_err(|()| {
                GameError::CannotBuildBuildings(
                    industry_buildings
                        .iter()
                        .map(BuildingInfo::building_id)
                        .collect(),
                )
            })
    }

    fn process_build_stations(
        &mut self,
        requesting_player_id: PlayerId,
        stations: &[BuildingInfo],
    ) -> Result<Vec<GameResponseWithAddress>, GameError> {
        self.state
            .build_stations(requesting_player_id, stations)
            .map(|()| {
                vec![GameResponseWithAddress::new(
                    AddressEnvelope::ToAllPlayersInGame(self.game_id()),
                    GameResponse::StationsAdded(stations.to_vec()),
                )]
            })
            .map_err(|()| {
                GameError::CannotBuildBuildings(
                    stations.iter().map(BuildingInfo::building_id).collect(),
                )
            })
    }

    fn process_build_tracks(
        &mut self,
        requesting_player_id: PlayerId,
        track_infos: &[TrackInfo],
    ) -> Result<Vec<GameResponseWithAddress>, GameError> {
        match self.state.build_tracks(requesting_player_id, track_infos) {
            Ok(()) => {
                Ok(vec![GameResponseWithAddress::new(
                    AddressEnvelope::ToAllPlayersInGame(self.game_id()),
                    GameResponse::TracksAdded(track_infos.to_vec()),
                )])
            },
            Err(()) => {
                Err(GameError::CannotBuildTracks(
                    track_infos
                        .iter()
                        .map(|track_info| track_info.track_id)
                        .collect(),
                ))
            },
        }
    }

    fn process_purchase_transport(
        &mut self,
        requesting_player_id: PlayerId,
        transport_info: &TransportInfo,
    ) -> Result<Vec<GameResponseWithAddress>, GameError> {
        if requesting_player_id == transport_info.owner_id() {
            // TODO: Check if the track / road / etc. is free and owned by the purchaser
            // TODO: Subtract money
            // TODO: But do it all within the `GameState`

            self.state.upsert_transport(transport_info.clone());
            Ok(vec![GameResponseWithAddress::new(
                AddressEnvelope::ToAllPlayersInGame(self.game_id()),
                GameResponse::TransportsAdded(vec![transport_info.clone()]),
            )])
        } else {
            Err(GameError::CannotPurchase(transport_info.transport_id()))
        }
    }

    fn process_update_transport_movement_orders(
        &mut self,
        _requesting_player_id: PlayerId,
        transport_id: TransportId,
        movement_orders: &MovementOrders,
    ) -> Result<Vec<GameResponseWithAddress>, GameError> {
        // TODO: Check this is the right player

        match self
            .state
            .update_transport_movement_orders(transport_id, movement_orders)
        {
            Err(()) => Err(GameError::UnspecifiedError),
            Ok(()) => Ok(vec![self.create_dynamic_info_sync()]),
        }
    }

    pub(crate) fn advance_time(&mut self, time: GameTime) {
        self.state.advance_time(time);
    }

    pub(crate) fn create_game_info(&self) -> GameInfo {
        self.state.create_game_info()
    }

    pub(crate) fn player_ids(&self) -> Vec<PlayerId> {
        self.state.player_ids()
    }

    pub(crate) fn remove_player(
        &mut self,
        player_id: PlayerId,
    ) -> Result<Vec<GameResponseWithAddress>, GameError> {
        self.state.remove_player(player_id);
        Ok(vec![GameResponseWithAddress::new(
            AddressEnvelope::ToAllPlayersInGame(self.game_id()),
            GameResponse::GameLeft,
        )])
    }

    pub(crate) fn join_game(
        &mut self,
        requesting_player_info: PlayerInfo,
    ) -> Result<Vec<GameResponseWithAddress>, GameError> {
        // Later: Don't allow joining multiple games

        let player_id = requesting_player_info.id;
        self.state.insert_player(requesting_player_info);

        Ok(vec![
            GameResponseWithAddress::new(
                AddressEnvelope::ToPlayer(player_id),
                GameResponse::GameJoined,
            ),
            GameResponseWithAddress::new(
                AddressEnvelope::ToAllPlayersInGame(self.game_id()),
                GameResponse::PlayersUpdated(self.state.players().clone()),
            ),
            GameResponseWithAddress::new(
                AddressEnvelope::ToPlayer(player_id),
                GameResponse::GameStateSnapshot(self.state.clone()),
            ),
        ])
    }

    pub(crate) fn sync(&self) -> Vec<GameResponseWithAddress> {
        if self.state.time_steps() % SYNC_EVERY_N_TIMESTEPS == 0 {
            vec![self.create_dynamic_info_sync()]
        } else {
            vec![]
        }
    }

    fn create_dynamic_info_sync(&self) -> GameResponseWithAddress {
        GameResponseWithAddress::new(
            AddressEnvelope::ToAllPlayersInGame(self.game_id()),
            GameResponse::DynamicInfosSync(
                self.state.time(),
                self.state
                    .building_state()
                    .all_industry_buildings()
                    .iter()
                    .map(|building| (building.building_id(), building.dynamic_info().clone()))
                    .collect(),
                self.state
                    .building_state()
                    .all_stations()
                    .iter()
                    .map(|building| (building.building_id(), building.dynamic_info().clone()))
                    .collect(),
                self.state
                    .transport_infos()
                    .iter()
                    .map(|transport| (transport.transport_id(), transport.dynamic_info().clone()))
                    .collect(),
            ),
        )
    }
}
