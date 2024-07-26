use serde::{Deserialize, Serialize};

use crate::building_state::BuildingState;
use crate::game_time::GameTimeDiff;
use crate::transport::advancement::advance;
use crate::transport::transport_info::{TransportDynamicInfo, TransportInfo};
use crate::TransportId;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct TransportState {
    transports: Vec<TransportInfo>,
}

impl TransportState {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            transports: Vec::new(),
        }
    }

    #[must_use]
    pub fn to_vec(&self) -> Vec<TransportInfo> {
        self.transports.clone()
    }

    pub(crate) fn advance_time_diff(&mut self, diff: GameTimeDiff, buildings: &mut BuildingState) {
        for transport in &mut self.transports {
            advance(transport, buildings, diff);
        }
    }

    pub(crate) fn upsert(&mut self, transport: TransportInfo) {
        let transport_id = transport.transport_id();
        if let Some(existing_transport) = self
            .transports
            .iter_mut()
            .find(|t| t.transport_id() == transport_id)
        {
            existing_transport.clone_from(&transport);
        } else {
            self.transports.push(transport);
        }
    }

    pub(crate) fn update_dynamic_info(
        &mut self,
        transport_id: TransportId,
        transport_dynamic_info: &TransportDynamicInfo,
    ) {
        for transport in &mut self.transports {
            if transport.transport_id() == transport_id {
                transport.update_dynamic_info(transport_dynamic_info);
                return;
            }
        }
    }

    pub(crate) fn info_by_id(&self, transport_id: TransportId) -> Option<&TransportInfo> {
        self.transports
            .iter()
            .find(|transport| transport.transport_id() == transport_id)
    }
}
