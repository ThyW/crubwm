pub struct WidgetSegment {
    widgets: Vec<Widget>,
}

pub struct Widget {
    icon: String,
    value: String,
    command: String,
    update_time: f64,
    last_update: f64,
}
