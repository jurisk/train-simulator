use crate::cargo_amount::CargoAmount;
use crate::cargo_map::CargoMap;
use crate::resource_type::ResourceType;

pub struct ResourceTransformItem {
    pub resource: ResourceType,
    pub amount:   CargoAmount,
}

impl ResourceTransformItem {
    #[must_use]
    pub fn new(resource: ResourceType, amount: CargoAmount) -> Self {
        Self { resource, amount }
    }
}

pub struct ResourceTransform {
    pub inputs:  Vec<ResourceTransformItem>,
    pub outputs: Vec<ResourceTransformItem>,
}

impl ResourceTransform {
    const CARGO_PER_SECOND: f32 = 0.2f32;

    #[must_use]
    pub fn make(inputs: Vec<(ResourceType, f32)>, outputs: Vec<(ResourceType, f32)>) -> Self {
        ResourceTransform::new(
            inputs
                .into_iter()
                .map(|(resource, coef)| {
                    ResourceTransformItem::new(
                        resource,
                        CargoAmount::new(Self::CARGO_PER_SECOND * coef),
                    )
                })
                .collect(),
            outputs
                .into_iter()
                .map(|(resource, coef)| {
                    ResourceTransformItem::new(
                        resource,
                        CargoAmount::new(Self::CARGO_PER_SECOND * coef),
                    )
                })
                .collect(),
        )
    }

    #[must_use]
    pub fn make_warehouse(inputs: &[ResourceType]) -> Self {
        let inputs = inputs.iter().map(|resource| (*resource, 0f32)).collect();
        ResourceTransform::make(inputs, vec![])
    }

    // Later: How do we handle when the stock is too full, and we should stop producing due to that?
    #[must_use]
    pub fn calculate_utilisation_percentage(&self, cargo: &CargoMap, seconds: f32) -> f32 {
        let mut utilisation = 1f32;
        for item in &self.inputs {
            let available = cargo.get(item.resource);
            let required = item.amount * seconds;
            let ratio = available / required;
            utilisation = utilisation.min(ratio);
        }
        utilisation
    }
}

impl ResourceTransform {
    #[must_use]
    pub fn new(inputs: Vec<ResourceTransformItem>, outputs: Vec<ResourceTransformItem>) -> Self {
        Self { inputs, outputs }
    }
}
