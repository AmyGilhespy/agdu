use godot::{
	classes::{Engine, SceneTree, Window},
	obj::{Gd, Singleton},
};

#[must_use]
pub fn get_scene_root() -> Option<Gd<Window>> {
	get_scene_tree().and_then(|s| s.get_root())
}

#[must_use]
pub fn get_scene_tree() -> Option<Gd<SceneTree>> {
	Engine::singleton()
		.get_main_loop()
		.and_then(|m| m.try_cast::<SceneTree>().ok())
}
