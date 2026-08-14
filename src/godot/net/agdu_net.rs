// VERY WIP

#![allow(clippy::similar_names)]

#[allow(clippy::wildcard_imports)]
use crate::{debugging::*, error, info};
use ahash::AHashMap;
use godot::classes::web_socket_peer::State as WebSocketPeerState;
use godot::global::Error as GodotError;
use godot::{
	builtin::VariantType,
	classes::{
		Json, MultiplayerPeer, Time, WebRtcMultiplayerPeer, WebRtcPeerConnection, WebSocketPeer,
		class_macros::private::virtuals::Os::{GString, VarDictionary},
		multiplayer_peer::ConnectionStatus,
	},
	meta::ToGodot,
	obj::NewGd,
	obj::Singleton,
	prelude::*,
};

#[derive(GodotClass)]
#[class(base=Node)]
pub struct AgduNet {
	ws: Gd<WebSocketPeer>,
	code: i32,
	reason: GString,
	old_state: WebSocketPeerState,
	keepalive_pings: bool,

	rtc_mp: Gd<WebRtcMultiplayerPeer>,

	own_peer_id: i32,
	own_server_peer_id: GString,
	is_host: bool,
	lobby_password: GString,
	peer_id_map: AHashMap<String, i32>,
	next_godot_id: i32,
	last_keepalive_ping: u64,

	base: Base<Node>,
}

#[godot_api]
impl INode for AgduNet {
	fn init(base: Base<Node>) -> Self {
		Self {
			ws: WebSocketPeer::new_gd(),
			code: 1000,
			reason: "Unknown".to_godot_owned(),
			old_state: WebSocketPeerState::CLOSED,
			keepalive_pings: false,

			rtc_mp: WebRtcMultiplayerPeer::new_gd(),

			own_peer_id: 0,
			own_server_peer_id: "".to_godot_owned(),
			is_host: false,
			lobby_password: "".to_godot_owned(),
			peer_id_map: AHashMap::new(),
			next_godot_id: 2,
			last_keepalive_ping: 0,

			base,
		}
	}

	fn process(&mut self, _delta: f64) {
		self.ws.poll();

		let state = self.ws.get_ready_state();

		if state == WebSocketPeerState::OPEN && self.old_state != WebSocketPeerState::OPEN {
			info!("WebSocket connected to server");
		}

		while state == WebSocketPeerState::OPEN && self.ws.get_available_packet_count() > 0 {
			let _ = self.parse_msg();
		}

		if state == WebSocketPeerState::CLOSED && self.old_state != WebSocketPeerState::CLOSED {
			self.code = self.ws.get_close_code();
			self.reason = self.ws.get_close_reason();

			info!("WebSocket closed: {} - {}", self.code, self.reason);
			self.disconnected();
		}

		self.old_state = state;

		if self.keepalive_pings {
			let now = Time::singleton().get_ticks_msec();

			if now >= self.last_keepalive_ping + 30000 {
				self.last_keepalive_ping = now;
				self.send_keepalive_ping();
			}
		}
	}
}

#[godot_api]
impl AgduNet {
	#[signal]
	fn connection_successful();

	#[signal]
	fn disconnected();

	#[signal]
	fn toast(message: GString);

	#[allow(clippy::needless_pass_by_value)]
	#[func]
	pub fn start(&mut self, url: GString, _room_code: GString, password: GString, as_host: bool) {
		self.stop();
		self.is_host = as_host;
		info!(
			"Starting as {}, connecting to: {url}",
			if as_host { "host" } else { "client" }
		);
		self.connect_to_url(&url.to_string(), &password.to_string());
		self.keepalive_pings = true;
	}

	#[func]
	pub fn stop(&mut self) {
		let Some(mut multiplayer) = self.base().get_multiplayer() else {
			return;
		};
		let none: Option<&Gd<MultiplayerPeer>> = None;
		multiplayer.set_multiplayer_peer(none);
		self.rtc_mp.close();
		self.rtc_mp = WebRtcMultiplayerPeer::new_gd();
		self.close();
	}

	#[func]
	pub fn stop_keepalive_pings(&mut self) {
		self.keepalive_pings = false;
	}

	fn answer_received(&mut self, godot_id: i32, answer: &str) {
		if self.rtc_mp.has_peer(godot_id)
			&& let Some(connection) = self.rtc_mp.get_peer(godot_id).get("connection")
			&& let Ok(mut connection) = connection.try_to::<Gd<WebRtcPeerConnection>>()
		{
			connection.set_remote_description("answer", answer);
		}
	}

