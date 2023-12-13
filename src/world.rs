use crate::vaults::Vault;

pub enum Plane {
    Terminal,
    Epsilon,
}

pub const WORLD_ORDER: &[Plane] = &[Plane::Terminal, Plane::Epsilon];

pub fn match_plane_with_vaults(
    plane: Plane
) -> Vault {
    match plane {
        Plane::Terminal => Vault::EpicWow,
        Plane::Epsilon => Vault::Epsilon,
    }
}