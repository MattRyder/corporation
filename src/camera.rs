use cgmath::*;

struct Camera<S>
where
    S: BaseFloat,
{
    position: Point3<S>,
    view: Matrix4<S>,
    projection: Matrix4<S>,
}

impl<S> Default for Camera<S>
where
    S: BaseFloat,
{
    fn default() -> Camera<S> {
        Camera {
            position: Point3::<S>::from_value(S::zero()),
            view: Matrix4::<S>::identity(),
            projection: Matrix4::<S>::identity(),
        }
    }
}

impl<S> Camera<S>
where
    S: BaseFloat,
{

    pub fn get_position(&self) -> Point3<S> {
        self.position
    }

    pub fn get_projection_matrix(&self) -> Matrix4<S> {
        self.projection
    }

    pub fn get_view_matrix(&self) -> Matrix4<S> {
        self.view
    }

    pub fn look_at(&mut self, target: Point3<S>, up: Vector3<S>) {
        self.view = Matrix4::look_at(self.position, target, up);
    }

    pub fn set_position(&mut self, x: S, y: S, z: S) {
        self.position = Point3::<S>::new(x, y, z);
    }

    /// Sets the Camera's projection matrix from the provided params.
    pub fn set_projection_matrix(&mut self, width: S, height: S, fov_deg: S, z_near: S, z_far: S) {
        let perspective_fov = PerspectiveFov {
            fovy: Deg(fov_deg).into(),
            aspect: width / height,
            near: z_near,
            far: z_far,
        };

        self.projection = perspective_fov.into();
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    const POSITION : [f32; 3] = [1.0, 2.0, 3.0];

    #[test]
    fn should_set_projection_matrix() {
        let expected_projection_matrix = Matrix4::<f32>::new(
            1.1772641, 0.0, 0.0, 0.0,
            0.0, 1.5696855, 0.0, 0.0,
            0.0, 0.0, -1.002002, -1.0,
            0.0, 0.0, -0.2002002, 0.0
        );
        let mut camera = Camera::<f32>::default();

        camera.set_projection_matrix(800.0, 600.0, 65.0, 0.1, 100.0);

        assert_eq!(expected_projection_matrix, camera.projection);
    }

    #[test]
    fn should_get_projection_matrix() {
        let expected_projection_matrix = Matrix4::<f32>::identity();
        let camera = Camera::<f32>::default();

        assert_eq!(expected_projection_matrix, camera.get_projection_matrix());
    }

    #[test]
    fn should_set_position() {
        let mut camera = Camera::<f32>::default();

        camera.set_position(POSITION[0], POSITION[1], POSITION[2]);

        assert_eq!(camera.position.x, POSITION[0]);
        assert_eq!(camera.position.y, POSITION[1]);
        assert_eq!(camera.position.z, POSITION[2]);
    }

    #[test]
    fn should_get_position() {
        let mut camera = Camera::<f32>::default();

        camera.position = Point3::<f32>::new(POSITION[0], POSITION[1], POSITION[2]);

        let camera_position = camera.get_position();
        assert_eq!(camera_position.x, POSITION[0]);
        assert_eq!(camera_position.y, POSITION[1]);
        assert_eq!(camera_position.z, POSITION[2]);
    }

    #[test]
    fn should_look_at_point() {
        // let expected_view_matrix = Matrix4::new(
        //     0.94868326, -0.09534626, 0.30151135, 0.0,
        //     0.0, 0.9534626, 0.30151135, 0.0,
        //     -0.31622776, -0.28603876, 0.90453404, 0.0,
        //     -0.0, -0.9534627, -3.6181362, 1.0
        // );

        // let target = Point3::new(0.0, 1.0, 0.0);
        // let up = Vector3::new(0.0, 1.0, 0.0);

        // let mut camera = Camera::<f32>::default();
        // camera.set_position(POSITION[0], POSITION[1], POSITION[2]);
        // camera.look_at(target, up);

        // assert_eq!(expected_view_matrix, camera.view);
    }
}
