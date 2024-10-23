#![expect(clippy::module_name_repetitions)]

use std::collections::HashMap;

use log::debug;
use shared_domain::building::industry_building_info::IndustryBuildingInfo;
use shared_domain::building::industry_type::IndustryType;
use shared_domain::building::station_info::StationInfo;
use shared_domain::building::station_type::{StationOrientation, StationType};
use shared_domain::client_command::GameCommand;
use shared_domain::directional_edge::DirectionalEdge;
use shared_domain::game_state::GameState;
use shared_domain::metrics::Metrics;
use shared_domain::resource_type::ResourceType;
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_domain::transport::track_planner::{DEFAULT_ALREADY_EXISTS_COEF, plan_tracks};
use shared_domain::{IndustryBuildingId, PlayerId, StationId};
use shared_util::direction_xz::DirectionXZ;
use shared_util::random::choose;

use crate::ArtificialIntelligenceState;

trait Goal {
    fn commands(
        &self,
        player_id: PlayerId,
        game_state: &GameState,
        metrics: &dyn Metrics,
    ) -> Vec<GameCommand>;
}

#[derive(Clone)]
struct BuildSupplyChain {
    resource_type:   ResourceType,
    target_location: TileCoordsXZ,
    states:          HashMap<IndustryType, Option<(IndustryBuildingId, TileCoordsXZ)>>,
}

impl Goal for BuildSupplyChain {
    fn commands(
        &self,
        player_id: PlayerId,
        game_state: &GameState,
        metrics: &dyn Metrics,
    ) -> Vec<GameCommand> {
        // TODO HIGH: Make more gradual, build one at a time, otherwise we get InvalidOverlap-s

        let mut results = vec![];
        let mut known = self.states.clone();
        let mut stations = HashMap::new();

        let industries = self.states.keys().copied().collect::<Vec<_>>();
        for industry in &industries {
            let existing = known.get(industry).unwrap_or(&None);
            match existing {
                None => {
                    if let Some(building) = select_industry_building(
                        player_id,
                        game_state,
                        *industry,
                        self.target_location,
                    ) {
                        known.insert(*industry, Some((building.id(), building.reference_tile())));
                        let station = select_station_building(player_id, game_state, &building);
                        stations.insert(building.id(), station.clone());

                        results.push(GameCommand::BuildIndustryBuilding(building));
                        results.push(GameCommand::BuildStation(station));
                    }
                },
                Some((building, _location)) => {
                    let building = game_state
                        .building_state()
                        .find_industry_building(*building)
                        .unwrap();
                    match game_state
                        .building_state()
                        .find_linked_station(building.id())
                    {
                        None => {
                            let station = select_station_building(player_id, game_state, building);
                            stations.insert(building.id(), station.clone());
                            results.push(GameCommand::BuildStation(station));
                        },
                        Some(station) => {
                            stations.insert(building.id(), station.clone());
                        },
                    }
                },
            }
        }

        for (from_industry, _resource, to_industry) in resource_links(&industries) {
            let (from_industry_id, _) = known.get(&from_industry).unwrap().unwrap();
            let from_station = stations.get(&from_industry_id).unwrap();
            let (to_industry_id, _) = known.get(&to_industry).unwrap().unwrap();
            let to_station = stations.get(&to_industry_id).unwrap();
            let mut pairs = vec![];
            for track_a in from_station.station_exit_tile_tracks() {
                for track_b in to_station.station_exit_tile_tracks() {
                    pairs.push((track_a, track_b));
                    pairs.push((track_b, track_a));
                }
            }

            for (source, target) in pairs {
                if let Some((route, _length)) = plan_tracks(
                    player_id,
                    DirectionalEdge::exit_from(source),
                    &[DirectionalEdge::entrance_to(target)],
                    game_state,
                    DEFAULT_ALREADY_EXISTS_COEF,
                    metrics,
                ) {
                    if route.is_empty() {
                        // If it's empty, it means it's already built
                        continue;
                    }
                    if game_state.can_build_tracks(player_id, &route).is_ok() {
                        results.push(GameCommand::BuildTracks(route));
                    }
                } else {
                    debug!("No route found for {:?} -> {:?}", source, target);
                }
            }
            // TODO HIGH: Ensure all tracks are built - right now we have invalid overlap
            // TODO HIGH: Ensure all trains are built
        }

        // TODO HIGH: Return what we have built to ensure that these are now "locked" for that goal and not reused for other goals...

        results
    }
}

