use std::collections::HashMap;

use crate::math::prelude::*;

use crate::gfx::scene::{NodeIndex, Scene};

#[derive(Debug, Clone)]
pub enum Channel {
    Translate(Vec<Vector3>),
    Rotate(Vec<Quat>),
    Scale(Vec<Vector3>),
}

#[allow(clippy::len_without_is_empty)]
impl Channel {
    pub fn len(&self) -> usize {
        match self {
            Channel::Translate(t) => t.len(),
            Channel::Rotate(r) => r.len(),
            Channel::Scale(s) => s.len(),
        }
    }
}

#[derive(Default, Clone)]
pub struct Animation {
    pub name: String,
    pub channels: Vec<(Channel, NodeIndex)>,
    pub len: usize,
}

impl Animation {
    pub fn apply(&self, fr: f32, scene: &mut Scene) {
        for (c, node) in &self.channels {
            let n = c.len() - 2;
            let frame = fr * n as f32;

            let i = frame.floor() as usize;
            let j = i + 1;
            let fract = frame.fract();

            // log::info!("{},{} / {} ({} / {})", i, j, n, fr, fract);

            let transform = &mut scene.desc.nodes[*node].transform;
            match c {
                Channel::Translate(t) => {
                    transform.translate = t[i].lerp(t[j], fract)
                },
                Channel::Rotate(r) => {
                    if r[i].dot(r[j]) < 0.0 {
                        transform.rotate = r[i].slerp(-r[j], fract);
                    } else {
                        transform.rotate = r[i].slerp(r[j], fract)
                    }
                },
                Channel::Scale(s) => transform.scale = s[i].lerp(s[j], fract),
            }
        }
    }
}

struct AnimationState {
    pub playing: bool,
    pub looping: bool,
    pub start: f32,
}

pub struct Animator {animations: Vec<Animation>,
    states: Vec<AnimationState>,
    named: HashMap<String, usize>,
}

impl Animator {
    pub fn new(scene: &Scene) -> Self {
        let mut named = HashMap::new();

        let (animations, states) = scene
            .desc
            .animations
            .iter()
            .enumerate()
            .map(|(i, a)| {
                named.insert(a.name.clone(), i);
                (
                    // (),
                    a.desc.clone(),
                    AnimationState {
                        playing: false,
                        looping: false,
                        start: 0.0,
                    },
                )
            })
            .unzip();

        Self {
            animations,
            states,
            named,
        }
    }

    pub fn update(&mut self, t: f32, scene: &mut Scene) {
        for (s, a) in self.states.iter_mut().zip(self.animations.iter()) {
            if s.playing {
                let t = (t - s.start) * 60.0;
                let fr = t / (a.len - 1) as f32;

                if fr >= 1.0 && !s.looping {
                    s.playing = false;
                    continue;
                }

                a.apply(fr % 1.0, scene);
            }
        }
    }

    pub fn play(&mut self, t: f32, looping: bool, animation: &str) {
        let state = &mut self.states[self.named[animation]];

        state.playing = true;
        state.looping = looping;
        state.start = t;
    }

    pub fn stop(&mut self, animation: &str) {
        let state = &mut self.states[self.named[animation]];
        state.playing = false;
    }
}