	fn assign_godot_id(&mut self, server_id: &str, role_is_host: bool) -> i32 {
		if let Some(existing) = self.peer_id_map.get(server_id) {
			if role_is_host == (*existing == 1) {
				return *existing;
			}
			self.peer_id_map.remove(server_id);
		}

		if role_is_host {
			self.peer_id_map.insert(server_id.to_owned(), 1);
			return 1;
		}

		let godot_id = self.next_godot_id;
		self.next_godot_id += 1;
		self.peer_id_map.insert(server_id.to_owned(), godot_id);
		godot_id
	}

	fn candidate_received(&mut self, godot_id: i32, mid: &str, index: i32, sdp: &str) {
		if self.rtc_mp.has_peer(godot_id)
			&& let Some(connection) = self.rtc_mp.get_peer(godot_id).get("connection")
			&& let Ok(mut connection) = connection.try_to::<Gd<WebRtcPeerConnection>>()
		{
			connection.add_ice_candidate(mid, index, sdp);
		}
	}

	fn close(&mut self) {
		self.ws.close();
	}

	fn connect_to_url(&mut self, url: &str, password: &str) {
		self.close();
		self.code = 1000;
		self.reason = "Unknown".to_godot_owned();
		self.lobby_password = password.to_godot_owned();
		let result = self.ws.connect_to_url(url);
		if result != GodotError::OK {
			error!("Failed to start WebSocket connection: {result:?}");
		}
	}

	fn connected(&mut self, godot_id: i32) {
		let result = if self.is_host || godot_id == 1 {
			self.rtc_mp.create_server()
		} else {
			self.rtc_mp.create_client(godot_id)
		};

		if result == GodotError::OK {
			let Some(mut multiplayer) = self.base().get_multiplayer() else {
				return;
			};
			multiplayer.set_multiplayer_peer(&self.rtc_mp);
			self.signals().connection_successful().emit();
		} else {
			error!("Failed to configure rtc_mp!");
		}
	}

	fn create_peer(&mut self, godot_id: i32) -> Gd<WebRtcPeerConnection> {
		let mut peer = WebRtcPeerConnection::new_gd();

		if self.rtc_mp.get_connection_status() == ConnectionStatus::CONNECTED
			&& !self.is_host
			&& godot_id != 1
		{
			error!("Client tried to add non-host peer {godot_id}");
			return peer;
		}

		let mut configuration = VarDictionary::new();
		let mut ice_servers = VarArray::new();
		let mut ice_server = VarDictionary::new();
		let mut urls = VarArray::new();
		urls.push(&"stun:stun.l.google.com:19302".to_godot_owned());
		let _ = ice_server.insert("urls", &urls);
		ice_servers.push(&ice_server);
		let _ = configuration.insert("iceServers", &ice_servers);
		let init_result = peer
			.initialize_ex()
			.configuration(&configuration.upcast_any_dictionary())
			.done();

		if init_result != GodotError::OK {
			error!("Failed to initialize peer connection: {init_result:?}");
			return peer;
		}

		peer.signals().session_description_created().connect_other(
			self,
			move |this, type_, sdp| {
				this.offer_created(
					type_.to_string().as_str(),
					sdp.to_string().as_str(),
					godot_id,
				);
			},
		);
		peer.signals().ice_candidate_created().connect_other(
			self,
			move |this, media, index, name| {
				this.new_ice_candidate(
					media.to_string().as_str(),
					index,
					name.to_string().as_str(),
					godot_id,
				);
			},
		);
		let add_result = self.rtc_mp.add_peer(&peer, godot_id);
		if add_result != GodotError::OK {
			error!("Failed to add peer: {add_result:?}");
			return peer;
		}

		if self.is_host {
			peer.create_offer();
		}

		peer
	}

	fn disconnected(&mut self) {
		info!("Disconnected: {}: {}", self.code, self.reason);
		self.stop();
		self.signals().connection_successful().emit();
	}

	fn get_godot_id(&self, server_id: &str) -> Option<i32> {
		if let Some(id) = self.peer_id_map.get(server_id) {
			return Some(*id);
		}
		None
	}

	fn get_server_id(&self, godot_id: i32) -> Option<String> {
		for (k, v) in &self.peer_id_map {
			if *v == godot_id {
				return Some(k.clone());
			}
		}
		None
	}

