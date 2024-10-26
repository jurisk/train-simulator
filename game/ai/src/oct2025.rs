#![expect(clippy::module_name_repetitions)]

use std::collections::HashMap;
use std::fmt::Debug;

use log::{debug, info, trace, warn};
use shared_domain::building::industry_building_info::IndustryBuildingInfo;
use shared_domain::building::industry_type::IndustryType;
use shared_domain::building::station_info::StationInfo;
use shared_domain::client_command::GameCommand;
use shared_domain::directional_edge::DirectionalEdge;
use shared_domain::game_state::GameState;
use shared_domain::metrics::Metrics;
use shared_domain::resource_type::ResourceType;
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_domain::transport::movement_orders::{MovementOrder, MovementOrders};
use shared_domain::transport::tile_track::TileTrack;
use shared_domain::transport::track_planner::{DEFAULT_ALREADY_EXISTS_COEF, plan_tracks};
use shared_domain::transport::transport_info::TransportInfo;
use shared_domain::transport::transport_type::TransportType;
use shared_domain::{IndustryBuildingId, PlayerId, StationId, TransportId};
use shared_util::random::choose;

use crate::ArtificialIntelligenceState;

trait Goal {
    fn commands(
        &mut self,
        player_id: PlayerId,
        game_state: &GameState,
        metrics: &dyn Metrics,
    ) -> Option<Vec<GameCommand>>;
}

#[derive(Clone, Debug)]
enum IndustryState {
    // TODO: Could have more gradual steps, e.g. don't assume that building will succeed and have "BuildingIndustry" and "BuildingStation" states...
    NothingDone,
    IndustryBuilt(IndustryBuildingId, TileCoordsXZ),
    StationBuilt(IndustryBuildingId, TileCoordsXZ, StationId),
}

impl IndustryState {
    #[expect(clippy::collapsible_else_if)]
    #[must_use]
    fn commands(
        &mut self,
        industry: IndustryType,
        player_id: PlayerId,
        game_state: &GameState,
        centre_of_gravity: TileCoordsXZ,
    ) -> Option<Vec<GameCommand>> {
        trace!("IndustryState for {industry:?}: {self:?}");
        match self {
            IndustryState::NothingDone => {
                if let Some(building) =
                    select_industry_building(player_id, game_state, industry, centre_of_gravity)
                {
                    *self = IndustryState::IndustryBuilt(building.id(), building.reference_tile());
                    Some(vec![GameCommand::BuildIndustryBuilding(building)])
                } else {
                    info!("Failed to select building for {industry:?}");
                    None
                }
            },
            IndustryState::IndustryBuilt(industry_building_id, location) => {
                if let Some(station) = game_state
                    .building_state()
                    .find_linked_station(*industry_building_id)
                {
                    *self =
                        IndustryState::StationBuilt(*industry_building_id, *location, station.id());
                    self.commands(industry, player_id, game_state, centre_of_gravity)
                } else {
                    if let Some(building) = game_state
                        .building_state()
                        .find_industry_building(*industry_building_id)
                    {
                        let station = select_station_building(player_id, game_state, building);
                        trace!(
                            "Building station {station:?} for {industry:?} at {industry_building_id:?}"
                        );
                        if let Some(station) = station {
                            *self = IndustryState::StationBuilt(
                                *industry_building_id,
                                *location,
                                station.id(),
                            );
                            Some(vec![GameCommand::BuildStation(station)])
                        } else {
                            // TODO: This could happen, as we may have built some tracks in the neighbourhood before building all industries and stations.
                            warn!("Failed to select station for {industry:?} at {location:?}");
                            None
                        }
                    } else {
                        info!("Industry building {industry_building_id:?} not found");
                        None
                    }
                }
            },
            IndustryState::StationBuilt(_industry_building_id, _location, _station_id) => None,
        }
    }
}

