use rapier3d::{control::KinematicCharacterController, prelude::RigidBodyHandle};

pub struct CharacterController {
    body: Option<RigidBodyHandle>,
    controller: rapier3d::control::KinematicCharacterController,
}