	fn new_ice_candidate(&mut self, mid: &str, index: i64, sdp: &str, godot_id: i32) {
		self.send_candidate(godot_id, mid, index, sdp);
	}

	fn offer_created(&mut self, type_: &str, data: &str, godot_id: i32) {
		if !self.rtc_mp.has_peer(godot_id) {
			error!("Peer {godot_id} not in rtc_mp, can't set local description");
			return;
		}

		if let Some(connection) = self.rtc_mp.get_peer(godot_id).get("connection")
			&& let Ok(mut connection) = connection.try_to::<Gd<WebRtcPeerConnection>>()
		{
			connection.set_local_description(type_, data);
			if type_ == "offer" {
				self.send_offer(godot_id, data);
			} else {
				self.send_answer(godot_id, data);
			}
		}
	}

	fn offer_received(&mut self, godot_id: i32, offer: &str) {
		if !self.rtc_mp.has_peer(godot_id) {
			let mut peer = self.create_peer(godot_id);
			if self.rtc_mp.has_peer(godot_id) {
				peer.set_remote_description("offer", offer);
			} else {
				error!("Failed to add peer, can't process offer");
			}
		} else if let Some(connection) = self.rtc_mp.get_peer(godot_id).get("connection")
			&& let Ok(mut connection) = connection.try_to::<Gd<WebRtcPeerConnection>>()
		{
			connection.set_remote_description("offer", offer);
			connection.create_offer();
		}
	}

