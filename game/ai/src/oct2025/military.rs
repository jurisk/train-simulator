use std::collections::HashMap;

use shared_domain::building::industry_building_info::IndustryBuildingInfo;
use shared_domain::building::industry_type::IndustryType;
use shared_domain::building::military_building_info::MilitaryBuildingInfo;
use shared_domain::building::military_building_type::MilitaryBuildingType;
use shared_domain::client_command::GameCommand;
use shared_domain::game_state::GameState;
use shared_domain::metrics::Metrics;
use shared_domain::supply_chain::SupplyChain;
use shared_domain::tile_coords_xz::{TileCoordsXZ, TileDistance};
use shared_domain::{IndustryBuildingId, MilitaryBuildingId, PlayerId};

use crate::oct2025::industries::select_industry_building;
use crate::oct2025::supply_chains::BuildSupplyChains;
use crate::oct2025::{Goal, GoalResult, invoke_to_finished};

#[derive(Clone)]
struct MilitaryBaseAI {
    build_supply_chains: BuildSupplyChains,
}

impl MilitaryBaseAI {
    fn for_base(
        supply_chain: &SupplyChain,
        location: TileCoordsXZ,
        base_id: IndustryBuildingId,
    ) -> Self {
        let build_supply_chains = BuildSupplyChains::for_known_target(
            supply_chain,
            IndustryType::MilitaryBase,
            location,
            base_id,
        );
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
    ) -> GoalResult {
        self.build_supply_chains
            .commands(player_id, game_state, metrics)
    }
}

#[derive(Clone)]
pub(crate) struct MilitaryBasesAI {
    bases:             HashMap<IndustryBuildingId, MilitaryBaseAI>,
    fixed_artilleries: HashMap<MilitaryBuildingId, TileCoordsXZ>,
}

impl MilitaryBasesAI {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            bases:             HashMap::new(),
            fixed_artilleries: HashMap::new(),
        }
    }
}

impl Goal for MilitaryBasesAI {
    fn commands(
        &mut self,
        player_id: PlayerId,
        game_state: &GameState,
        metrics: &dyn Metrics,
    ) -> GoalResult {
        // Ensure all existing this player's 'MilitaryBase'-s have corresponding AI
        for base in game_state
            .building_state()
            .find_industry_buildings_by_owner_and_type(player_id, IndustryType::MilitaryBase)
        {
            self.bases.entry(base.id()).or_insert_with(|| {
                MilitaryBaseAI::for_base(
                    game_state.supply_chain(),
                    base.reference_tile(),
                    base.id(),
                )
            });
        }

        let empty = self.bases.is_empty();

        if empty {
            // TODO: We could have a race conditions that we keep spamming multiple such commands before the first one gets processed!?
            match select_military_base(player_id, game_state) {
                None => GoalResult::TryAgainLater,
                Some(base) => {
                    GoalResult::SendCommands(vec![GameCommand::BuildIndustryBuilding(base)])
                },
            }
        } else {
            for base in self.bases.values_mut() {
                let result = invoke_to_finished(|| base.commands(player_id, game_state, metrics));
                match result {
                    GoalResult::Finished => {},
                    other => return other,
                }
            }

            for artillery in game_state
                .building_state()
                .find_military_buildings_by_owner_and_type(
                    player_id,
                    MilitaryBuildingType::FixedArtillery,
                )
            {
                self.fixed_artilleries
                    .entry(artillery.id())
                    .or_insert_with(|| artillery.reference_tile());
            }

            if self.fixed_artilleries.len() < self.bases.len() {
                if let Some(artillery) = select_fixed_artillery(player_id, game_state) {
                    GoalResult::SendCommands(vec![GameCommand::BuildMilitaryBuilding(artillery)])
                } else {
                    GoalResult::TryAgainLater
                }
            } else {
                GoalResult::Finished
            }
        }
    }
}

fn select_fixed_artillery(
    player_id: PlayerId,
    game_state: &GameState,
) -> Option<MilitaryBuildingInfo> {
    for base in game_state
        .building_state()
        .find_industry_buildings_by_owner_and_type(player_id, IndustryType::MilitaryBase)
    {
        let base_tile = base.reference_tile();

        // TODO: This is just for testing, in reality we should be building them in the direction of the enemy
        let random_offset = TileCoordsXZ::new(fastrand::i32(-10 ..= 10), fastrand::i32(-10 ..= 10));

        let artillery_tile = base_tile + random_offset;
        let info = MilitaryBuildingInfo::new(
            MilitaryBuildingId::random(),
            player_id,
            MilitaryBuildingType::FixedArtillery,
            artillery_tile,
        );
        if game_state
            .can_build_military_building(player_id, &info)
            .is_ok()
        {
            return Some(info);
        }
    }

    None
}

#[expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
fn select_military_base(
    player_id: PlayerId,
    game_state: &GameState,
) -> Option<IndustryBuildingInfo> {
    // TODO: Pick a better location for a military base, perhaps in the direction of enemy ConstructionYard?
    let mid_x = (game_state.map_level().terrain().tile_count_x() / 2) as TileDistance;
    let mid_z = (game_state.map_level().terrain().tile_count_z() / 2) as TileDistance;

    select_industry_building(
        player_id,
        game_state,
        IndustryType::MilitaryBase,
        TileCoordsXZ::new(mid_x, mid_z),
    )
}
