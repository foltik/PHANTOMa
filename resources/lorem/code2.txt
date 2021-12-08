neuro_setup(() => {
    neuro_source('in');
    let canvas = createCanvas(1280, 720, WEBGL);
    smooth();
});

const [rows, cols] = [50, 50, 10];

neuro_init(() => {
    const bpm = 250;
    let onset = new OnsetDetect(40, 120, 0.025, bpm);
    let decay = new Decay(1, 1.05, 1 / (bpm / 60), 50);
    neuro_set_all({onset, decay});
});

neuro_draw(() => {
    const t = neuro_get('t', 0);
    const tc = neuro_get('tc', 0);
    const [onset, decay] = neuro_get_all('onset', 'decay');

    onset.detect() && decay.set(1.05);

    let terrain = range(rows).amap(i =>
        range(cols).amap(j =>
            map(noise(i * 0.1, j * 0.1 + t * 0.2), 0, 1, -10, 10)));

    const [r, g, b] = rgb(t * 0.01, map(decay.get(), 1, 1.05, 0.8, 1), 1);
    stroke(r, g, b);

    background(0);
    rotateX(radians(60));
    translate(1280 / -2, 720 / -2);

    translate(150, -450, -150);

    scale(20);

    range(rows - 1).each(i => {
        noFill();
        beginShape(TRIANGLE_STRIP);
        range(cols).each(j => {
            vertex(j, i, terrain[j][i]);
            vertex(j, i + 1, terrain[j][i + 1]);
        });
        endShape();
    });

    neuro_set('t', t + (3 * decay.get()) * amp.getLevel());
    neuro_set('tc', tc + 1);
});