	#[allow(clippy::too_many_lines)]
	fn parse_msg(&mut self) -> bool {
		let packet = self.ws.get_packet();
		let text = packet.get_string_from_utf8();
		let parsed = Json::parse_string(&text);

		if parsed.get_type() != VariantType::DICTIONARY {
			return false;
		}
		let Ok(parsed) = parsed.try_to::<VarDictionary>() else {
			return false;
		};
		let Some(type_variant) = parsed.get("type") else {
			return false;
		};
		if type_variant.get_type() != VariantType::STRING {
			return false;
		}
		let Ok(type_gstring) = type_variant.try_to::<GString>() else {
			return false;
		};
		let type_string = type_gstring.to_string();
		match type_string.as_str() {
			"room_created" => {
				error!("Message was not handled: \"{type_string}\"");
			}

			"joined_room" => {
				let Some(peer_id_variant) = parsed.get("peer_id") else {
					return false;
				};
				if peer_id_variant.get_type() != VariantType::STRING {
					return false;
				}
				let Ok(peer_id_gstring) = peer_id_variant.try_to::<GString>() else {
					return false;
				};
				self.own_server_peer_id = peer_id_gstring;
				let peer_id_string = self.own_server_peer_id.to_string();
				self.own_peer_id = self.assign_godot_id(peer_id_string.as_str(), self.is_host);
				info!(
					"Joined room. Server ID: {peer_id_string}, Godot ID: {}",
					self.own_peer_id,
				);
				let mut message = VarDictionary::new();
				if self.is_host {
					let _ = message.insert(&"type".to_godot_owned(), &"set_role".to_godot_owned());
					let _ = message.insert(&"role".to_godot_owned(), &"host".to_godot_owned());
					let _ = message.insert(&"password".to_godot_owned(), &self.lobby_password);
				} else {
					let _ = message.insert(&"type".to_godot_owned(), &"set_role".to_godot_owned());
					let _ = message.insert(&"role".to_godot_owned(), &"client".to_godot_owned());
					let _ = message.insert(&"password".to_godot_owned(), &self.lobby_password);
				}
				self.send_ws_message(&message);
			}

			"peer_joined" => {
				let Some(peer_id_variant) = parsed.get("peer_id") else {
					return false;
				};
				if peer_id_variant.get_type() != VariantType::STRING {
					return false;
				}
				let Ok(peer_id_gstring) = peer_id_variant.try_to::<GString>() else {
					return false;
				};
				let Some(role_variant) = parsed.get("role") else {
					return false;
				};
				if role_variant.get_type() != VariantType::STRING {
					return false;
				}
				let Ok(role_gstring) = role_variant.try_to::<GString>() else {
					return false;
				};
				info!("Peer joined. Server ID: {peer_id_gstring}, role: {role_gstring}");
				match role_gstring.to_string().as_str() {
					"host" => {
						if peer_id_gstring == self.own_server_peer_id {
							self.connected(1);
							// self.peer_connected(1);
						}
					}

					"client" => {
						if peer_id_gstring == self.own_server_peer_id {
							let own_godot_id = self.own_peer_id;
							self.connected(own_godot_id);
							self.peer_connected(own_godot_id);
						} else {
							let godot_id =
								self.assign_godot_id(peer_id_gstring.to_string().as_str(), false);
							self.peer_connected(godot_id);
						}
					}

					_ => {}
				}
			}

			"peer_left" => {
				let Some(peer_id_variant) = parsed.get("peer_id") else {
					return false;
				};
				if peer_id_variant.get_type() != VariantType::STRING {
					return false;
				}
				let Ok(peer_id_gstring) = peer_id_variant.try_to::<GString>() else {
					return false;
				};
				let peer_id_string = peer_id_gstring.to_string();
				if let Some(godot_id) = self.get_godot_id(peer_id_string.as_str()) {
					info!("Peer left: Godot ID {godot_id}");
					self.peer_disconnected(godot_id);
					self.remove_godot_id(peer_id_string.as_str());
				}
			}

			"sdp_offer_received" => {
				let Some(from_peer_id_variant) = parsed.get("from") else {
					return false;
				};
				if from_peer_id_variant.get_type() != VariantType::STRING {
					return false;
				}
				let Ok(from_peer_id_gstring) = from_peer_id_variant.try_to::<GString>() else {
					return false;
				};
				let from_peer_id_string = from_peer_id_gstring.to_string();
				if let Some(from_godot_id) = self.get_godot_id(from_peer_id_string.as_str()) {
					let Some(sdp_variant) = parsed.get("sdp") else {
						return false;
					};
					if sdp_variant.get_type() != VariantType::STRING {
						return false;
					}
					let Ok(sdp_gstring) = sdp_variant.try_to::<GString>() else {
						return false;
					};
					let sdp_string = sdp_gstring.to_string();
					self.offer_received(from_godot_id, sdp_string.as_str());
				}
			}

			"sdp_answer_received" => {
				let Some(from_peer_id_variant) = parsed.get("from") else {
					return false;
				};
				if from_peer_id_variant.get_type() != VariantType::STRING {
					return false;
				}
				let Ok(from_peer_id_gstring) = from_peer_id_variant.try_to::<GString>() else {
					return false;
				};
				let from_peer_id_string = from_peer_id_gstring.to_string();
				if let Some(from_godot_id) = self.get_godot_id(from_peer_id_string.as_str()) {
					let Some(sdp_variant) = parsed.get("sdp") else {
						return false;
					};
					if sdp_variant.get_type() != VariantType::STRING {
						return false;
					}
					let Ok(sdp_gstring) = sdp_variant.try_to::<GString>() else {
						return false;
					};
					let sdp_string = sdp_gstring.to_string();
					self.answer_received(from_godot_id, sdp_string.as_str());
				}
			}

			"ice_candidate_received" => {
				let Some(from_peer_id_variant) = parsed.get("from") else {
					return false;
				};
				if from_peer_id_variant.get_type() != VariantType::STRING {
					return false;
				}
				let Ok(from_peer_id_gstring) = from_peer_id_variant.try_to::<GString>() else {
					return false;
				};
				let from_peer_id_string = from_peer_id_gstring.to_string();
				if let Some(from_godot_id) = self.get_godot_id(from_peer_id_string.as_str()) {
					let Some(candidate_variant) = parsed.get("candidate") else {
						return false;
					};
					if candidate_variant.get_type() != VariantType::STRING {
						return false;
					}
					let Ok(candidate_gstring) = candidate_variant.try_to::<GString>() else {
						return false;
					};
					let candidate_string = candidate_gstring.to_string();
					let mut sdp_mid_string = "0".to_owned();
					if let Some(sdp_mid_variant) = parsed.get("sdp_mid")
						&& sdp_mid_variant.get_type() == VariantType::STRING
						&& let Ok(sdp_mid_gstring) = sdp_mid_variant.try_to::<GString>()
					{
						sdp_mid_string = sdp_mid_gstring.to_string();
					}
					let mut sdp_mline_index = 0;
					if let Some(sdp_mline_index_variant) = parsed.get("sdp_mline_index")
						&& sdp_mline_index_variant.get_type() == VariantType::INT
						&& let Ok(sdp_mline_index_i32) = sdp_mline_index_variant.try_to::<i32>()
					{
						sdp_mline_index = sdp_mline_index_i32;
					}
					self.candidate_received(
						from_godot_id,
						sdp_mid_string.as_str(),
						sdp_mline_index,
						candidate_string.as_str(),
					);
				}
			}

			"pong" => {
				// Heartbeat, ignore
			}

			"password_required" => {
				self.signals()
					.toast()
					.emit(&"Password missing".to_godot_owned());
			}

			"wrong_password" => {
				self.signals()
					.toast()
					.emit(&"Wrong password".to_godot_owned());
			}

			"error" => {
				let Some(message_variant) = parsed.get("message") else {
					return false;
				};
				if message_variant.get_type() != VariantType::STRING {
					return false;
				}
				let Ok(message_gstring) = message_variant.try_to::<GString>() else {
					return false;
				};
				let message_string = message_gstring.to_string();
				error!("Server error: {message_string}");
			}

			_ => {
				error!("Unknown message type: {type_string}");
				return false;
			}
		}

		true
	}