#[derive(Clone)]
enum ResourceLinkState {
    Pending,
    TracksPending(Vec<(TileTrack, TileTrack)>),
    TracksBuilt,
    TrainPurchased,
}

fn lookup_station_id(industry_state: &IndustryState) -> Option<StationId> {
    if let IndustryState::StationBuilt(_industry_building_id, _location, station_id) =
        industry_state
    {
        Some(*station_id)
    } else {
        trace!("No station built for {industry_state:?}");
        None
    }
}

fn lookup_station<'a>(
    industry_state: &'a IndustryState,
    game_state: &'a GameState,
) -> Option<&'a StationInfo> {
    let station_id = lookup_station_id(industry_state)?;
    game_state.building_state().find_station(station_id)
}

impl ResourceLinkState {
    #[expect(clippy::collapsible_else_if)]
    #[must_use]
    fn commands(
        &mut self,
        from_industry_state: &IndustryState,
        resource: ResourceType,
        to_industry_state: &IndustryState,
        player_id: PlayerId,
        game_state: &GameState,
        metrics: &dyn Metrics,
    ) -> Option<Vec<GameCommand>> {
        match self {
            ResourceLinkState::Pending => {
                let from_station = lookup_station(from_industry_state, game_state)?;
                let to_station = lookup_station(to_industry_state, game_state)?;

                let mut pairs = vec![];
                for track_a in from_station.station_exit_tile_tracks() {
                    for track_b in to_station.station_exit_tile_tracks() {
                        pairs.push((track_a, track_b));
                        pairs.push((track_b, track_a));
                    }
                }
                *self = ResourceLinkState::TracksPending(pairs);
                self.commands(
                    from_industry_state,
                    resource,
                    to_industry_state,
                    player_id,
                    game_state,
                    metrics,
                )
            },
            ResourceLinkState::TracksPending(pairs) => {
                if let Some((source, target)) = pairs.pop() {
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
                            self.commands(
                                from_industry_state,
                                resource,
                                to_industry_state,
                                player_id,
                                game_state,
                                metrics,
                            )
                        } else {
                            if game_state.can_build_tracks(player_id, &route).is_ok() {
                                Some(vec![GameCommand::BuildTracks(route)])
                            } else {
                                Some(vec![])
                            }
                        }
                    } else {
                        debug!("No route found for {:?} -> {:?}", source, target);
                        self.commands(
                            from_industry_state,
                            resource,
                            to_industry_state,
                            player_id,
                            game_state,
                            metrics,
                        )
                    }
                } else {
                    *self = ResourceLinkState::TracksBuilt;
                    self.commands(
                        from_industry_state,
                        resource,
                        to_industry_state,
                        player_id,
                        game_state,
                        metrics,
                    )
                }
            },
            ResourceLinkState::TracksBuilt => {
                *self = ResourceLinkState::TrainPurchased;
                let from_station = lookup_station_id(from_industry_state)?;
                let to_station = lookup_station_id(to_industry_state)?;

                let command = purchase_transport_command(
                    player_id,
                    game_state,
                    from_station,
                    resource,
                    to_station,
                )?;
                Some(vec![command])
            },
            ResourceLinkState::TrainPurchased => None,
        }
    }
}

fn purchase_transport_command(
    player_id: PlayerId,
    game_state: &GameState,
    from_station: StationId,
    resource_type: ResourceType,
    to_station: StationId,
) -> Option<GameCommand> {
    let mut movement_orders = MovementOrders::one(MovementOrder::stop_at_station(from_station));
    movement_orders.push(MovementOrder::stop_at_station(to_station));

    let from_station_info = game_state.building_state().find_station(from_station)?;
    let tile_tracks = from_station_info.station_exit_tile_tracks();
    let tile_track = tile_tracks.first()?;
    let transport_location =
        from_station_info.transport_location_at_station(tile_track.tile, tile_track.pointing_in)?;

    let transport_info = TransportInfo::new(
        TransportId::random(),
        player_id,
        TransportType::cargo_train(resource_type),
        transport_location,
        movement_orders,
    );
    let command = GameCommand::PurchaseTransport(from_station, transport_info);

    Some(command)
}

