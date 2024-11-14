#![allow(clippy::unnecessary_wraps, clippy::missing_errors_doc)]

use bimap::BiMap;
use shared_domain::building::building_info::{WithBuildingDynamicInfo, WithOwner};
use shared_domain::building::industry_building_info::IndustryBuildingInfo;
use shared_domain::building::military_building_info::MilitaryBuildingInfo;
use shared_domain::building::station_info::StationInfo;
use shared_domain::building::track_info::TrackInfo;
use shared_domain::client_command::{DemolishSelector, GameCommand};
use shared_domain::game_state::GameState;
use shared_domain::game_time::GameTimeDiff;
use shared_domain::metrics::Metrics;
use shared_domain::scenario::Scenario;
use shared_domain::server_response::{
    AddressEnvelope, GameError, GameInfo, GameResponse, UserInfo,
};
use shared_domain::transport::movement_orders::MovementOrders;
use shared_domain::transport::transport_info::TransportInfo;
use shared_domain::{GameId, PlayerId, StationId, TransportId, UserId};

// Public only for tests
#[derive(Clone, Debug)]
pub struct GameResponseWithAddress {
    pub address:  AddressEnvelope,
    pub response: GameResponse,
}

impl GameResponseWithAddress {
    fn new(address: AddressEnvelope, response: GameResponse) -> Self {
        Self { address, response }
    }
}

pub struct GameService {
    state:        GameState,
    user_players: BiMap<UserId, PlayerId>,
}

const SYNC_EVERY_N_TIMESTEPS: u64 = 100;

impl GameService {
    #[must_use]
    pub fn from_prototype(scenario: &Scenario, ignore_requesting_player_id: bool) -> Self {
        Self {
            state:        GameState::from_scenario(scenario.clone(), ignore_requesting_player_id),
            user_players: BiMap::new(),
        }
    }

    #[must_use]
    pub fn from_game_state(game_state: GameState) -> Self {
        Self {
            state:        game_state,
            user_players: BiMap::new(),
        }
    }

    #[must_use]
    pub fn game_id(&self) -> GameId {
        self.state.game_id()
    }

    pub(crate) fn user_ids_for_player(&self, player_id: PlayerId) -> Vec<UserId> {
        self.user_players
            .get_by_right(&player_id)
            .map(|user_id| vec![*user_id])
            .unwrap_or_default()
    }

    pub(crate) fn player_id_for_user_id(&self, user_id: UserId) -> Option<PlayerId> {
        self.user_players.get_by_left(&user_id).copied()
    }

    // Public only for tests
    pub fn process_command(
        &mut self,
        requesting_player_id: PlayerId,
        game_command: &GameCommand,
    ) -> Result<Vec<GameResponseWithAddress>, GameError> {
        match game_command {
            GameCommand::PurchaseTransport(station_id, transport_info) => {
                self.process_purchase_transport(requesting_player_id, *station_id, transport_info)
            },
            GameCommand::BuildIndustryBuilding(industry_building) => {
                self.process_build_industry_building(requesting_player_id, industry_building)
            },
            GameCommand::BuildStation(station) => {
                self.process_build_station(requesting_player_id, station)
            },
            GameCommand::BuildMilitaryBuilding(military_building) => {
                self.process_build_military_building(requesting_player_id, military_building)
            },
            GameCommand::BuildTracks(track_infos) => {
                self.process_build_tracks(requesting_player_id, track_infos)
            },
            GameCommand::UpdateTransportMovementOrders(transport_id, movement_orders) => {
                self.process_update_transport_movement_orders(
                    requesting_player_id,
                    *transport_id,
                    movement_orders,
                )
            },
            GameCommand::Demolish(demolish_selector) => {
                self.process_demolish(requesting_player_id, demolish_selector)
            },
            GameCommand::RequestGameStateSnapshot => {
                self.request_game_state_snapshot(requesting_player_id)
            },
        }
    }

    fn request_game_state_snapshot(
        &self,
        requesting_player_id: PlayerId,
    ) -> Result<Vec<GameResponseWithAddress>, GameError> {
        Ok(vec![GameResponseWithAddress::new(
            AddressEnvelope::ToPlayer(self.game_id(), requesting_player_id),
            GameResponse::GameStateSnapshot(self.state.clone()),
        )])
    }

