use crate::{surface::Sector, textures::Texture};

use super::{plane::PlaneRenderer, portal::PortalTree, wall::WallRenderer, RendererState};

pub struct SectorRenderer {
    wall_renderer: WallRenderer,
    plane_renderer: PlaneRenderer,
}

impl SectorRenderer {
    pub fn new(state: &RendererState) -> Self {
        Self {
            wall_renderer: WallRenderer::new(state),
            plane_renderer: PlaneRenderer::new(state),
        }
    }

    pub fn set_viewport(&mut self, state: &RendererState) {
        self.wall_renderer.set_viewport(state);
        self.plane_renderer.set_viewport(state);
    }

    pub fn update(&mut self, state: &RendererState) {
        self.wall_renderer.update(state);
        self.plane_renderer.update(state);
    }

    pub fn draw_sector(
        &mut self,
        state: &mut RendererState,
        portals: &mut PortalTree,
        sectors: &[Sector],
        textures: &[Texture],
        portal_index: usize,
    ) {
        let sector_index = portals.nodes[portal_index].sector_index;
        let sector = &sectors[sector_index];

        for wall in sector.walls.iter() {
            self.wall_renderer
                .render(state, portals, sectors, textures, portal_index, wall)
        }

        let portal = unsafe { portals.get_node_unchecked(portal_index) };

        let (min_portal_bounds, max_portal_bounds) =
            unsafe { portals.get_bounds_unchecked(portal.tree_depth) };

        let (min_wall_bounds, max_wall_bounds) = self.wall_renderer.get_wall_bounds();

        let vs_ceiling_height = state.camera.z - sector.ceiling.height;
        let vs_floor_height = state.camera.z - sector.floor.height;

        // Draw sector ceiling
        self.plane_renderer.draw_plane(
            state,
            portal,
            (min_portal_bounds, min_wall_bounds),
            vs_ceiling_height,
            textures.get(sector.ceiling.texture_data.index).unwrap(),
            sector.ceiling.texture_data.offset,
            &sector.ceiling.texture_data.scale_rotate,
        );

        // Draw sector floor
        self.plane_renderer.draw_plane(
            state,
            portal,
            (max_wall_bounds, max_portal_bounds),
            vs_floor_height,
            textures.get(sector.floor.texture_data.index).unwrap(),
            sector.floor.texture_data.offset,
            &sector.floor.texture_data.scale_rotate,
        );
    }
}