#[derive(Clone)]
struct BuildSupplyChain {
    target_type:          IndustryType,
    centre_of_gravity:    Option<TileCoordsXZ>,
    industry_states:      HashMap<IndustryType, IndustryState>,
    resource_link_states: HashMap<(IndustryType, ResourceType, IndustryType), ResourceLinkState>,
}

impl BuildSupplyChain {
    #[expect(clippy::too_many_arguments)]
    fn resource_link_commands(
        industry_states: &HashMap<IndustryType, IndustryState>,
        state: &mut ResourceLinkState,
        from_industry: IndustryType,
        resource: ResourceType,
        to_industry: IndustryType,
        player_id: PlayerId,
        game_state: &GameState,
        metrics: &dyn Metrics,
    ) -> Option<Vec<GameCommand>> {
        let from_industry_state = industry_states.get(&from_industry)?;
        let to_industry_state = industry_states.get(&to_industry)?;
        state.commands(
            from_industry_state,
            resource,
            to_industry_state,
            player_id,
            game_state,
            metrics,
        )
    }
}

impl Goal for BuildSupplyChain {
    fn commands(
        &mut self,
        player_id: PlayerId,
        game_state: &GameState,
        metrics: &dyn Metrics,
    ) -> Option<Vec<GameCommand>> {
        if let Some(centre_of_gravity) = self.centre_of_gravity {
            for (industry, state) in &mut self.industry_states {
                if let Some(responses) =
                    state.commands(*industry, player_id, game_state, centre_of_gravity)
                {
                    return Some(responses);
                }
            }
        } else {
            // TODO HIGH: Implement
            panic!(
                "We don't know yet how to set the centre of gravity for target {:?}",
                self.target_type
            );
            // self.commands(player_id, game_state, metrics);
        }

        for ((from_industry, resource, to_industry), state) in &mut self.resource_link_states {
            if let Some(responses) = Self::resource_link_commands(
                &self.industry_states,
                state,
                *from_industry,
                *resource,
                *to_industry,
                player_id,
                game_state,
                metrics,
            ) {
                return Some(responses);
            }
        }

        None
    }
}

// TODO: You can generate this from the industry definitions
fn industries_for_resource_and_target(
    resource_type: ResourceType,
    target_type: IndustryType,
) -> Vec<IndustryType> {
    match (resource_type, target_type) {
        (ResourceType::Steel, IndustryType::ConstructionYard) => {
            vec![
                IndustryType::IronMine,
                IndustryType::CoalMine,
                IndustryType::SteelMill,
                IndustryType::ConstructionYard,
            ]
        },
        (ResourceType::Timber, IndustryType::ConstructionYard) => {
            vec![
                IndustryType::Forestry,
                IndustryType::LumberMill,
                IndustryType::ConstructionYard,
            ]
        },
        (ResourceType::Concrete, IndustryType::ConstructionYard) => {
            vec![
                IndustryType::ClayPit,
                IndustryType::SandAndGravelQuarry,
                IndustryType::LimestoneMine,
                IndustryType::CementPlant,
                IndustryType::ConcretePlant,
                IndustryType::ConstructionYard,
            ]
        },
        (ResourceType::ArtilleryWeapons, IndustryType::MilitaryBase) => {
            vec![
                IndustryType::CoalMine,
                IndustryType::IronMine,
                IndustryType::SteelMill,
                IndustryType::WeaponsFactory,
                IndustryType::MilitaryBase,
            ]
        },
        (ResourceType::Food, IndustryType::MilitaryBase) => {
            vec![
                IndustryType::Farm,
                IndustryType::FoodProcessingPlant,
                IndustryType::MilitaryBase,
            ]
        },
        (ResourceType::Ammunition, IndustryType::MilitaryBase) => {
            vec![
                IndustryType::AmmunitionFactory,
                IndustryType::ExplosivesPlant,
                IndustryType::NitrateMine,
                IndustryType::SulfurMine,
                IndustryType::IronMine,
                IndustryType::CoalMine,
                IndustryType::SteelMill,
                IndustryType::MilitaryBase,
            ]
        },
        _ => {
            panic!(
                "Unsupported resource and target combination: {resource_type:?} -> {target_type:?}"
            )
        },
    }
}

