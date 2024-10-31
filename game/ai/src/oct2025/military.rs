use std::collections::HashMap;

use shared_domain::building::industry_building_info::IndustryBuildingInfo;
use shared_domain::building::industry_type::IndustryType;
use shared_domain::client_command::GameCommand;
use shared_domain::game_state::GameState;
use shared_domain::metrics::Metrics;
use shared_domain::tile_coords_xz::{TileCoordsXZ, TileDistance};
use shared_domain::{IndustryBuildingId, PlayerId};

use crate::oct2025::Goal;
use crate::oct2025::industries::select_industry_building;
use crate::oct2025::supply_chains::BuildSupplyChains;

#[derive(Clone)]
struct MilitaryBaseAI {
    build_supply_chains: BuildSupplyChains,
}

impl MilitaryBaseAI {
    fn for_base(location: TileCoordsXZ, base_id: IndustryBuildingId) -> Self {
        let build_supply_chains =
            BuildSupplyChains::for_known_target(IndustryType::MilitaryBase, location, base_id);
        Self {
            build_supply_chains,
        }
    }
}

impl Goal for MilitaryBaseAI {
    fn commands(
        &mut self,
        player_id: PlayerId,
        game_state: &GameState,
        metrics: &dyn Metrics,
    ) -> Option<Vec<GameCommand>> {
        self.build_supply_chains
            .commands(player_id, game_state, metrics)
    }
}

#[derive(Clone)]
pub(crate) struct MilitaryBasesAI {
    bases: HashMap<IndustryBuildingId, MilitaryBaseAI>,
}

impl MilitaryBasesAI {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            bases: HashMap::new(),
        }
    }
}

impl Goal for MilitaryBasesAI {
    fn commands(
        &mut self,
        player_id: PlayerId,
        game_state: &GameState,
        metrics: &dyn Metrics,
    ) -> Option<Vec<GameCommand>> {
        for base in game_state
            .building_state()
            .find_industry_building_by_owner_and_type(player_id, IndustryType::MilitaryBase)
        {
            self.bases
                .entry(base.id())
                .or_insert_with(|| MilitaryBaseAI::for_base(base.reference_tile(), base.id()));
        }

        let empty = self.bases.is_empty();

        if empty {
            // TODO: We could have a race conditions that we keep spamming multiple such commands before the first one gets processed!?
            select_military_base(player_id, game_state)
                .map(|base| vec![GameCommand::BuildIndustryBuilding(base)])
        } else {
            for base in self.bases.values_mut() {
                if let Some(commands) = base.commands(player_id, game_state, metrics) {
                    return Some(commands);
                }
            }

            None
        }
    }
}

#[expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
fn select_military_base(
    player_id: PlayerId,
    game_state: &GameState,
) -> Option<IndustryBuildingInfo> {
    // TODO HIGH: Pick a better location for a military base, perhaps in the direction of enemy ConstructionYard?
    let mid_x = (game_state.map_level().terrain().tile_count_x() / 2) as TileDistance;
    let mid_z = (game_state.map_level().terrain().tile_count_z() / 2) as TileDistance;

    select_industry_building(
        player_id,
        game_state,
        IndustryType::MilitaryBase,
        TileCoordsXZ::new(mid_x, mid_z),
    )
}
