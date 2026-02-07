use bevy::prelude::*;
use bevy_bullet_dynamics::prelude::*;
use bevy_renet2::prelude::*;
use bevy_renet2::netcode::{
    NetcodeServerPlugin, NetcodeClientPlugin, ServerSetupConfig, ClientAuthentication, 
    NetcodeServerTransport, NetcodeClientTransport
};
use renet2::ConnectionConfig;
use renet2_netcode::NativeSocket;
use std::time::SystemTime;
use std::net::{UdpSocket, SocketAddr};

// Feature gate entire example
#[cfg(feature = "netcode")]
fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(BallisticsPluginGroup); 
    
    let args: Vec<String> = std::env::args().collect();
    let is_server = args.contains(&"server".to_string());
    
    if is_server {
        setup_server(&mut app);
    } else {
        setup_client(&mut app);
    }
    
    app.run();
}

#[cfg(not(feature = "netcode"))]
fn main() {
    println!("Please enable 'netcode' feature to run this example.");
}

#[cfg(feature = "netcode")]
fn setup_server(app: &mut App) {
    app.add_plugins(RenetServerPlugin);
    app.add_plugins(NetcodeServerPlugin);
    app.add_plugins(bevy_bullet_dynamics::network::server::BallisticsServerPlugin);

    let server_addr: SocketAddr = "127.0.0.1:5000".parse().unwrap();
    let udp_socket = UdpSocket::bind(server_addr).unwrap();
    udp_socket.set_nonblocking(true).unwrap(); // Important for renet? Usually yes.
    let socket = NativeSocket::new(udp_socket).unwrap();
    
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    
    let server_setup_config = ServerSetupConfig {
        current_time,
        max_clients: 64,
        protocol_id: bevy_bullet_dynamics::network::protocol::PROTOCOL_ID,
        socket_addresses: vec![vec![server_addr]], 
        authentication: bevy_renet2::netcode::ServerAuthentication::Unsecure,
    };

    let transport = NetcodeServerTransport::new(server_setup_config, socket).unwrap();
    app.insert_resource(transport);
    
    let renet_server = RenetServer::new(ConnectionConfig::test());
    app.insert_resource(renet_server);
    
    app.add_systems(Startup, setup_scene_server);
}

#[cfg(feature = "netcode")]
fn setup_client(app: &mut App) {
    app.add_plugins(RenetClientPlugin);
    app.add_plugins(NetcodeClientPlugin);
    app.add_plugins(bevy_bullet_dynamics::network::client::BallisticsClientPlugin);

    let server_addr: SocketAddr = "127.0.0.1:5000".parse().unwrap();
    let udp_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    udp_socket.set_nonblocking(true).unwrap();
    let socket = NativeSocket::new(udp_socket).unwrap();
    
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let client_id = current_time.as_millis() as u64; 
    
    let authentication = ClientAuthentication::Unsecure {
        server_addr,
        client_id,
        user_data: None,
        protocol_id: bevy_bullet_dynamics::network::protocol::PROTOCOL_ID,
        socket_id: 0, 
    };
    
    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();
    app.insert_resource(transport);
    
    let renet_client = RenetClient::new(ConnectionConfig::test(), false); 
    app.insert_resource(renet_client);
    
    app.add_systems(Startup, setup_scene_client);
}

fn setup_scene_server(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    
    commands.spawn((
        Mesh3d(meshes.add(Plane3d { half_size: Vec2::splat(20.0), ..default() })),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

fn setup_scene_client(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