impl BuildSupplyChain {
    #[must_use]
    pub fn with_center_of_gravity(
        resource_type: ResourceType,
        target_type: IndustryType,
        center_of_gravity: TileCoordsXZ,
    ) -> Self {
        let industries = industries_for_resource_and_target(resource_type, target_type);

        let industry_states: HashMap<IndustryType, IndustryState> = industries
            .iter()
            .map(|industry| (*industry, IndustryState::NothingDone))
            .collect();

        let resource_link_states = resource_links(&industries)
            .into_iter()
            .map(|(from_industry, resource, to_industry)| {
                (
                    (from_industry, resource, to_industry),
                    ResourceLinkState::Pending,
                )
            })
            .collect();

        Self {
            target_type,
            centre_of_gravity: Some(center_of_gravity),
            industry_states,
            resource_link_states,
        }
    }

    #[must_use]
    pub fn with_built_target(
        resource_type: ResourceType,
        target_type: IndustryType,
        target_location: TileCoordsXZ,
        target_id: IndustryBuildingId,
    ) -> Self {
        let mut result = Self::with_center_of_gravity(resource_type, target_type, target_location);
        result.industry_states.insert(
            target_type,
            IndustryState::IndustryBuilt(target_id, target_location),
        );
        result
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
            BuildSupplyChain::with_built_target(
                ResourceType::Steel,
                IndustryType::ConstructionYard,
                construction_yard_location,
                construction_yard_id,
            ),
            BuildSupplyChain::with_built_target(
                ResourceType::Timber,
                IndustryType::ConstructionYard,
                construction_yard_location,
                construction_yard_id,
            ),
            BuildSupplyChain::with_built_target(
                ResourceType::Concrete,
                IndustryType::ConstructionYard,
                construction_yard_location,
                construction_yard_id,
            ),
            // TODO HIGH: These should be built without a center of gravity, and thus their locations decided dynamically
            BuildSupplyChain::with_center_of_gravity(
                ResourceType::ArtilleryWeapons,
                IndustryType::MilitaryBase,
                construction_yard_location,
            ),
            BuildSupplyChain::with_center_of_gravity(
                ResourceType::Ammunition,
                IndustryType::MilitaryBase,
                construction_yard_location,
            ),
            BuildSupplyChain::with_center_of_gravity(
                ResourceType::Food,
                IndustryType::MilitaryBase,
                construction_yard_location,
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
) -> Option<StationInfo> {
    let options = industry_building
        .candidate_station_locations()
        .into_iter()
        .map(|(tile, station_type)| {
            StationInfo::new(owner_id, StationId::random(), tile, station_type)
        })
        .filter(|station_info| {
            game_state.can_build_station(owner_id, station_info).is_ok()
                && station_info
                    .station_exit_tile_tracks()
                    .into_iter()
                    .all(|tile_track| {
                        let next_tile = tile_track.next_tile_coords();
                        let not_under_water =
                            !game_state.map_level().any_vertex_under_water(next_tile);
                        let free_tile =
                            game_state.building_state().building_at(next_tile).is_none();
                        not_under_water && free_tile
                    })
        })
        .collect::<Vec<_>>();

    choose(&options).cloned()
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
        for goal in &mut self.pending_goals {
            if let Some(commands) = goal.commands(self.player_id, game_state, metrics) {
                return Some(commands);
            }
        }

        info!("AI has nothing to do");
        None
    }
}
