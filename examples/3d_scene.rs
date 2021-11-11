use bevy::prelude::*;
use bevy_ultralight::{UltralightInstance, UltralightPlugin};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(UltralightPlugin::default())
        .add_startup_system(setup)
        .add_system(set_startup_content)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });
    // cube
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
    });
    // light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

fn set_startup_content(mut query: Query<&mut UltralightInstance, Added<UltralightInstance>>) {
    for mut instance in query.iter_mut() {
        instance.set_html(r#"
        <html>
        <head>
         <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/jsoneditor@9/dist/jsoneditor.min.css">
         <script src="https://cdn.jsdelivr.net/npm/jsoneditor@9/dist/jsoneditor.min.js"></script>
        </head>

        <body>
            <style>
                p, iframe { opacity: 0.55; }
                #jsoneditor { opacity: 0.85; }
            </style>

         <div id="jsoneditor" style="width: 400px; height: 400px;"></div>

        <script>
            window.__bevy_tick = () => {
                // create the editor
                const container = document.getElementById("jsoneditor")
                const options = {}
                const editor = new JSONEditor(container, options)
                editor.set(window.__bevy_scene)
                const updatedJson = editor.get()

                window.__bevy_tick = null;
            };
        </script>

        <p class="codepen" data-height="300" data-default-tab="result" data-slug-hash="CsoGt" data-user="GabbeV" style="height: 300px; box-sizing: border-box; display: flex; align-items: center; justify-content: center; border: 2px solid; margin: 1em 0; padding: 1em;">
  <span>See the Pen <a href="https://codepen.io/GabbeV/pen/CsoGt">
  requestAnimationFrame fps test</a> by Gabriel Valfridsson (<a href="https://codepen.io/GabbeV">@GabbeV</a>)
  on <a href="https://codepen.io">CodePen</a>.</span>
</p>
<script async src="https://cpwebassets.codepen.io/assets/embed/ei.js"></script>
</body>
</html>
        "#);
    }
}
