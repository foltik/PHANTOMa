const bins = 64;
neuro_config({
    fft_smooth: 0.8,
    fft_bins: bins
});

neuro_setup(() => {
    neuro_source('in');
    let canvas = createCanvas(1600, 900, WEBGL);
});

neuro_init(() => {
    const bpm = 250;
    onset = new OnsetDetect(40, 120, 0.015, bpm);
    decay = new Decay(1, 1.05, 1 / (bpm / 60), 50);

    neuro_set_all({onset, decay});
});

neuro_draw(() => {
    const [onset, decay] = neuro_get_all('onset', 'decay');
    let t = neuro_get('t', 0);

    onset.detect() && decay.set(1.05);
    t += amp.getLevel() * decay.get() * 2;

    background(0);

    fill(0, 0, 0, 0);
    stroke(255, 255, 255, 255);
    strokeWeight(1);
    rotateZ(t * 0.1);
    rotateY(t * 0.1);

    const spec = fft.analyze();

    const n = bins * 0.63;
    const i1 = istep(n)(t * 0.03);
    const i2 = istep(n)(t * 0.03 + 0.5);

    push();
    range(0, 2*PI, 2*PI / n).each((f, i) => {
        const d = spec[i] / 255;
        const ir = 100 + (100 * d);
        const or = 200 + (50 * d);

        const [tx, ty] = [or * cos(f), or * sin(f)];

        let [lr, lg, lb] = rgb(0.73, d, 1);

        if (i == i1 || i == i2) {
            [lr, lg, lb] = rgb(0.73, map(Math.min(amp.getLevel(), 0.2), 0, 0.2, 0, 1));
        }

        push();
        rotate(90, createVector(tx, ty, 0));

        point(tx, ty, 0);


        if (i == i1 || i == i2) {
            strokeWeight(5);
            [lr, lg, lb] = rgb(0.73, 1, 1);
        }

        stroke(lr, lg, lb);
        circle(tx, ty, ir);

        pop();
    });
    pop();

    neuro_set('t', t);
});
