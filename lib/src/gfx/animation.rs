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
    pub channels: Vec<(Channel, NodeIndex)>,
    pub len: usize,
}

impl Animation {
    pub fn apply(&self, frame: usize, fr: f32, scene: &mut Scene) {
        let i = frame % self.len;
        let j = (frame + 1) % self.len;

        for (c, node) in &self.channels {
            let transform = &mut scene.desc.nodes[*node].transform;
            match c {
                Channel::Translate(t) => transform.translate = t[i].lerp(t[j], fr),
                Channel::Rotate(r) => transform.rotate = r[i].slerp(r[j], fr),
                Channel::Scale(s) => transform.scale = s[i].lerp(s[j], fr),
            }
        }
    }
}

struct AnimationState {
    pub playing: bool,
    pub looping: bool,
    pub start: f32,
}

pub struct Animator {
    animations: Vec<Animation>,
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
                let t = ((t - s.start) * 60.0);

                let frame = t as usize;
                let fr = t.fract();

                if frame as usize > a.len && !s.looping {
                    s.playing = false;
                    continue;
                }

                a.apply(frame as usize, fr, scene);
            }
        }
    }

    pub fn play(&mut self, t: f32, looping: bool, animation: &str) {
        let state = &mut self.states[self.named[animation]];

        state.playing = true;
        state.looping = looping;
        state.start = t;
    }
}
