#![allow(clippy::unnecessary_wraps, clippy::missing_errors_doc)]

use shared_domain::client_command::GameCommand;
use shared_domain::game_state::{GameState, GameTime};
use shared_domain::server_response::{AddressEnvelope, GameInfo, GameResponse, PlayerInfo};
use shared_domain::{BuildingInfo, GameId, PlayerId, TransportInfo};

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
        game_command: GameCommand,
    ) -> Result<Vec<GameResponseWithAddress>, GameResponse> {
        match game_command {
            GameCommand::PurchaseTransport(transport_info) => {
                self.process_purchase_transport(requesting_player_id, transport_info)
            },
            GameCommand::BuildBuildings(building_infos) => {
                self.process_build_buildings(requesting_player_id, building_infos)
            },
            GameCommand::QueryBuildings => self.process_query_buildings(requesting_player_id),
            GameCommand::QueryTransports => self.process_query_transports(requesting_player_id),
        }
    }

    fn process_query_transports(
        &mut self,
        requesting_player_id: PlayerId,
    ) -> Result<Vec<GameResponseWithAddress>, GameResponse> {
        Ok(vec![GameResponseWithAddress::new(
            AddressEnvelope::ToPlayer(requesting_player_id),
            GameResponse::TransportsExist(self.state.transport_infos()),
        )])
    }

    fn process_query_buildings(
        &mut self,
        requesting_player_id: PlayerId,
    ) -> Result<Vec<GameResponseWithAddress>, GameResponse> {
        Ok(vec![GameResponseWithAddress::new(
            AddressEnvelope::ToPlayer(requesting_player_id),
            GameResponse::BuildingsBuilt(self.state.building_infos()),
        )])
    }

    fn process_build_buildings(
        &mut self,
        requesting_player_id: PlayerId,
        building_infos: Vec<BuildingInfo>,
    ) -> Result<Vec<GameResponseWithAddress>, GameResponse> {
        match self
            .state
            .build_buildings(requesting_player_id, building_infos.clone())
        {
            Ok(()) => {
                Ok(vec![GameResponseWithAddress::new(
                    AddressEnvelope::ToAllPlayersInGame(self.game_id()),
                    GameResponse::BuildingsBuilt(building_infos),
                )])
            },
            Err(()) => {
                Err(GameResponse::CannotBuild(
                    building_infos
                        .into_iter()
                        .map(|building_info| building_info.building_id)
                        .collect(),
                ))
            },
        }
    }

    fn process_purchase_transport(
        &mut self,
        requesting_player_id: PlayerId,
        transport_info: TransportInfo,
    ) -> Result<Vec<GameResponseWithAddress>, GameResponse> {
        if requesting_player_id == transport_info.owner_id() {
            // TODO: Check if the track / road / etc. is free and owned by the purchaser
            // TODO: Subtract money

            self.state.insert_transport(transport_info.clone());
            Ok(vec![GameResponseWithAddress::new(
                AddressEnvelope::ToAllPlayersInGame(self.game_id()),
                GameResponse::TransportsExist(vec![transport_info]),
            )])
        } else {
            Err(GameResponse::CannotPurchase(transport_info.transport_id()))
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
    ) -> Result<Vec<GameResponseWithAddress>, GameResponse> {
        self.state.remove_player(player_id);
        Ok(vec![GameResponseWithAddress::new(
            AddressEnvelope::ToAllPlayersInGame(self.game_id()),
            GameResponse::GameLeft,
        )])
    }

    pub(crate) fn join_game(
        &mut self,
        requesting_player_info: PlayerInfo,
    ) -> Result<Vec<GameResponseWithAddress>, GameResponse> {
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
                GameResponse::PlayersUpdated(self.state.players()),
            ),
            GameResponseWithAddress::new(
                AddressEnvelope::ToPlayer(player_id),
                GameResponse::MapLevelProvided(self.state.map_level()),
            ),
        ])
    }

    pub(crate) fn sync(&self) -> Vec<GameResponseWithAddress> {
        if self.state.time_steps() % SYNC_EVERY_N_TIMESTEPS == 0 {
            vec![GameResponseWithAddress::new(
                AddressEnvelope::ToAllPlayersInGame(self.game_id()),
                GameResponse::TransportsSync(
                    self.state
                        .transport_infos()
                        .into_iter()
                        .map(|transport| (transport.id(), transport.dynamic_info()))
                        .collect(),
                ),
            )]
        } else {
            vec![]
        }
    }
}
