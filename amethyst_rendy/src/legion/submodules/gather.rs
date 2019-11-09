//! Helper gatherer structures for collecting information about the world.
use crate::{
    camera::Camera,
    legion::camera::ActiveCamera,
    pod::{self, IntoPod},
    resources::AmbientColor,
};
use amethyst_core::{
    legion::*,
    math::{convert, Matrix4, Vector3},
    transform::Transform,
};
use glsl_layout::*;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

type Std140<T> = <T as AsStd140>::Std140;

/// Helper `CameraGatherer` for fetching appropriate matrix information from camera entities.
#[derive(Debug)]
pub struct CameraGatherer {
    /// Fetched camera world position
    pub camera_position: vec3,
    /// Fetched camera projection matrix.
    pub projview: Std140<pod::ViewArgs>,
}

impl CameraGatherer {
    /// Collect just the entity which has the current `ActiveCamera`
    pub fn gather_camera_entity(world: &World) -> Option<Entity> {
        #[cfg(feature = "profiler")]
        profile_scope!("gather_camera (1st)");

        // TODO: we do not support active camera atm because of migration

        unsafe { <(Read<Camera>, Read<Transform>)>::query().iter_entities_unchecked(world) }
            .nth(0)
            .map(|(e, _)| e)
    }

    /// Collect `ActiveCamera` and `Camera` instances from the provided resource storage and selects
    /// the appropriate camera to use for projection, and returns the camera position and extracted
    /// projection matrix.
    ///
    /// The matrix returned is the camera's `Projection` matrix and the camera `Transform::global_view_matrix`
    pub fn gather(world: &World) -> Self {
        #[cfg(feature = "profiler")]
        profile_scope!("gather_cameras");

        let defcam = Camera::standard_3d(1.0, 1.0);
        let identity = Transform::default();

        let active_camera = world.resources.get::<ActiveCamera>();
        let mut camera_query = <(Read<Camera>, Read<Transform>)>::query();

        let active_camera = active_camera
            .and_then(|active_camera| {
                active_camera
                    .entity
                    .and_then(|e| {
                        unsafe { camera_query.iter_entities_unchecked(world) }
                            .find(|(camera_entity, (_, _))| *camera_entity == e)
                            .map(|(_, (camera, transform))| (camera, transform))
                    })
                    .or_else(|| {
                        unsafe { camera_query.iter_entities_unchecked(world) }
                            .nth(0)
                            .map(|(e, (camera, transform))| (camera, transform))
                    })
            })
            .or_else(|| None);

        let (position, view_matrix, projection) =
            if let Some((camera, transform)) = active_camera.as_ref() {
                (
                    transform.global_matrix().column(3).xyz(),
                    transform.global_view_matrix(),
                    camera.as_matrix(),
                )
            } else {
                (
                    identity.global_matrix().column(3).xyz(),
                    identity.global_view_matrix(),
                    defcam.as_matrix(),
                )
            };

        let camera_position = convert::<_, Vector3<f32>>(position).into_pod();

        let proj_view: [[f32; 4]; 4] = ((*projection) * view_matrix).into();
        let proj: [[f32; 4]; 4] = (*projection).into();
        let view: [[f32; 4]; 4] = convert::<_, Matrix4<f32>>(view_matrix).into();

        let projview = pod::ViewArgs {
            proj: proj.into(),
            view: view.into(),
            proj_view: proj_view.into(),
        }
        .std140();

        Self {
            camera_position,
            projview,
        }
    }
}

/// If an `AmbientColor` exists in the world, return it - otherwise return pure white.
#[derive(Debug)]
pub struct AmbientGatherer;
impl AmbientGatherer {
    /// If an `AmbientColor` exists in the world, return it - otherwise return pure white.
    pub fn gather(world: &World) -> vec3 {
        let ambient_color = world.resources.get::<AmbientColor>();

        ambient_color.map_or([0.0, 0.0, 0.0].into(), |c| {
            let (r, g, b, _) = c.0.into_components();
            [r, g, b].into()
        })
    }
}
