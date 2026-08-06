pub mod net;

use godot::{
	classes::{
		CanvasItem, Engine, Node, SceneTree, Viewport, Window,
		class_macros::private::virtuals::Os::{NodePath, Rect2},
	},
	meta::{AsArg, ToGodot},
	obj::{Gd, Inherits, Singleton},
};

#[must_use]
pub fn get_camera_rect(viewport: &Gd<Viewport>, canvas_item: &Gd<CanvasItem>) -> Rect2 {
	canvas_item.get_canvas_transform().affine_inverse() * viewport.get_visible_rect()
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
	if let Some(mut scene_tree) = try_get_scene_tree() {
		scene_tree.quit_ex().exit_code(exit_code).done();
	}
}

#[must_use]
pub fn try_get_camera_rect(canvas_item: &Gd<CanvasItem>) -> Option<Rect2> {
	canvas_item
		.get_viewport()
		.map(|viewport| get_camera_rect(&viewport, canvas_item))
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
