#![allow(clippy::unnecessary_wraps, clippy::missing_errors_doc)]

use shared_domain::building::building_info::WithBuildingDynamicInfo;
use shared_domain::building::industry_building_info::IndustryBuildingInfo;
use shared_domain::building::station_info::StationInfo;
use shared_domain::building::track_info::TrackInfo;
use shared_domain::client_command::{DemolishSelector, GameCommand};
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
            GameCommand::BuildIndustryBuilding(industry_building) => {
                self.process_build_industry_building(requesting_player_id, industry_building)
            },
            GameCommand::BuildStation(station) => {
                self.process_build_station(requesting_player_id, station)
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
            GameCommand::Demolish(demolish_selector) => {
                self.process_demolish(requesting_player_id, *demolish_selector)
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
        // TODO: This is inefficient, however, it is also unneeded - we already send `GameStateSnapshot` message - and then just ignore most of it!
        let mut results = vec![];
        for building in self.state.building_state().all_industry_buildings() {
            results.push(GameResponseWithAddress::new(
                AddressEnvelope::ToPlayer(requesting_player_id),
                GameResponse::IndustryBuildingAdded(building.clone()),
            ));
        }
        for building in self.state.building_state().all_stations() {
            results.push(GameResponseWithAddress::new(
                AddressEnvelope::ToPlayer(requesting_player_id),
                GameResponse::StationAdded(building.clone()),
            ));
        }
        Ok(results)
    }

    fn process_query_tracks(
        &mut self,
        requesting_player_id: PlayerId,
    ) -> Result<Vec<GameResponseWithAddress>, GameError> {
        Ok(vec![GameResponseWithAddress::new(
            AddressEnvelope::ToPlayer(requesting_player_id),
            GameResponse::TracksAdded(self.state.track_infos().clone()),
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
            .map_err(|()| GameError::CannotBuildIndustryBuilding(industry_building.id()))
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
            .map_err(|()| GameError::CannotBuildStation(station.id()))
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
            Err(()) => {
                Err(GameError::CannotBuildTracks(
                    track_infos.iter().map(TrackInfo::id).collect(),
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

    fn process_demolish(
        &mut self,
        requesting_player_id: PlayerId,
        demolish_selector: DemolishSelector,
    ) -> Result<Vec<GameResponseWithAddress>, GameError> {
        match demolish_selector {
            DemolishSelector::Track(track_id) => {
                self.state
                    .remove_track(requesting_player_id, track_id)
                    .map(|()| GameResponse::TrackRemoved(track_id))
            },
            DemolishSelector::Industry(industry_building_id) => {
                self.state
                    .remove_industry_building(requesting_player_id, industry_building_id)
                    .map(|()| GameResponse::IndustryBuildingRemoved(industry_building_id))
            },
            DemolishSelector::Station(station_id) => {
                self.state
                    .remove_station(requesting_player_id, station_id)
                    .map(|()| GameResponse::StationRemoved(station_id))
            },
        }
        .map(|success| {
            vec![GameResponseWithAddress::new(
                AddressEnvelope::ToAllPlayersInGame(self.game_id()),
                success,
            )]
        })
        .map_err(|()| GameError::CannotDemolish(demolish_selector))
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
        self.state.players().ids()
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
                GameResponse::PlayersUpdated(self.state.players().infos_cloned()),
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
                    .map(|building| (building.id(), building.dynamic_info().clone()))
                    .collect(),
                self.state
                    .building_state()
                    .all_stations()
                    .iter()
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
}
