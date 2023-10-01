#[derive(Clone, Copy, Debug)]
pub struct PortalNode {
    pub tree_depth: usize,
    pub sector_index: usize,
    pub x_min: usize,
    pub x_max: usize,
    pub depth_min: f32,
    pub depth_max: f32,
}

/// # Implementation
/// For each level of the tree, we store a set of bounds that define the clipping region, known
/// as a portal, for the next sector to be rendered within. For each X coordinate, these bounds
/// define the lower (inc.) and upper (exc.) Y coordinates.
///
/// Each node represents a portal, and it stores the X coordinate range that is used to retrieve
/// the relevant Y coordinates. Nodes of the same depth share the same bounds buffer, which is possible
/// as the X coordinate ranges do not overlap between nodes of the same depth. Thus, when the X coordinate
/// range _is_ known to overlap (In the case that a portal is inside another portal), the overlapping
/// node becomes a child in a deeper layer and will use a different bounds buffer.
///
/// Nodes are stored in a flat array, in the order they are added. So, depending on the order they are
/// added, it can be used to traverse the tree breadth/depth-first in forward/reverse order. But, because
/// the functionality is not needed, nodes do not record their parent/child indices.
pub struct PortalTree {
    screen_width: usize,
    screen_height: usize,

    pub nodes: Vec<PortalNode>,
    pub portal_bounds_min: Vec<Vec<u16>>,
    pub portal_bounds_max: Vec<Vec<u16>>,
}

impl PortalTree {
    pub fn new(screen_width: usize, screen_height: usize) -> Self {
        Self::with_depth(1, screen_width, screen_height)
    }

    pub fn with_depth(depth: usize, screen_width: usize, screen_height: usize) -> Self {
        let nodes = Vec::with_capacity(32);
        let portal_bounds_min = vec![vec![0; screen_width]; depth];
        let portal_bounds_max = vec![vec![screen_height as u16; screen_width]; depth];

        Self {
            screen_width,
            screen_height,

            nodes,
            portal_bounds_min,
            portal_bounds_max,
        }
    }

    pub fn resize_bounds(&mut self, screen_width: usize, screen_height: usize) {
        self.screen_width = screen_width;
        self.screen_height = screen_height;

        for bounds in self.portal_bounds_min.iter_mut() {
            bounds.resize(screen_width, 0);
        }

        for bounds in self.portal_bounds_max.iter_mut() {
            bounds.resize(screen_width, self.screen_height as u16);
        }
    }

    pub fn reset(&mut self) {
        self.nodes.clear();

        for bounds in self.portal_bounds_min.iter_mut() {
            bounds.fill(0);
        }

        for bounds in self.portal_bounds_max.iter_mut() {
            bounds.fill(self.screen_height as u16);
        }
    }

    pub fn push_node(&mut self, node: PortalNode) {
        while node.tree_depth > (self.portal_bounds_min.len() - 1) {
            self.add_layer();
        }

        self.nodes.push(node);
    }

    pub unsafe fn get_node_unchecked(&self, index: usize) -> &PortalNode {
        debug_assert!(index < self.nodes.len());

        unsafe { self.nodes.get_unchecked(index) }
    }

    pub unsafe fn get_node_mut_unchecked(&mut self, index: usize) -> &mut PortalNode {
        debug_assert!(index < self.nodes.len());

        unsafe { self.nodes.get_unchecked_mut(index) }
    }

    pub fn nodes_len(&self) -> usize {
        self.nodes.len()
    }

    pub unsafe fn get_bounds_unchecked(&self, depth: usize) -> (&[u16], &[u16]) {
        debug_assert!(depth < self.portal_bounds_min.len());

        let min_bounds = unsafe { self.portal_bounds_min.get_unchecked(depth) };
        let max_bounds = unsafe { self.portal_bounds_max.get_unchecked(depth) };

        (min_bounds, max_bounds)
    }

    pub unsafe fn get_bounds_mut_unchecked(&mut self, depth: usize) -> (&mut [u16], &mut [u16]) {
        debug_assert!(depth < self.portal_bounds_min.len());

        let min_bounds = unsafe { self.portal_bounds_min.get_unchecked_mut(depth) };
        let max_bounds = unsafe { self.portal_bounds_max.get_unchecked_mut(depth) };

        (min_bounds, max_bounds)
    }

    pub fn get_many_bounds_mut_unchecked(
        &mut self,
        depth_1: usize,
        depth_2: usize,
    ) -> ((&mut [u16], &mut [u16]), (&mut [u16], &mut [u16])) {
        debug_assert_ne!(depth_1, depth_2);
        debug_assert!(depth_1 < self.portal_bounds_min.len());
        debug_assert!(depth_2 < self.portal_bounds_min.len());

        let min_bounds_1 = unsafe {
            core::slice::from_raw_parts_mut(
                self.portal_bounds_min
                    .get_unchecked_mut(depth_1)
                    .as_mut_ptr(),
                self.screen_width,
            )
        };

        let max_bounds_1 = unsafe {
            core::slice::from_raw_parts_mut(
                self.portal_bounds_max
                    .get_unchecked_mut(depth_1)
                    .as_mut_ptr(),
                self.screen_width,
            )
        };

        let min_bounds_2 = unsafe {
            core::slice::from_raw_parts_mut(
                self.portal_bounds_min
                    .get_unchecked_mut(depth_2)
                    .as_mut_ptr(),
                self.screen_width,
            )
        };

        let max_bounds_2 = unsafe {
            core::slice::from_raw_parts_mut(
                self.portal_bounds_max
                    .get_unchecked_mut(depth_2)
                    .as_mut_ptr(),
                self.screen_width,
            )
        };

        ((min_bounds_1, max_bounds_1), (min_bounds_2, max_bounds_2))
    }

    fn add_layer(&mut self) {
        self.portal_bounds_min.push(vec![0; self.screen_width]);
        self.portal_bounds_max
            .push(vec![self.screen_height as u16; self.screen_width]);
    }
}
