#![expect(clippy::module_name_repetitions)]

use std::collections::HashMap;

use log::debug;
use shared_domain::building::industry_building_info::IndustryBuildingInfo;
use shared_domain::building::industry_type::IndustryType;
use shared_domain::building::station_info::StationInfo;
use shared_domain::building::station_type::{StationOrientation, StationType};
use shared_domain::client_command::GameCommand;
use shared_domain::game_state::GameState;
use shared_domain::metrics::Metrics;
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_domain::{IndustryBuildingId, PlayerId, StationId};
use shared_util::direction_xz::DirectionXZ;
use shared_util::random::choose;

use crate::ArtificialIntelligenceState;

#[derive(Copy, Clone)]
enum Goal {
    SteelToConstructionYard(IndustryBuildingId),
    TimberToConstructionYard(IndustryBuildingId),
    ConcreteToConstructionYard(IndustryBuildingId),
}

pub struct Oct2025ArtificialIntelligenceState {
    player_id:     PlayerId,
    pending_goals: Vec<Goal>,
}

impl Oct2025ArtificialIntelligenceState {
    #[must_use]
    #[expect(clippy::missing_panics_doc)]
    pub fn new(player_id: PlayerId, game_state: &GameState) -> Self {
        let construction_yards = game_state
            .building_state()
            .find_industry_building_by_owner_and_type(player_id, IndustryType::ConstructionYard);
        assert_eq!(
            construction_yards.len(),
            1,
            "Expected exactly one construction yard for player {player_id}"
        );
        let construction_yard = construction_yards[0];
        let construction_yard_id = construction_yard.id();
        Self {
            player_id,
            pending_goals: vec![
                Goal::SteelToConstructionYard(construction_yard_id),
                Goal::TimberToConstructionYard(construction_yard_id),
                Goal::ConcreteToConstructionYard(construction_yard_id),
            ],
        }
    }
}

impl Oct2025ArtificialIntelligenceState {
    fn select_industry_building(
        &self,
        game_state: &GameState,
        industry_type: IndustryType,
        reference_tile: TileCoordsXZ,
    ) -> Option<IndustryBuildingInfo> {
        let free = game_state.all_free_zonings();

        let found = free
            .iter()
            .filter(|zoning| Some(zoning.zoning_type()) == industry_type.required_zoning())
            .map(|zoning| {
                IndustryBuildingInfo::new(
                    self.player_id,
                    IndustryBuildingId::random(),
                    zoning.reference_tile(),
                    industry_type,
                )
            })
            .filter(|info| {
                game_state
                    .can_build_industry_building(self.player_id, info)
                    .is_ok()
            })
            .min_by_key(|info| info.reference_tile().manhattan_distance(reference_tile));

        // TODO: If industry has no zoning requirement, build in an empty space, but choose the best place - closest to the industries for its inputs/outputs, or even just closest to ConstructionYard.
        if let Some(info) = found {
            Some(info.clone())
        } else {
            debug!("No free zoning for {:?}", industry_type);
            None
        }
    }

    fn select_station_building(
        &self,
        game_state: &GameState,
        industry_building: &IndustryBuildingInfo,
    ) -> StationInfo {
        let options = industry_building
            .candidate_station_locations()
            .into_iter()
            .map(|(tile, station_type)| {
                StationInfo::new(self.player_id, StationId::random(), tile, station_type)
            })
            .filter(|station_info| {
                // This `extended_station_info` is a hack to avoid a situation where we build a station but its ends are blocked
                let station_type = station_info.station_type();
                let extended_type = StationType {
                    orientation:     station_type.orientation,
                    platforms:       station_type.platforms,
                    length_in_tiles: station_type.length_in_tiles + 2,
                };
                let tile_diff = match station_type.orientation {
                    StationOrientation::NorthToSouth => DirectionXZ::North,
                    StationOrientation::WestToEast => DirectionXZ::West,
                };
                let extended_station_info = StationInfo::new(
                    self.player_id,
                    StationId::random(),
                    station_info.reference_tile() + tile_diff,
                    extended_type,
                );

                game_state
                    .can_build_station(self.player_id, &extended_station_info)
                    .is_ok()
            })
            .collect::<Vec<_>>();

        choose(&options)
            .expect("Expected to find a station")
            .clone()
    }

    fn build_fully_connected_supply_chain(
        &self,
        game_state: &GameState,
        industries: &[IndustryType],
        known: &HashMap<IndustryType, IndustryBuildingId>,
        reference_tile: TileCoordsXZ,
    ) -> Vec<GameCommand> {
        let mut results = vec![];
        let mut known = known.clone();
        let mut stations = HashMap::new();
        for industry in industries {
            if !known.contains_key(industry) {
                if let Some(building) =
                    self.select_industry_building(game_state, *industry, reference_tile)
                {
                    known.insert(*industry, building.id());
                    let station = self.select_station_building(game_state, &building);
                    stations.insert(building.id(), station.clone());

                    results.push(GameCommand::BuildIndustryBuilding(building));
                    results.push(GameCommand::BuildStation(station));
                }
            }
        }

        // TODO HIGH: What about the station for ConstructionYard !?
        // TODO HIGH: Ensure all tracks are built
        // TODO HIGH: Ensure all trains are built
        // TODO HIGH: Return what we have built to ensure that these are now "locked" for that goal and not reused for other goals... the nuance here is that LumberMill produces Cellulose and Timber... and only Timber gets used in Timber flow...

        results
    }

    fn commands_for_goal(&self, game_state: &GameState, goal: Goal) -> Vec<GameCommand> {
        match goal {
            Goal::SteelToConstructionYard(construction_yard_id) => {
                let reference_tile = game_state
                    .building_state()
                    .find_industry_building(construction_yard_id)
                    .expect(&format!(
                        "Expected to find construction yard {construction_yard_id}"
                    ))
                    .reference_tile();
                self.build_fully_connected_supply_chain(
                    game_state,
                    &[
                        IndustryType::IronMine,
                        IndustryType::CoalMine,
                        IndustryType::SteelMill,
                        IndustryType::ConstructionYard,
                    ],
                    &HashMap::from([(IndustryType::ConstructionYard, construction_yard_id)]),
                    reference_tile,
                )
            },
            Goal::TimberToConstructionYard(_construction_yard_id) => {
                vec![] // TODO HIGH
            },
            Goal::ConcreteToConstructionYard(_construction_yard_id) => {
                vec![] // TODO HIGH
            },
        }
    }
}

impl ArtificialIntelligenceState for Oct2025ArtificialIntelligenceState {
    fn ai_commands(
        &mut self,
        game_state: &GameState,
        _metrics: &dyn Metrics,
    ) -> Option<Vec<GameCommand>> {
        let next_goal = self.pending_goals.first().copied();
        match next_goal {
            None => None,
            Some(goal) => {
                // TODO: This assumes that the goal is always achieved, that all commands succeed. That's wrong.
                self.pending_goals.remove(0);
                Some(self.commands_for_goal(game_state, goal))
            },
        }
    }
}