	fn peer_connected(&mut self, godot_id: i32) {
		if self.is_host && godot_id != 1 {
			if self.rtc_mp.get_connection_status() == ConnectionStatus::DISCONNECTED {
				error!("rtc_mp still disconnected! Can't create peer.");
				error!("This likely means _connected hasn't been called or failed.");
			} else {
				self.create_peer(godot_id);
			}
		} else {
			error!("Not creating peer (host={}, id={godot_id})", self.is_host);
		}
	}

	fn peer_disconnected(&mut self, godot_id: i32) {
		info!("Peer disconnected: {godot_id}");
		if self.rtc_mp.has_peer(godot_id) {
			self.rtc_mp.remove_peer(godot_id);
		}
	}

	fn remove_godot_id(&mut self, server_id: &str) {
		let _ = self.peer_id_map.remove(server_id);
	}

	fn send_answer(&mut self, godot_id: i32, answer: &str) -> GodotError {
		let Some(server_id) = self.get_server_id(godot_id) else {
			error!("No server ID mapping for Godot ID {godot_id}",);
			return GodotError::ERR_DOES_NOT_EXIST;
		};

		let mut message = VarDictionary::new();
		let _ = message.insert(&"type".to_godot_owned(), &"sdp_answer".to_godot_owned());
		let _ = message.insert(&"to".to_godot_owned(), &server_id.to_godot_owned());
		let _ = message.insert(&"sdp".to_godot_owned(), answer);
		self.send_ws_message(&message)
	}

	fn send_candidate(&mut self, godot_id: i32, mid: &str, index: i64, sdp: &str) -> GodotError {
		let Some(server_id) = self.get_server_id(godot_id) else {
			error!("No server ID mapping for Godot ID {godot_id}",);
			return GodotError::ERR_DOES_NOT_EXIST;
		};

		let mut message = VarDictionary::new();
		let _ = message.insert(&"type".to_godot_owned(), &"ice_candidate".to_godot_owned());
		let _ = message.insert(&"to".to_godot_owned(), &server_id.to_godot_owned());
		let _ = message.insert(&"candidate".to_godot_owned(), sdp);
		let _ = message.insert(&"sdp_mid".to_godot_owned(), mid);
		let _ = message.insert(&"sdp_mline_index".to_godot_owned(), index);
		self.send_ws_message(&message)
	}

	fn send_keepalive_ping(&mut self) {
		let mut message = VarDictionary::new();
		let _ = message.insert(&"type".to_godot_owned(), &"ping".to_godot_owned());
		let err = self.send_ws_message(&message);
		if err == GodotError::ERR_CONNECTION_ERROR {
			let state = self.ws.get_ready_state();
			if state != WebSocketPeerState::CONNECTING {
				self.keepalive_pings = false;
			}
		}
	}

	fn send_offer(&mut self, godot_id: i32, offer: &str) -> GodotError {
		let Some(server_id) = self.get_server_id(godot_id) else {
			error!("No server ID mapping for Godot ID {godot_id}",);
			return GodotError::ERR_DOES_NOT_EXIST;
		};

		let mut message = VarDictionary::new();
		let _ = message.insert(&"type".to_godot_owned(), &"sdp_offer".to_godot_owned());
		let _ = message.insert(&"to".to_godot_owned(), &server_id.to_godot_owned());
		let _ = message.insert(&"sdp".to_godot_owned(), offer);
		self.send_ws_message(&message)
	}

	fn send_ws_message(&mut self, message: &VarDictionary) -> GodotError {
		if self.ws.get_ready_state() == WebSocketPeerState::OPEN {
			let json = Json::stringify(&message.to_variant());
			return self.ws.send_text(&json);
		}
		GodotError::ERR_CONNECTION_ERROR
	}
}
