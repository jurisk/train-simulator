use serde::{Deserialize, Serialize};

use crate::building::building_info::BuildingStaticInfo;
use crate::resource_type::ResourceType;
use crate::ResourceId;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct ResourceInfo {
    id:            ResourceId,
    resource_type: ResourceType,
    static_info:   BuildingStaticInfo,
}
