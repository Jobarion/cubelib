export function create_slider(id, min, max, min_selected, max_selected, on_set) {
    var sliderElem = document.getElementById(id);

    var slider = noUiSlider.create(sliderElem, {
        start: [min_selected, max_selected],
        step: 1,
        connect: true,
        range: {
            'min': min,
            'max': max
        }
    });
    slider.on('change', function (values) {
        on_set(values[0], values[1]);
    });
}

export function set_slider(id, min_selected, max_selected) {
    var sliderElem = document.getElementById(id);
    sliderElem.noUiSlider.set([min_selected, max_selected]);
}