impl BuildSupplyChain {
    #[must_use]
    pub fn new(
        resource_type: ResourceType,
        target_type: IndustryType,
        target_location: TileCoordsXZ,
        target_id: IndustryBuildingId,
    ) -> Self {
        let states = match resource_type {
            ResourceType::Steel => {
                vec![
                    IndustryType::IronMine,
                    IndustryType::CoalMine,
                    IndustryType::SteelMill,
                ]
            },
            ResourceType::Timber => {
                vec![IndustryType::Forestry, IndustryType::LumberMill]
            },
            ResourceType::Concrete => {
                vec![
                    IndustryType::ClayPit,
                    IndustryType::SandAndGravelQuarry,
                    IndustryType::LimestoneMine,
                    IndustryType::CementPlant,
                    IndustryType::ConcretePlant,
                ]
            },
            _ => panic!("Unsupported resource type"),
        };

        let mut states: HashMap<IndustryType, Option<(IndustryBuildingId, TileCoordsXZ)>> = states
            .into_iter()
            .map(|industry| (industry, None))
            .collect();
        states.insert(target_type, Some((target_id, target_location)));

        Self {
            resource_type,
            target_location,
            states,
        }
    }
}

pub struct Oct2025ArtificialIntelligenceState {
    player_id:     PlayerId,
    pending_goals: Vec<Box<dyn Goal + Send + Sync>>,
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
        let construction_yard_location = construction_yard.reference_tile();
        let construction_yard_id = construction_yard.id();
        let pending_goals = vec![
            BuildSupplyChain::new(
                ResourceType::Steel,
                IndustryType::ConstructionYard,
                construction_yard_location,
                construction_yard_id,
            ),
            BuildSupplyChain::new(
                ResourceType::Timber,
                IndustryType::ConstructionYard,
                construction_yard_location,
                construction_yard_id,
            ),
            BuildSupplyChain::new(
                ResourceType::Concrete,
                IndustryType::ConstructionYard,
                construction_yard_location,
                construction_yard_id,
            ),
        ]
        .into_iter()
        .map(|goal| Box::new(goal) as Box<dyn Goal + Send + Sync>)
        .collect();
        Self {
            player_id,
            pending_goals,
        }
    }
}

fn select_industry_building(
    owner_id: PlayerId,
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
                owner_id,
                IndustryBuildingId::random(),
                zoning.reference_tile(),
                industry_type,
            )
        })
        .filter(|info| {
            game_state
                .can_build_industry_building(owner_id, info)
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
    owner_id: PlayerId,
    game_state: &GameState,
    industry_building: &IndustryBuildingInfo,
) -> StationInfo {
    let options = industry_building
        .candidate_station_locations()
        .into_iter()
        .map(|(tile, station_type)| {
            StationInfo::new(owner_id, StationId::random(), tile, station_type)
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
                owner_id,
                StationId::random(),
                station_info.reference_tile() + tile_diff,
                extended_type,
            );

            game_state
                .can_build_station(owner_id, &extended_station_info)
                .is_ok()
        })
        .collect::<Vec<_>>();

    choose(&options)
        .expect("Expected to find a station")
        .clone()
}

fn resource_links(industries: &[IndustryType]) -> Vec<(IndustryType, ResourceType, IndustryType)> {
    let mut results = vec![];
    for a in industries {
        for b in industries {
            for c in ResourceType::all() {
                if a.produces(c) && b.consumes(c) {
                    results.push((*a, c, *b));
                }
            }
        }
    }
    results
}

impl ArtificialIntelligenceState for Oct2025ArtificialIntelligenceState {
    fn ai_commands(
        &mut self,
        game_state: &GameState,
        metrics: &dyn Metrics,
    ) -> Option<Vec<GameCommand>> {
        if self.pending_goals.is_empty() {
            None
        } else {
            // TODO: This assumes that the goal is always achieved, that all commands succeed. That's wrong.
            let goal = self.pending_goals.pop().unwrap();
            Some(goal.commands(self.player_id, game_state, metrics))
        }
    }
}
