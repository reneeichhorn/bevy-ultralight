use std::sync::{Arc, Mutex};

use bevy::{
    input::mouse::{MouseButtonInput, MouseMotion},
    prelude::{shape::Plane, *},
    reflect::{TypeRegistry, TypeUuid},
    render::{
        pipeline::*,
        render_graph::{base::node::MAIN_PASS, AssetRenderResourcesNode, RenderGraph},
        renderer::RenderResources,
        shader::*,
    },
    scene::serde::SceneSerializer,
};

mod ultralight;
use ultralight::Ultralight;

/// Ultralights main plugin that will add all needed resources and systems.
/// Right now this by default automatically spawns a always on top webview that is transparently rendering on top of your scene.
#[derive(Default)]
pub struct UltralightPlugin {}

/// System label for our systems.
#[derive(SystemLabel, Clone, Hash, Debug, PartialEq, Eq)]
pub enum Label {
    /// Executed every frame to update the view.
    /// Usually you would want your system to execute before `Tick`.
    Tick,
    /// The initialization system.
    Init,
}

impl Plugin for UltralightPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<WebMaterial>();
        app.add_startup_system_to_stage(StartupStage::PostStartup, initialize.label(Label::Init));
        app.add_system(tick.label(Label::Tick));
        app.add_system(user_input.before(Label::Tick));
        app.add_system(handle_ecs_sync.exclusive_system());
    }
}

/// A component that holds a webview and acts as an interface to that webview.
#[derive(Component)]
pub struct UltralightInstance {
    instance: Mutex<Ultralight>,
    state: TextureState,
}

impl UltralightInstance {
    /// Updates the html of the webview
    pub fn set_html(&mut self, html: &str) {
        let instance = self.instance.lock().unwrap();
        instance.load_html(html);
    }
}

#[derive(Clone)]
enum TextureState {
    None,
    Pending(Arc<Mutex<Option<Texture>>>),
}

#[derive(Debug, RenderResources, TypeUuid, Clone, Default)]
#[uuid = "a57878c4-569e-4511-be7c-b0e5b2c983e2"]
struct WebMaterial {
    color: Handle<Texture>,
}

fn initialize(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
    mut materials: ResMut<Assets<WebMaterial>>,
    mut render_graph: ResMut<RenderGraph>,
    windows: Res<Windows>,
) {
    // Extend render graph.
    render_graph.add_system_node(
        "WebMaterial",
        AssetRenderResourcesNode::<WebMaterial>::new(true),
    );
    render_graph
        .add_node_edge("WebMaterial", MAIN_PASS)
        .unwrap();

    // Init  instance
    let window = windows.get_primary().unwrap();
    let instance = Ultralight::new(window.width() as u32, window.height() as u32);
    let instance = UltralightInstance {
        instance: Mutex::new(instance),
        state: TextureState::None,
    };

    // Create a new shader pipeline
    let mut pipeline_desc = PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(
            ShaderStage::Vertex,
            include_str!("./shader.vert"),
        )),
        fragment: Some(shaders.add(Shader::from_glsl(
            ShaderStage::Fragment,
            include_str!("./shader.frag"),
        ))),
    });
    pipeline_desc.primitive.cull_mode = None;
    if let Some(depth) = &mut pipeline_desc.depth_stencil {
        depth.depth_write_enabled = false;
    }

    let pipeline_handle = pipelines.add(pipeline_desc);
    let material = materials.add(WebMaterial::default());

    commands
        .spawn_bundle(MeshBundle {
            mesh: meshes.add((Plane { size: 2.0 }).into()),
            visible: Visible {
                is_transparent: true,
                is_visible: true,
            },
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                pipeline_handle,
            )]),
            ..Default::default()
        })
        .insert(material)
        .insert(instance);
}

fn user_input(
    windows: Res<Windows>,
    mut query: Query<&mut UltralightInstance>,
    mut motion_events: EventReader<MouseMotion>,
    mut mousebtn_events: EventReader<MouseButtonInput>,
) {
    let window = windows.get_primary().unwrap();

    // Y-axis is reversed in ultralight
    let cursor = window
        .cursor_position()
        .map(|position| Vec2::new(position.x, window.height() - position.y));

    if let Some(instance) = query.iter_mut().next() {
        // Handle Mouse movements
        let ul_instance = instance.instance.lock().unwrap();
        if motion_events.iter().next().is_some() {
            if let Some(cursor) = cursor {
                ul_instance.fire_mouse_motion_event(cursor);
            }
        }

        // Handle Mouse buttons
        for event in mousebtn_events.iter() {
            if let Some(cursor) = cursor {
                ul_instance.fire_mouse_button_event(cursor, event.button, event.state);
            }
        }
    }
}

fn handle_ecs_sync(world: &mut World) {
    let type_registry = world.get_resource::<TypeRegistry>().unwrap();
    let scene = DynamicScene::from_world(world, type_registry);
    let serializer = SceneSerializer::new(&scene, type_registry);
    let json = serde_json::to_string(&serializer).unwrap();

    let mut query = world.query::<&mut UltralightInstance>();
    for instance in query.iter_mut(world) {
        let ul_instance = instance.instance.lock().unwrap();
        ul_instance.execute_javascript(
            format!(
                "window.__bevy_scene = {}; if (window.__bevy_tick) {{ window.__bevy_tick() }}",
                json
            )
            .as_str(),
        );
    }
}

fn tick(
    mut query: Query<(&mut UltralightInstance, &mut Handle<WebMaterial>)>,
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<WebMaterial>>,
) {
    for (mut instance, mut material) in query.iter_mut() {
        // Update internal timers etc.
        {
            let ul_instance = instance.instance.lock().unwrap();
            ul_instance.update();
        }

        // Update texture.
        match instance.state.clone() {
            TextureState::None => {
                let texture = {
                    let ul_instance = instance.instance.lock().unwrap();
                    ul_instance.receive_texture_buffer()
                };
                instance.state = TextureState::Pending(texture);
            }
            TextureState::Pending(texture) => {
                let mut texture = texture.lock().unwrap();
                if let Some(texture) = texture.take() {
                    instance.state = TextureState::None;

                    *material = materials.add(WebMaterial {
                        color: textures.add(texture),
                    });
                }
            }
        }
    }
}
