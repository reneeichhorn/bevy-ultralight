# bevy-ultralight

[![ultralight 0.2.1](https://img.shields.io/badge/ultralight-0.2.1-green)](https://github.com/ultralight-ux/Ultralight/releases/tag/v1.2.1)

`bevy-ultralight` creates a webview on top of the primary window and renders any given webkit capable (html, videos, images, ..) content on top of your scene. The root DOM document is transparent.

# Usage

```rust
// In main add the plugin.
app.add_plugin(UltralightPlugin::default())

// The plugin automatically spawns a always on top webview that can be queried
fn set_content(mut query: Query<&mut UltralightInstance, Added<UltralightInstance>>) {
    for mut instance in query.iter_mut() {
        instance.set_html("<h1>Hello World!</h1>");
    }
}

```

For further usage info have a look at the example at `examples/3d_scene.rs`.

## Building

`bevy-ultralight` relies on the proprietary library [ultralight](https://github.com/ultralight-ux/Ultralight) you will need to download binaries from ultralights github page (linked in the badge)

Once downloaded and extracted make sure that the ultralight folder is set to the `ULTRALIGHT` environment variable and is available for cargo during building. This ensures all files can be found during linking.

`ultralight` is dynamically linked so make sure your built program has access to the files in the `/bin` folder of `ultralight`. For development purpose you can add these files to your workspace root.
