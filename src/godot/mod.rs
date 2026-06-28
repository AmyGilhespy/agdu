use godot::{
	classes::{Engine, Node, SceneTree, Window, class_macros::private::virtuals::Os::NodePath},
	meta::{AsArg, ToGodot},
	obj::{Gd, Inherits, Singleton},
};

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
	if let Some(mut scene_tree) = try_get_scene_tree() {
		scene_tree.quit_ex().exit_code(exit_code).done();
	}
}

#[must_use]
pub fn try_get_rooted_node_as<T>(path: impl AsArg<NodePath>) -> Option<Gd<T>>
where
	T: Inherits<Node>,
{
	try_get_scene_root().and_then(|r| r.try_get_node_as::<T>(path))
}

#[must_use]
pub fn try_get_scene_root() -> Option<Gd<Window>> {
	try_get_scene_tree().and_then(|s| s.get_root())
}

#[must_use]
pub fn try_get_scene_tree() -> Option<Gd<SceneTree>> {
	Engine::singleton()
		.get_main_loop()
		.and_then(|m| m.try_cast::<SceneTree>().ok())
}
