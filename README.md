# PHANTOMa

<table>
  <tr>
    <td><img src="https://foltz.io/img/nyxt.webp"></td>
    <td><img src="https://foltz.io/img/phantoma.webp"></td>
    <td><img src="https://foltz.io/img/neurotic.webp"></td>
  </tr>
  <tr>
    <td><a href="https://youtu.be/J3St66XFQzE">https://youtu.be/J3St66XFQzE</a></td>
    <td><a href="https://youtu.be/nrh4Sin6PLM">https://youtu.be/nrh4Sin6PLM</a></td>
    <td><a href="https://youtu.be/Ow8Ewml8fz0">https://youtu.be/Ow8Ewml8fz0</a></td>
  </tr>
 </table>

A music reactive environment for creating live visual effects with a custom WebGPU rendering pipeline and audio analysis tools.

## Requirements

PHANTOMa requires a nightly rust compiler.

When cloning, be sure to also clone the required submodules: 

```sh
git clone --recurse-submodules https://github.com/Foltik/PHANTOMa
```

## Goals
### App Framework
- [X] Async event loop
- [X] Multiple windows
### Resources
- [X] Load resources from crate root
- [ ] More granular control of resource loading
### Rendering
- [X] WebGPU Wrappers
- [X] Data driven pipelines with spirv-reflect
- [X] Data driven UI generation
- [X] Composable scenes
- [ ] Asset pipeline
- [ ] Ray tracing
- [ ] Expose more native vulkan extensions
### Audio
- [X] Send/Receive with JACK
- [X] FFT
- [X] Basic frequency range beat detection
- [X] More advanced beat detection algorithms
- [ ] Pitch detection
### Video
- [ ] Playback
- [ ] Scrubbing / Timewarp
### MIDI
- [X] Send/Receive with JACK
- [X] Separate implementation from midir
- [X] Swappable profiles with generic outputs
- [X] Launchpad control library
### OSC
- [X] Send/Receive with nannou_osc
- [X] Mixxx plugin
- [X] VirtualDJ plugin
- [ ] Supercollider Control
### E.131/sACN
- [X] Send packets to lighting hardware
- [ ] Receive packets
