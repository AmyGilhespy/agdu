use godot::{
	classes::{Engine, SceneTree, Window},
	meta::ToGodot,
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

pub fn push_error(message: &str) {
	godot::global::push_error(&[message.to_variant()]);
}

pub fn push_warning(message: &str) {
	godot::global::push_warning(&[message.to_variant()]);
}

pub fn print(message: &str) {
	godot::global::print(&[message.to_variant()]);
}

pub fn print_rich(message: &str) {
	godot::global::print_rich(&[message.to_variant()]);
}

pub fn quit(exit_code: i32) {
	if let Some(mut scene_tree) = get_scene_tree() {
		scene_tree.quit_ex().exit_code(exit_code).done();
	}
}