    fn process_build_industry_building(
        &mut self,
        requesting_player_id: PlayerId,
        industry_building: &IndustryBuildingInfo,
    ) -> Result<Vec<GameResponseWithAddress>, GameError> {
        self.state
            .build_industry_building(requesting_player_id, industry_building)
            .map(|()| {
                vec![GameResponseWithAddress::new(
                    AddressEnvelope::ToAllPlayersInGame(self.game_id()),
                    GameResponse::IndustryBuildingAdded(industry_building.clone()),
                )]
            })
            .map_err(|error| GameError::CannotBuildIndustryBuilding(industry_building.id(), error))
    }

    fn process_build_station(
        &mut self,
        requesting_player_id: PlayerId,
        station: &StationInfo,
    ) -> Result<Vec<GameResponseWithAddress>, GameError> {
        self.state
            .build_station(requesting_player_id, station)
            .map(|()| {
                vec![GameResponseWithAddress::new(
                    AddressEnvelope::ToAllPlayersInGame(self.game_id()),
                    GameResponse::StationAdded(station.clone()),
                )]
            })
            .map_err(|error| GameError::CannotBuildStation(station.id(), error))
    }

    fn process_build_military_building(
        &mut self,
        requesting_player_id: PlayerId,
        military_building: &MilitaryBuildingInfo,
    ) -> Result<Vec<GameResponseWithAddress>, GameError> {
        self.state
            .build_military_building(requesting_player_id, military_building)
            .map(|()| {
                vec![GameResponseWithAddress::new(
                    AddressEnvelope::ToAllPlayersInGame(self.game_id()),
                    GameResponse::MilitaryBuildingAdded(military_building.clone()),
                )]
            })
            .map_err(|error| GameError::CannotBuildMilitaryBuilding(military_building.id(), error))
    }

    fn process_build_tracks(
        &mut self,
        requesting_player_id: PlayerId,
        track_infos: &[TrackInfo],
    ) -> Result<Vec<GameResponseWithAddress>, GameError> {
        match self.state.build_tracks(requesting_player_id, track_infos) {
            Ok(built) => {
                Ok(vec![GameResponseWithAddress::new(
                    AddressEnvelope::ToAllPlayersInGame(self.game_id()),
                    GameResponse::TracksAdded(built),
                )])
            },
            Err(error) => {
                Err(GameError::CannotBuildTracks(
                    track_infos.iter().map(TrackInfo::id).collect(),
                    error,
                ))
            },
        }
    }

    fn process_purchase_transport(
        &mut self,
        requesting_player_id: PlayerId,
        station_id: StationId,
        transport_info: &TransportInfo,
    ) -> Result<Vec<GameResponseWithAddress>, GameError> {
        match self
            .state
            .purchase_transport(requesting_player_id, station_id, transport_info)
        {
            Ok(()) => {
                Ok(vec![GameResponseWithAddress::new(
                    AddressEnvelope::ToAllPlayersInGame(self.game_id()),
                    GameResponse::TransportsAdded(vec![transport_info.clone()]),
                )])
            },
            Err(error) => {
                Err(GameError::CannotPurchaseTransport(
                    transport_info.transport_id(),
                    error,
                ))
            },
        }
    }

    fn process_demolish(
        &mut self,
        requesting_player_id: PlayerId,
        demolish_selector: &DemolishSelector,
    ) -> Result<Vec<GameResponseWithAddress>, GameError> {
        match demolish_selector {
            DemolishSelector::Tracks(track_ids) => {
                self.state
                    .remove_tracks(requesting_player_id, track_ids)
                    .map(|()| GameResponse::TracksRemoved(track_ids.clone()))
            },
            DemolishSelector::Industry(industry_building_id) => {
                self.state
                    .remove_industry_building(requesting_player_id, *industry_building_id)
                    .map(|()| GameResponse::IndustryBuildingRemoved(*industry_building_id))
            },
            DemolishSelector::Station(station_id) => {
                self.state
                    .remove_station(requesting_player_id, *station_id)
                    .map(|()| GameResponse::StationRemoved(*station_id))
            },
            DemolishSelector::MilitaryBuilding(military_building_id) => {
                self.state
                    .remove_military_building(requesting_player_id, *military_building_id)
                    .map(|()| GameResponse::MilitaryBuildingRemoved(*military_building_id))
            },
        }
        .map(|success| {
            vec![GameResponseWithAddress::new(
                AddressEnvelope::ToAllPlayersInGame(self.game_id()),
                success,
            )]
        })
        .map_err(|()| GameError::CannotDemolish(demolish_selector.clone()))
    }

