use bevy::{
    camera::{CameraProjection, PerspectiveProjection, SubCameraView},
    math::{Mat4, Vec3A, Vec4},
};

#[derive(Clone, Debug, Default)]
pub struct PerspectiveMinAspect {
    inner: PerspectiveProjection,
    portrait: bool,
}

impl PerspectiveMinAspect {
    const FLIP: Mat4 = Mat4::from_cols(Vec4::Y, Vec4::X, Vec4::Z, Vec4::W);
}

impl CameraProjection for PerspectiveMinAspect {
    fn get_clip_from_view(&self) -> Mat4 {
        let inner = self.inner.get_clip_from_view();
        if self.portrait { Self::FLIP * inner * Self::FLIP } else { inner }
    }

    fn get_clip_from_view_for_sub(&self, sub_view: &SubCameraView) -> Mat4 {
        let inner = self.inner.get_clip_from_view_for_sub(sub_view);
        if self.portrait { Self::FLIP * inner * Self::FLIP } else { inner }
    }

    fn update(&mut self, width: f32, height: f32) {
        self.portrait = width < height;
        if self.portrait {
            self.inner.update(height, width);
        } else {
            self.inner.update(width, height);
        }
    }

    fn far(&self) -> f32 {
        self.inner.far
    }

    fn get_frustum_corners(&self, z_near: f32, z_far: f32) -> [Vec3A; 8] {
        // I'm realizing this might technically be wrong, and we only haven't noticed because it hasn't come up
        self.inner.get_frustum_corners(z_near, z_far)
    }
}