    fn process_update_transport_movement_orders(
        &mut self,
        requesting_player_id: PlayerId,
        transport_id: TransportId,
        movement_orders: &MovementOrders,
    ) -> Result<Vec<GameResponseWithAddress>, GameError> {
        let transport = self
            .state
            .transport_state()
            .info_by_id(transport_id)
            .ok_or(GameError::UnspecifiedError)?;
        if transport.owner_id() == requesting_player_id {
            match self
                .state
                .update_transport_movement_orders(transport_id, movement_orders)
            {
                Err(()) => Err(GameError::UnspecifiedError),
                Ok(()) => Ok(vec![self.create_dynamic_info_sync()]),
            }
        } else {
            Err(GameError::UnspecifiedError)
        }
    }

    pub fn advance_time_diff(&mut self, diff: GameTimeDiff, metrics: &impl Metrics) {
        self.state.advance_time_diff(diff, metrics);
    }

    #[must_use]
    pub fn create_game_info(&self) -> GameInfo {
        GameInfo {
            scenario_id:  self.state.scenario_id(),
            game_id:      self.state.game_id(),
            players:      self.state.players().infos_cloned(),
            user_players: self.user_players_vec(),
        }
    }

    pub(crate) fn player_ids(&self) -> Vec<PlayerId> {
        self.state.players().ids()
    }

    pub(crate) fn remove_player(
        &mut self,
        user_id: UserId,
    ) -> Result<Vec<GameResponseWithAddress>, GameError> {
        if let Some((user_id, _)) = self.user_players.remove_by_left(&user_id) {
            Ok(vec![
                GameResponseWithAddress::new(
                    AddressEnvelope::ToUser(user_id),
                    GameResponse::GameLeft,
                ),
                GameResponseWithAddress::new(
                    AddressEnvelope::ToAllPlayersInGame(self.game_id()),
                    GameResponse::PlayersUpdated(self.user_players_vec()),
                ),
            ])
        } else {
            Err(GameError::UnspecifiedError)
        }
    }

    fn first_free_player_id(&self) -> Option<PlayerId> {
        self.state
            .players()
            .ids()
            .into_iter()
            .find(|&player_id| !self.user_players.contains_right(&player_id))
    }

    pub(crate) fn join_game(
        &mut self,
        requesting_user_info: &UserInfo,
        player_id: Option<PlayerId>,
    ) -> Result<Vec<GameResponseWithAddress>, GameError> {
        // Later: Don't allow joining multiple games at once

        let user_id = requesting_user_info.id;
        let player_id = match player_id {
            None => self.first_free_player_id(),
            Some(player_id) => Some(player_id),
        };
        if let Some(player_id) = player_id {
            self.user_players.insert(user_id, player_id);

            Ok(vec![
                GameResponseWithAddress::new(
                    AddressEnvelope::ToUser(user_id),
                    GameResponse::GameJoined(player_id, self.state.clone()),
                ),
                GameResponseWithAddress::new(
                    AddressEnvelope::ToAllPlayersInGame(self.game_id()),
                    GameResponse::PlayersUpdated(self.user_players_vec()),
                ),
            ])
        } else {
            Err(GameError::UnspecifiedError)
        }
    }

    fn user_players_vec(&self) -> Vec<(UserId, PlayerId)> {
        self.user_players
            .iter()
            .map(|(user_id, player_id)| (*user_id, *player_id))
            .collect()
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
                    .into_iter()
                    .map(|building| (building.id(), building.dynamic_info().clone()))
                    .collect(),
                self.state
                    .building_state()
                    .all_stations()
                    .into_iter()
                    .map(|building| (building.id(), building.dynamic_info().clone()))
                    .collect(),
                self.state
                    .transport_infos()
                    .iter()
                    .map(|transport| (transport.transport_id(), transport.dynamic_info().clone()))
                    .collect(),
            ),
        )
    }

    // Hack used just for tests
    #[must_use]
    pub fn game_state(&self) -> &GameState {
        &self.state
    }
}
